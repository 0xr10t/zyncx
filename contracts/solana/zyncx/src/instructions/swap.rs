use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::dex::{
    jupiter::{execute_jupiter_swap, transfer_sol_from_treasury, JUPITER_V6_PROGRAM_ID},
    types::SwapRoute,
};
use crate::state::{MerkleTreeState, VaultState, VaultType, NullifierState, SwapParam};
use crate::errors::ZyncxError;

// ============================================================================
// CROSS-TOKEN SWAP - The Main Swap Instruction
// ============================================================================
// This instruction handles swaps between different tokens (SOL → USDC etc.)
//
// CRITICAL: Cross-token swaps need TWO vaults:
// - Source vault: Where the nullifier is created (spent)
// - Destination vault: Where the new commitment is inserted
//
// Example: Swap 10 SOL → 1000 USDC
// 1. Verify ZK proof (proves ownership of SOL commitment)
// 2. Nullify SOL commitment in SOL vault
// 3. Execute Jupiter swap (10 SOL → 1000 USDC)
// 4. Insert USDC commitment in USDC vault  
// ============================================================================

#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct CrossTokenSwap<'info> {
    /// CHECK: Final recipient (for direct transfers) or Jupiter route output
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    // === SOURCE VAULT (where commitment is being spent) ===
    #[account(
        mut,
        seeds = [b"vault", src_vault.asset_mint.as_ref()],
        bump = src_vault.bump,
    )]
    pub src_vault: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        seeds = [b"merkle_tree", src_vault.key().as_ref()],
        bump = src_merkle_tree.bump,
    )]
    pub src_merkle_tree: Box<Account<'info, MerkleTreeState>>,

    /// CHECK: Source vault treasury (holds SOL or is token account)
    #[account(
        mut,
        seeds = [b"vault_treasury", src_vault.key().as_ref()],
        bump,
    )]
    pub src_vault_treasury: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        space = NullifierState::INIT_SPACE,
        seeds = [b"nullifier", src_vault.key().as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    // === DESTINATION VAULT (where new commitment goes) ===
    #[account(
        mut,
        seeds = [b"vault", dst_vault.asset_mint.as_ref()],
        bump = dst_vault.bump,
        constraint = dst_vault.key() != src_vault.key() @ ZyncxError::SameVaultSwap,
    )]
    pub dst_vault: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        seeds = [b"merkle_tree", dst_vault.key().as_ref()],
        bump = dst_merkle_tree.bump,
    )]
    pub dst_merkle_tree: Box<Account<'info, MerkleTreeState>>,

    /// CHECK: Jupiter V6 program for DEX aggregation
    #[account(address = JUPITER_V6_PROGRAM_ID)]
    pub jupiter_program: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    // Remaining accounts: All accounts required by Jupiter swap route
}

pub fn handler_cross_token<'info>(
    ctx: Context<'_, '_, 'info, 'info, CrossTokenSwap<'info>>,
    swap_param: SwapParam,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],  // This goes into DESTINATION vault!
    proof: Vec<u8>,
    swap_data: Vec<u8>,
) -> Result<()> {
    require!(swap_param.amount_in > 0, ZyncxError::InvalidSwapAmount);
    
    // Verify source and destination tokens match the vaults
    require!(
        swap_param.src_token == ctx.accounts.src_vault.asset_mint,
        ZyncxError::TokenMintMismatch
    );
    require!(
        swap_param.dst_token == ctx.accounts.dst_vault.asset_mint,
        ZyncxError::TokenMintMismatch
    );

    let src_vault = &ctx.accounts.src_vault;
    let src_merkle_tree = &mut ctx.accounts.src_merkle_tree;
    let dst_merkle_tree = &mut ctx.accounts.dst_merkle_tree;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    // Get source merkle root (for proof verification)
    let src_root = src_merkle_tree.get_root();

    // ========================================================================
    // Verify ZK proof (swap_circuit from Noir)
    // ========================================================================
    // Public inputs for swap_circuit:
    // - src_root: Source vault merkle root
    // - src_nullifier_hash: Nullifier hash
    // - src_token_mint_public: Source token mint (32 bytes)
    // - dst_token_mint_public: Destination token mint (32 bytes)
    // - dst_commitment: New commitment for destination vault
    // - min_dst_amount: Minimum output amount (slippage protection)
    
    let mut src_mint_bytes = [0u8; 32];
    src_mint_bytes.copy_from_slice(swap_param.src_token.as_ref());
    
    let mut dst_mint_bytes = [0u8; 32];
    dst_mint_bytes.copy_from_slice(swap_param.dst_token.as_ref());
    
    let mut min_amount_bytes = [0u8; 32];
    min_amount_bytes[24..32].copy_from_slice(&swap_param.min_amount_out.to_be_bytes());
    
    let public_inputs = SwapPublicInputs {
        src_root,
        nullifier,
        src_token_mint: src_mint_bytes,
        dst_token_mint: dst_mint_bytes,
        dst_commitment: new_commitment,
        min_dst_amount: min_amount_bytes,
    };
    
    verify_swap_proof(&proof, &public_inputs)?;

    // ========================================================================
    // Mark nullifier as spent in SOURCE vault
    // ========================================================================
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = src_vault.key();

    // ========================================================================
    // CRITICAL: Insert new commitment into DESTINATION vault's merkle tree
    // ========================================================================
    dst_merkle_tree.insert(new_commitment)?;

    // ========================================================================
    // Execute Jupiter swap
    // ========================================================================
    execute_jupiter_swap(
        &ctx.accounts.src_vault_treasury,
        &ctx.accounts.recipient,
        &ctx.accounts.jupiter_program,
        swap_data,
        ctx.remaining_accounts,
        &src_vault.key(),
        ctx.bumps.src_vault_treasury,
    )?;

    // Emit event
    emit!(CrossTokenSwapEvent {
        recipient: swap_param.recipient,
        src_token: swap_param.src_token,
        dst_token: swap_param.dst_token,
        amount_in: swap_param.amount_in,
        min_amount_out: swap_param.min_amount_out,
        nullifier,
        new_commitment,
        src_vault: ctx.accounts.src_vault.key(),
        dst_vault: ctx.accounts.dst_vault.key(),
    });

    msg!("Cross-token swap: {} → {} via Jupiter", swap_param.src_token, swap_param.dst_token);

    Ok(())
}

// ============================================================================
// SAME-TOKEN OPERATIONS (Withdrawal/Transfer within same vault)
// ============================================================================

#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct SwapNative<'info> {
    /// CHECK: Recipient of swapped tokens (or intermediate token account)
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

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

    /// CHECK: Vault PDA that holds SOL
    #[account(
        mut,
        seeds = [b"vault_treasury", vault.key().as_ref()],
        bump,
    )]
    pub vault_treasury: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        space = NullifierState::INIT_SPACE,
        seeds = [b"nullifier", vault.key().as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    /// CHECK: Jupiter V6 program for DEX aggregation
    #[account(address = JUPITER_V6_PROGRAM_ID)]
    pub jupiter_program: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    // Remaining accounts: All accounts required by Jupiter swap route
}

pub fn handler_native<'info>(
    ctx: Context<'_, '_, 'info, 'info, SwapNative<'info>>,
    swap_param: SwapParam,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],
    proof: Vec<u8>,
    swap_data: Vec<u8>,
) -> Result<()> {
    require!(swap_param.amount_in > 0, ZyncxError::InvalidSwapAmount);

    let vault = &ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    require!(vault.vault_type == VaultType::Native, ZyncxError::VaultNotFound);

    // Get current merkle root
    let root = merkle_tree.get_root();

    // Verify ZK proof
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&swap_param.amount_in.to_be_bytes());
    
    let public_inputs = [
        amount_bytes,
        root,
        new_commitment,
        nullifier,
    ];
    
    verify_groth16_proof(&proof, &public_inputs)?;

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // Insert new commitment into merkle tree
    merkle_tree.insert(new_commitment)?;

    // Check if this is a direct transfer (same token) or a swap
    let is_direct_transfer = swap_param.src_token == swap_param.dst_token;

    if is_direct_transfer {
        // Direct SOL transfer - no swap needed
        transfer_sol_from_treasury(
            &ctx.accounts.vault_treasury,
            &ctx.accounts.recipient,
            swap_param.amount_in,
            &vault.key(),
            ctx.bumps.vault_treasury,
        )?;
    } else {
        // Execute swap via Jupiter
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

    // Emit event
    emit!(SwappedEvent {
        recipient: swap_param.recipient,
        src_token: swap_param.src_token,
        dst_token: swap_param.dst_token,
        amount_in: swap_param.amount_in,
        min_amount_out: swap_param.min_amount_out,
        nullifier,
        new_commitment,
    });

    msg!("Swapped {} lamports via Jupiter", swap_param.amount_in);

    Ok(())
}

#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct SwapToken<'info> {
    /// CHECK: Recipient of swapped tokens
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

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
        mut,
        seeds = [b"vault_token_account", vault.key().as_ref()],
        bump,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        space = NullifierState::INIT_SPACE,
        seeds = [b"nullifier", vault.key().as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    /// CHECK: Jupiter V6 program for DEX aggregation
    #[account(address = JUPITER_V6_PROGRAM_ID)]
    pub jupiter_program: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    // Remaining accounts: All accounts required by Jupiter swap route
}

pub fn handler_token<'info>(
    ctx: Context<'_, '_, 'info, 'info, SwapToken<'info>>,
    swap_param: SwapParam,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],
    proof: Vec<u8>,
    swap_data: Vec<u8>,
) -> Result<()> {
    require!(swap_param.amount_in > 0, ZyncxError::InvalidSwapAmount);

    let vault = &ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    require!(vault.vault_type == VaultType::Alternative, ZyncxError::VaultNotFound);

    // Get current merkle root
    let root = merkle_tree.get_root();

    // Verify ZK proof
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&swap_param.amount_in.to_be_bytes());
    
    let public_inputs = [
        amount_bytes,
        root,
        new_commitment,
        nullifier,
    ];
    
    verify_groth16_proof(&proof, &public_inputs)?;

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // Insert new commitment into merkle tree
    merkle_tree.insert(new_commitment)?;

    // Check if this is a direct transfer (same token) or a swap
    let is_direct_transfer = swap_param.src_token == swap_param.dst_token;

    if is_direct_transfer {
        // Direct token transfer - no swap needed
        use crate::dex::jupiter::transfer_tokens_from_vault;
        transfer_tokens_from_vault(
            &ctx.accounts.vault_token_account,
            &ctx.accounts.recipient,
            &ctx.accounts.token_program,
            swap_param.amount_in,
            &vault.key(),
            ctx.bumps.vault_token_account,
        )?;
    } else {
        // Execute swap via Jupiter
        execute_jupiter_swap(
            &ctx.accounts.vault_token_account.to_account_info(),
            &ctx.accounts.recipient,
            &ctx.accounts.jupiter_program,
            swap_data,
            ctx.remaining_accounts,
            &vault.key(),
            ctx.bumps.vault_token_account,
        )?;
    }

    // Emit event
    emit!(SwappedEvent {
        recipient: swap_param.recipient,
        src_token: swap_param.src_token,
        dst_token: swap_param.dst_token,
        amount_in: swap_param.amount_in,
        min_amount_out: swap_param.min_amount_out,
        nullifier,
        new_commitment,
    });

    msg!("Swapped {} tokens via Jupiter", swap_param.amount_in);

    Ok(())
}

// ============================================================================
// PROOF VERIFICATION
// ============================================================================

/// Public inputs for swap_circuit (cross-token swap)
struct SwapPublicInputs {
    pub src_root: [u8; 32],
    pub nullifier: [u8; 32],
    pub src_token_mint: [u8; 32],
    pub dst_token_mint: [u8; 32],
    pub dst_commitment: [u8; 32],
    pub min_dst_amount: [u8; 32],
}

#[allow(unused_variables)]
fn verify_swap_proof(proof: &[u8], public_inputs: &SwapPublicInputs) -> Result<()> {
    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }
    // TODO: Implement actual Groth16/Noir proof verification
    // This will use groth16-solana or similar library
    msg!("Swap ZK Proof verification - implement with groth16-solana");
    Ok(())
}

#[allow(unused_variables)]
fn verify_groth16_proof(proof: &[u8], public_inputs: &[[u8; 32]; 4]) -> Result<()> {
    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }
    msg!("ZK Proof verification placeholder - implement with Arcium/groth16-solana");
    Ok(())
}

#[event]
pub struct CrossTokenSwapEvent {
    pub recipient: Pubkey,
    pub src_token: Pubkey,
    pub dst_token: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub nullifier: [u8; 32],
    pub new_commitment: [u8; 32],
    pub src_vault: Pubkey,
    pub dst_vault: Pubkey,
}

#[event]
pub struct SwappedEvent {
    pub recipient: Pubkey,
    pub src_token: Pubkey,
    pub dst_token: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub nullifier: [u8; 32],
    pub new_commitment: [u8; 32],
}
