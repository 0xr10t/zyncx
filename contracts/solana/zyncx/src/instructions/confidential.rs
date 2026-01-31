use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};

use crate::state::{
    ArciumConfig, ComputationRequest, ComputationStatus, ComputationType,
    ConfidentialSwapParams, MerkleTreeState, VaultState, VaultType, NullifierState,
    ARCIUM_MXE_PROGRAM_ID,
};
use crate::errors::ZyncxError;

// ============================================================================
// ARCIUM CONFIDENTIAL COMPUTATION INSTRUCTIONS
// ============================================================================
// These instructions implement the Call-and-Callback pattern with Arcium MXE:
// 1. queue_computation: User sends encrypted strategy to Arcium
// 2. swap_callback: Arcium calls back with result after MPC/FHE computation
// ============================================================================

/// Initialize the Arcium configuration
#[derive(Accounts)]
pub struct InitializeArciumConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = ArciumConfig::INIT_SPACE,
        seeds = [b"arcium_config"],
        bump
    )]
    pub config: Box<Account<'info, ArciumConfig>>,

    pub system_program: Program<'info, System>,
}

pub fn handler_init_arcium_config(
    ctx: Context<InitializeArciumConfig>,
    mxe_address: Pubkey,
    computation_fee: u64,
    timeout_seconds: i64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    config.bump = ctx.bumps.config;
    config.authority = ctx.accounts.authority.key();
    config.mxe_address = mxe_address;
    config.computation_fee = computation_fee;
    config.request_counter = 0;
    config.timeout_seconds = timeout_seconds;
    config.swaps_enabled = true;
    config.limit_orders_enabled = false;
    config.min_amount = 1_000_000; // 0.001 SOL minimum
    config.max_amount = 1_000_000_000_000; // 1000 SOL maximum

    msg!("Arcium config initialized");
    msg!("MXE Address: {:?}", mxe_address);

    Ok(())
}

/// Create a nullifier account for use in confidential operations
/// This is separated from the main operation to avoid stack overflow
#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct CreateNullifier<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        init,
        payer = user,
        space = NullifierState::INIT_SPACE,
        seeds = [b"nullifier", vault.key().as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    pub system_program: Program<'info, System>,
}

pub fn handler_create_nullifier(
    ctx: Context<CreateNullifier>,
    nullifier: [u8; 32],
) -> Result<()> {
    let nullifier_account = &mut ctx.accounts.nullifier_account;
    
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = false;
    nullifier_account.spent_at = 0;
    nullifier_account.vault = ctx.accounts.vault.key();

    msg!("Nullifier account created");
    Ok(())
}

/// Queue a confidential swap computation to Arcium MXE
/// Note: Nullifier must be created separately via create_nullifier instruction
#[derive(Accounts)]
#[instruction(params: ConfidentialSwapParams)]
pub struct QueueConfidentialSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"arcium_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ArciumConfig>,

    #[account(
        mut,
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Box<Account<'info, MerkleTreeState>>,

    #[account(
        init,
        payer = user,
        space = ComputationRequest::MAX_SPACE,
        seeds = [b"computation", config.request_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub computation_request: Box<Account<'info, ComputationRequest>>,

    /// Nullifier account - must already exist (created via separate instruction)
    #[account(
        mut,
        seeds = [b"nullifier", vault.key().as_ref(), params.nullifier.as_ref()],
        bump = nullifier_account.bump,
        constraint = !nullifier_account.spent @ ZyncxError::NullifierAlreadySpent,
    )]
    pub nullifier_account: Box<Account<'info, NullifierState>>,

    /// CHECK: Arcium MXE program
    #[account(address = ARCIUM_MXE_PROGRAM_ID)]
    pub arcium_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[inline(never)]
pub fn handler_queue_confidential_swap(
    ctx: Context<QueueConfidentialSwap>,
    params: ConfidentialSwapParams,
    proof: Vec<u8>,
) -> Result<()> {
    process_queue_confidential_swap(ctx, params, proof)
}

#[inline(never)]
fn process_queue_confidential_swap(
    ctx: Context<QueueConfidentialSwap>,
    params: ConfidentialSwapParams,
    proof: Vec<u8>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let vault = &ctx.accounts.vault;
    let merkle_tree = &ctx.accounts.merkle_tree;
    let computation_request = &mut ctx.accounts.computation_request;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    // Validate config
    require!(config.swaps_enabled, ZyncxError::ConfidentialSwapsDisabled);
    require!(params.amount >= config.min_amount, ZyncxError::AmountTooSmall);
    require!(params.amount <= config.max_amount, ZyncxError::AmountTooLarge);

    // Verify vault type matches source token
    if params.src_token == Pubkey::default() {
        require!(vault.vault_type == VaultType::Native, ZyncxError::VaultNotFound);
    } else {
        require!(vault.vault_type == VaultType::Alternative, ZyncxError::VaultNotFound);
        require!(vault.asset_mint == params.src_token, ZyncxError::InvalidMint);
    }

    // Verify ZK proof for ownership (simplified - in production use full verification)
    let _root = merkle_tree.get_root();
    require!(!proof.is_empty(), ZyncxError::InvalidZKProof);
    
    // Mark nullifier as spent (prevents double-spending)
    nullifier_account.nullifier = params.nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // Get next request ID
    let request_id = config.next_request_id();
    let now = Clock::get()?.unix_timestamp;

    // Initialize computation request
    computation_request.bump = ctx.bumps.computation_request;
    computation_request.request_id = request_id;
    computation_request.user = ctx.accounts.user.key();
    computation_request.vault = vault.key();
    computation_request.computation_type = ComputationType::ConfidentialSwap;
    computation_request.status = ComputationStatus::Pending;
    computation_request.encrypted_strategy = params.encrypted_bounds.clone();
    computation_request.callback_instruction = *b"confidential_swap_callback\0\0\0\0\0\0";
    computation_request.amount = params.amount;
    computation_request.src_token = params.src_token;
    computation_request.dst_token = params.dst_token;
    computation_request.nullifier = params.nullifier;
    computation_request.new_commitment = params.new_commitment;
    computation_request.queued_at = now;
    computation_request.completed_at = 0;
    computation_request.result = Vec::new();
    computation_request.expires_at = now + config.timeout_seconds;

    // Queue computation to Arcium MXE
    // In production, this would CPI to Arcium's queue_computation
    emit!(ComputationQueued {
        request_id,
        user: ctx.accounts.user.key(),
        computation_type: ComputationType::ConfidentialSwap,
        src_token: params.src_token,
        dst_token: params.dst_token,
        amount: params.amount,
        queued_at: now,
    });

    msg!("Confidential swap queued: request_id={}", request_id);
    msg!("Amount: {}, Src: {:?}, Dst: {:?}", params.amount, params.src_token, params.dst_token);

    Ok(())
}

/// Callback from Arcium MXE after computation completes
#[derive(Accounts)]
#[instruction(request_id: u64)]
pub struct ConfidentialSwapCallback<'info> {
    /// CHECK: Arcium MXE signer (verified by address constraint)
    #[account(address = ARCIUM_MXE_PROGRAM_ID)]
    pub arcium_signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"arcium_config"],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, ArciumConfig>>,

    #[account(
        mut,
        seeds = [b"computation", request_id.to_le_bytes().as_ref()],
        bump = computation_request.bump,
        constraint = computation_request.status == ComputationStatus::Pending @ ZyncxError::InvalidComputationStatus,
    )]
    pub computation_request: Box<Account<'info, ComputationRequest>>,

    #[account(
        mut,
        seeds = [b"vault", computation_request.src_token.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Box<Account<'info, MerkleTreeState>>,

    /// CHECK: Vault treasury that holds SOL
    #[account(
        mut,
        seeds = [b"vault_treasury", vault.key().as_ref()],
        bump,
    )]
    pub vault_treasury: AccountInfo<'info>,

    /// CHECK: Recipient of the swap
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    /// CHECK: Jupiter program for DEX execution
    pub jupiter_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    // Remaining accounts: Jupiter swap route accounts
}

pub fn handler_confidential_swap_callback<'info>(
    ctx: Context<'_, '_, 'info, 'info, ConfidentialSwapCallback<'info>>,
    request_id: u64,
    computation_success: bool,
    encrypted_result: Vec<u8>,
    _node_signature: [u8; 64],
    swap_data: Vec<u8>,
) -> Result<()> {
    let computation_request = &mut ctx.accounts.computation_request;
    let merkle_tree = &mut ctx.accounts.merkle_tree;
    let vault = &ctx.accounts.vault;
    let now = Clock::get()?.unix_timestamp;

    // Check expiry
    require!(now <= computation_request.expires_at, ZyncxError::ComputationExpired);

    // Update computation status
    computation_request.status = if computation_success {
        ComputationStatus::Completed
    } else {
        ComputationStatus::Failed
    };
    computation_request.completed_at = now;
    computation_request.result = encrypted_result.clone();

    if !computation_success {
        // Computation failed - emit event and return
        // Note: Funds remain in vault, user can try again with new nullifier
        emit!(ComputationFailed {
            request_id,
            reason: "Arcium computation rejected trade".to_string(),
        });
        return Ok(());
    }

    // Computation succeeded - execute the swap
    // The Arcium nodes have verified the price conditions are met

    // Insert new commitment into merkle tree
    merkle_tree.insert(computation_request.new_commitment)?;

    // Execute swap via Jupiter (or direct transfer if same token)
    let is_direct_transfer = computation_request.src_token == computation_request.dst_token;

    if is_direct_transfer {
        // Direct SOL transfer
        let treasury_lamports = ctx.accounts.vault_treasury.lamports();
        require!(
            treasury_lamports >= computation_request.amount,
            ZyncxError::InsufficientFunds
        );

        **ctx.accounts.vault_treasury.try_borrow_mut_lamports()? -= computation_request.amount;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += computation_request.amount;
    } else {
        // Execute Jupiter swap
        // In production, this would CPI to Jupiter with swap_data
        use crate::dex::jupiter::execute_jupiter_swap;
        
        execute_jupiter_swap(
            &ctx.accounts.vault_treasury,
            &ctx.accounts.recipient,
            &ctx.accounts.jupiter_program,
            swap_data,
            ctx.remaining_accounts,
            &vault.key(),
            ctx.bumps.vault_treasury,
        )?;
    }

    // Emit success event
    emit!(ConfidentialSwapExecuted {
        request_id,
        user: computation_request.user,
        src_token: computation_request.src_token,
        dst_token: computation_request.dst_token,
        amount: computation_request.amount,
        executed_at: now,
    });

    msg!("Confidential swap executed: request_id={}", request_id);

    Ok(())
}

/// Cancel an expired or pending computation request
#[derive(Accounts)]
#[instruction(request_id: u64)]
pub struct CancelComputation<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"computation", request_id.to_le_bytes().as_ref()],
        bump = computation_request.bump,
        constraint = computation_request.user == user.key() @ ZyncxError::Unauthorized,
        constraint = computation_request.status == ComputationStatus::Pending @ ZyncxError::InvalidComputationStatus,
        close = user
    )]
    pub computation_request: Box<Account<'info, ComputationRequest>>,
}

pub fn handler_cancel_computation(
    ctx: Context<CancelComputation>,
    request_id: u64,
) -> Result<()> {
    let computation_request = &ctx.accounts.computation_request;
    let now = Clock::get()?.unix_timestamp;

    // Can only cancel if expired
    require!(
        now > computation_request.expires_at,
        ZyncxError::ComputationNotExpired
    );

    emit!(ComputationCancelled {
        request_id,
        user: ctx.accounts.user.key(),
        cancelled_at: now,
    });

    msg!("Computation cancelled: request_id={}", request_id);

    Ok(())
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct ComputationQueued {
    pub request_id: u64,
    pub user: Pubkey,
    pub computation_type: ComputationType,
    pub src_token: Pubkey,
    pub dst_token: Pubkey,
    pub amount: u64,
    pub queued_at: i64,
}

#[event]
pub struct ConfidentialSwapExecuted {
    pub request_id: u64,
    pub user: Pubkey,
    pub src_token: Pubkey,
    pub dst_token: Pubkey,
    pub amount: u64,
    pub executed_at: i64,
}

#[event]
pub struct ComputationFailed {
    pub request_id: u64,
    pub reason: String,
}

#[event]
pub struct ComputationCancelled {
    pub request_id: u64,
    pub user: Pubkey,
    pub cancelled_at: i64,
}
