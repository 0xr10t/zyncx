use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::{CallbackAccount, CircuitSource, OffChainCircuitSource};
use arcium_macros::circuit_hash;

use crate::state::{
    EncryptedVaultAccount, EncryptedUserPosition, EncryptedSwapRequest,
    SwapRequestStatus, MerkleTreeState,
};
use crate::errors::ZyncxError;

// ============================================================================
// COMPUTATION DEFINITION OFFSETS
// ============================================================================
// Auto-derived from instruction names using sha256(name)[0..4] as u32
// These must match the Arcis circuit function names in encrypted-ixs/
// ============================================================================

const COMP_DEF_OFFSET_INIT_VAULT: u32 = comp_def_offset("init_vault");
const COMP_DEF_OFFSET_INIT_POSITION: u32 = comp_def_offset("init_position");
const COMP_DEF_OFFSET_PROCESS_DEPOSIT: u32 = comp_def_offset("process_deposit");
const COMP_DEF_OFFSET_CONFIDENTIAL_SWAP: u32 = comp_def_offset("confidential_swap");
const COMP_DEF_OFFSET_COMPUTE_WITHDRAWAL: u32 = comp_def_offset("compute_withdrawal");
const COMP_DEF_OFFSET_CLEAR_POSITION: u32 = comp_def_offset("clear_position");

// ============================================================================
// 1. INIT COMPUTATION DEFINITIONS (one-time setup)
// ============================================================================
// These instructions initialize the MPC circuit definitions on-chain.
// Must be called once per circuit before any computations can be queued.
// ============================================================================

/// Initialize the init_vault computation definition
#[init_computation_definition_accounts("init_vault", payer)]
#[derive(Accounts)]
pub struct InitVaultCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    
    #[account(mut)]
    /// CHECK: Initialized by arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub fn handler_init_vault_comp_def(ctx: Context<InitVaultCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: "https://raw.githubusercontent.com/zyncx-protocol/circuits/main/init_vault.arcis".to_string(),
            hash: circuit_hash!("init_vault"),
        })),
        None,
    )?;
    msg!("init_vault computation definition initialized");
    Ok(())
}

/// Initialize the process_deposit computation definition
#[init_computation_definition_accounts("process_deposit", payer)]
#[derive(Accounts)]
pub struct InitDepositCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    
    #[account(mut)]
    /// CHECK: Initialized by arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub fn handler_init_deposit_comp_def(ctx: Context<InitDepositCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: "https://raw.githubusercontent.com/zyncx-protocol/circuits/main/process_deposit.arcis".to_string(),
            hash: circuit_hash!("process_deposit"),
        })),
        None,
    )?;
    msg!("process_deposit computation definition initialized");
    Ok(())
}

/// Initialize the confidential_swap computation definition
#[init_computation_definition_accounts("confidential_swap", payer)]
#[derive(Accounts)]
pub struct InitSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    
    #[account(mut)]
    /// CHECK: Initialized by arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub fn handler_init_swap_comp_def(ctx: Context<InitSwapCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: "https://raw.githubusercontent.com/zyncx-protocol/circuits/main/confidential_swap.arcis".to_string(),
            hash: circuit_hash!("confidential_swap"),
        })),
        None,
    )?;
    msg!("confidential_swap computation definition initialized");
    Ok(())
}

/// Initialize the compute_withdrawal computation definition
#[init_computation_definition_accounts("compute_withdrawal", payer)]
#[derive(Accounts)]
pub struct InitWithdrawalCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    
    #[account(mut)]
    /// CHECK: Initialized by arcium program
    pub comp_def_account: UncheckedAccount<'info>,
    
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

pub fn handler_init_withdrawal_comp_def(ctx: Context<InitWithdrawalCompDef>) -> Result<()> {
    init_comp_def(
        ctx.accounts,
        Some(CircuitSource::OffChain(OffChainCircuitSource {
            source: "https://raw.githubusercontent.com/zyncx-protocol/circuits/main/compute_withdrawal.arcis".to_string(),
            hash: circuit_hash!("compute_withdrawal"),
        })),
        None,
    )?;
    msg!("compute_withdrawal computation definition initialized");
    Ok(())
}

// ============================================================================
// 2. QUEUE COMPUTATION INSTRUCTIONS
// ============================================================================
// These instructions queue encrypted computations to the MXE.
// User provides encrypted parameters; MXE processes without seeing plaintext.
// ============================================================================

/// Queue an encrypted deposit computation
#[queue_computation_accounts("process_deposit", user)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct QueueEncryptedDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed, space = 9, payer = user,
        seeds = [&SIGN_PDA_SEED], bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(mut, address = derive_mempool_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub mempool_account: UncheckedAccount<'info>,

    #[account(mut, address = derive_execpool_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub executing_pool: UncheckedAccount<'info>,

    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ZyncxError::ClusterNotSet))]
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(mut, address = derive_cluster_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,

    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,

    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,

    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,

    // Custom accounts
    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + EncryptedUserPosition::INIT_SPACE,
        seeds = [b"enc_position", vault.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub user_position: Box<Account<'info, EncryptedUserPosition>>,
}

/// Parameters for encrypted deposit
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EncryptedDepositParams {
    /// Client's X25519 public key for encryption
    pub encryption_pubkey: [u8; 32],
    /// Nonce for the encrypted amount
    pub amount_nonce: u128,
    /// Encrypted deposit amount (ciphertext)
    pub encrypted_amount: [u8; 32],
}

pub fn handler_queue_encrypted_deposit(
    ctx: Context<QueueEncryptedDeposit>,
    computation_offset: u64,
    params: EncryptedDepositParams,
) -> Result<()> {
    // Build arguments matching circuit: process_deposit(
    //   deposit_input: Enc<Shared, DepositInput>,
    //   vault_state: Enc<Mxe, VaultState>,
    //   user_position: Enc<Mxe, UserPosition>,
    // )
    let args = ArgBuilder::new()
        // Enc<Shared, DepositInput>: pubkey + nonce + ciphertext
        .x25519_pubkey(params.encryption_pubkey)
        .plaintext_u128(params.amount_nonce)
        .encrypted_u64(params.encrypted_amount)
        // Enc<Mxe, VaultState>: nonce + account
        .plaintext_u128(ctx.accounts.vault.nonce)
        .account(
            ctx.accounts.vault.key(),
            EncryptedVaultAccount::ENCRYPTED_STATE_OFFSET,
            EncryptedVaultAccount::ENCRYPTED_STATE_SIZE,
        )
        // Enc<Mxe, UserPosition>: nonce + account
        .plaintext_u128(ctx.accounts.user_position.nonce)
        .account(
            ctx.accounts.user_position.key(),
            EncryptedUserPosition::ENCRYPTED_STATE_OFFSET,
            EncryptedUserPosition::ENCRYPTED_STATE_SIZE,
        )
        .build();

    // Queue computation with callback
    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![DepositCallback::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[
                CallbackAccount {
                    pubkey: ctx.accounts.vault.key(),
                    is_signer: false,
                    is_writable: true,
                },
                CallbackAccount {
                    pubkey: ctx.accounts.user_position.key(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
        )?],
        2, // num_return_outputs (VaultState, UserPosition)
        0, // reserved
    )?;

    msg!("Encrypted deposit queued with offset: {}", computation_offset);
    Ok(())
}

/// Queue a confidential swap computation
#[queue_computation_accounts("confidential_swap", user)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct QueueConfidentialSwapMxe<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed, space = 9, payer = user,
        seeds = [&SIGN_PDA_SEED], bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(mut, address = derive_mempool_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub mempool_account: UncheckedAccount<'info>,

    #[account(mut, address = derive_execpool_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub executing_pool: UncheckedAccount<'info>,

    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ZyncxError::ClusterNotSet))]
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(mut, address = derive_cluster_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,

    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,

    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,

    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,

    // Custom accounts
    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    #[account(mut)]
    pub user_position: Box<Account<'info, EncryptedUserPosition>>,
    
    #[account(
        init,
        payer = user,
        space = 8 + EncryptedSwapRequest::INIT_SPACE,
        seeds = [b"swap_request", computation_offset.to_le_bytes().as_ref()],
        bump,
    )]
    pub swap_request: Box<Account<'info, EncryptedSwapRequest>>,
    
    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump,
    )]
    pub merkle_tree: Box<Account<'info, MerkleTreeState>>,
}

/// Parameters for confidential swap
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConfidentialSwapMxeParams {
    /// Client's X25519 public key
    pub encryption_pubkey: [u8; 32],
    /// Nonce for encrypted bounds
    pub bounds_nonce: u128,
    /// Encrypted minimum output [u8; 32]
    pub encrypted_min_out: [u8; 32],
    /// Encrypted max slippage bps [u8; 32]
    pub encrypted_max_slippage: [u8; 32],
    /// Encrypted aggressive flag [u8; 32]
    pub encrypted_aggressive: [u8; 32],
    /// Swap amount (plaintext, validated by ZK proof)
    pub amount: u64,
    /// Current price from oracle (plaintext)
    pub current_price: u64,
    /// Nullifier from ZK proof
    pub nullifier: [u8; 32],
    /// New commitment for Merkle tree
    pub new_commitment: [u8; 32],
    /// ZK proof bytes
    pub proof: Vec<u8>,
}

pub fn handler_queue_confidential_swap_mxe(
    ctx: Context<QueueConfidentialSwapMxe>,
    computation_offset: u64,
    params: ConfidentialSwapMxeParams,
) -> Result<()> {
    // Verify ZK proof (simplified - in production use full verification)
    require!(!params.proof.is_empty(), ZyncxError::InvalidZKProof);
    
    // Store swap request metadata
    let swap_request = &mut ctx.accounts.swap_request;
    swap_request.bump = ctx.bumps.swap_request;
    swap_request.user = ctx.accounts.user.key();
    swap_request.source_vault = ctx.accounts.vault.key();
    swap_request.dest_vault = ctx.accounts.vault.key(); // Same vault for now
    swap_request.computation_offset = computation_offset;
    swap_request.encrypted_bounds = [
        params.encrypted_min_out,
        params.encrypted_max_slippage,
        params.encrypted_aggressive,
    ];
    swap_request.bounds_nonce = params.bounds_nonce;
    swap_request.client_pubkey = params.encryption_pubkey;
    swap_request.amount = params.amount;
    swap_request.nullifier = params.nullifier;
    swap_request.new_commitment = params.new_commitment;
    swap_request.status = SwapRequestStatus::Pending;
    swap_request.queued_at = Clock::get()?.unix_timestamp;

    // Build arguments matching circuit: confidential_swap(
    //   swap_bounds: Enc<Shared, SwapBounds>,
    //   vault_state: Enc<Mxe, VaultState>,
    //   user_position: Enc<Mxe, UserPosition>,
    //   swap_amount: u64,
    //   current_price: u64,
    // )
    let args = ArgBuilder::new()
        // Enc<Shared, SwapBounds>: pubkey + nonce + encrypted fields
        .x25519_pubkey(params.encryption_pubkey)
        .plaintext_u128(params.bounds_nonce)
        .encrypted_u64(params.encrypted_min_out)
        .encrypted_u16(params.encrypted_max_slippage)
        .encrypted_bool(params.encrypted_aggressive)
        // Enc<Mxe, VaultState>: nonce + account
        .plaintext_u128(ctx.accounts.vault.nonce)
        .account(
            ctx.accounts.vault.key(),
            EncryptedVaultAccount::ENCRYPTED_STATE_OFFSET,
            EncryptedVaultAccount::ENCRYPTED_STATE_SIZE,
        )
        // Enc<Mxe, UserPosition>: nonce + account
        .plaintext_u128(ctx.accounts.user_position.nonce)
        .account(
            ctx.accounts.user_position.key(),
            EncryptedUserPosition::ENCRYPTED_STATE_OFFSET,
            EncryptedUserPosition::ENCRYPTED_STATE_SIZE,
        )
        // Plaintext params
        .plaintext_u64(params.amount)
        .plaintext_u64(params.current_price)
        .build();

    // Queue computation with callback
    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![ConfidentialSwapCallbackMxe::callback_ix(
            computation_offset,
            &ctx.accounts.mxe_account,
            &[
                CallbackAccount {
                    pubkey: ctx.accounts.swap_request.key(),
                    is_signer: false,
                    is_writable: true,
                },
                CallbackAccount {
                    pubkey: ctx.accounts.vault.key(),
                    is_signer: false,
                    is_writable: true,
                },
                CallbackAccount {
                    pubkey: ctx.accounts.user_position.key(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
        )?],
        3, // num_return_outputs (SwapResult, VaultState, UserPosition)
        0, // reserved
    )?;

    msg!("Confidential swap queued with offset: {}", computation_offset);
    Ok(())
}

// ============================================================================
// 3. CALLBACK INSTRUCTIONS
// ============================================================================
// These instructions receive results from the MXE after computation completes.
// They update on-chain state with the encrypted outputs.
// ============================================================================

/// Callback for deposit computation
#[callback_accounts("process_deposit")]
#[derive(Accounts)]
pub struct DepositCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: Verified by arcium program
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_cluster_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions sysvar
    pub instructions_sysvar: AccountInfo<'info>,

    // Custom accounts (must match CallbackAccount order)
    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    #[account(mut)]
    pub user_position: Box<Account<'info, EncryptedUserPosition>>,
}

/// Output type for process_deposit callback
/// Circuit returns: (Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>)
#[derive(AnchorDeserialize)]
pub struct DepositOutput {
    pub field_0: DepositOutputTuple,
}

#[derive(ArciumDeserialize)]
pub struct DepositOutputTuple {
    /// Updated vault state (3 ciphertexts)
    pub field_0: EncryptedVaultState,
    /// Updated user position (2 ciphertexts)
    pub field_1: EncryptedUserPositionState,
}

#[derive(ArciumDeserialize)]
pub struct EncryptedVaultState {
    pub ciphertexts: [[u8; 32]; 3],
    pub nonce: u128,
}

#[derive(ArciumDeserialize)]
pub struct EncryptedUserPositionState {
    pub ciphertexts: [[u8; 32]; 2],
    pub nonce: u128,
}

#[arcium_callback(encrypted_ix = "process_deposit")]
pub fn deposit_callback(
    ctx: Context<DepositCallback>,
    output: SignedComputationOutputs<DepositOutput>,
) -> Result<()> {
    // Verify output signature from cluster
    let tuple = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(DepositOutput { field_0 }) => field_0,
        Err(_) => return Err(ZyncxError::AbortedComputation.into()),
    };

    // Update vault state
    ctx.accounts.vault.vault_state = tuple.field_0.ciphertexts;
    ctx.accounts.vault.nonce = tuple.field_0.nonce;

    // Update user position state
    ctx.accounts.user_position.position_state = tuple.field_1.ciphertexts;
    ctx.accounts.user_position.nonce = tuple.field_1.nonce;

    msg!("Deposit callback completed successfully");
    Ok(())
}

/// Callback for confidential swap computation
#[callback_accounts("confidential_swap")]
#[derive(Accounts)]
pub struct ConfidentialSwapCallbackMxe<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: Verified by arcium program
    pub computation_account: UncheckedAccount<'info>,

    #[account(address = derive_cluster_pda!(mxe_account, ZyncxError::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions sysvar
    pub instructions_sysvar: AccountInfo<'info>,

    // Custom accounts (must match CallbackAccount order)
    #[account(mut)]
    pub swap_request: Box<Account<'info, EncryptedSwapRequest>>,
    
    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    #[account(mut)]
    pub user_position: Box<Account<'info, EncryptedUserPosition>>,
}

/// Output type for confidential_swap callback
/// Circuit returns: (Enc<Shared, SwapResult>, Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>)
#[derive(AnchorDeserialize)]
pub struct SwapOutput {
    pub field_0: SwapOutputTuple,
}

#[derive(ArciumDeserialize)]
pub struct SwapOutputTuple {
    /// Swap result (encrypted for client)
    pub field_0: EncryptedSwapResult,
    /// Updated vault state
    pub field_1: EncryptedVaultState,
    /// Updated user position
    pub field_2: EncryptedUserPositionState,
}

#[derive(ArciumDeserialize)]
pub struct EncryptedSwapResult {
    /// [should_execute, min_amount_out]
    pub ciphertexts: [[u8; 32]; 2],
    pub nonce: u128,
}

#[arcium_callback(encrypted_ix = "confidential_swap")]
pub fn confidential_swap_callback(
    ctx: Context<ConfidentialSwapCallbackMxe>,
    output: SignedComputationOutputs<SwapOutput>,
) -> Result<()> {
    // Verify output signature
    let tuple = match output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    ) {
        Ok(SwapOutput { field_0 }) => field_0,
        Err(_) => {
            ctx.accounts.swap_request.status = SwapRequestStatus::Failed;
            return Err(ZyncxError::AbortedComputation.into());
        }
    };

    // Update swap request with result
    ctx.accounts.swap_request.encrypted_result = tuple.field_0.ciphertexts;
    ctx.accounts.swap_request.result_nonce = tuple.field_0.nonce;
    ctx.accounts.swap_request.status = SwapRequestStatus::Completed;
    ctx.accounts.swap_request.completed_at = Clock::get()?.unix_timestamp;

    // Update vault state
    ctx.accounts.vault.vault_state = tuple.field_1.ciphertexts;
    ctx.accounts.vault.nonce = tuple.field_1.nonce;

    // Update user position
    ctx.accounts.user_position.position_state = tuple.field_2.ciphertexts;
    ctx.accounts.user_position.nonce = tuple.field_2.nonce;

    msg!("Confidential swap callback completed successfully");
    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create encrypted vault account
#[derive(Accounts)]
pub struct CreateEncryptedVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + EncryptedVaultAccount::INIT_SPACE,
        seeds = [b"enc_vault", token_mint.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    /// CHECK: Token mint
    pub token_mint: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler_create_encrypted_vault(ctx: Context<CreateEncryptedVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.bump = ctx.bumps.vault;
    vault.authority = ctx.accounts.authority.key();
    vault.token_mint = ctx.accounts.token_mint.key();
    vault.vault_state = [[0u8; 32]; 3]; // Zeroed until init_vault MPC completes
    vault.nonce = 0;
    vault.meta_nonce = 0;
    vault.created_at = Clock::get()?.unix_timestamp;
    
    msg!("Encrypted vault account created");
    Ok(())
}
