use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
use anchor_spl::token::{Token, TokenAccount};

use crate::dex::{
    jupiter::{execute_jupiter_swap, transfer_sol_from_treasury, JUPITER_V6_PROGRAM_ID},
    types::SwapRoute,
};
use crate::state::{MerkleTreeState, VaultState, VaultType, NullifierState, SwapParam};
use crate::errors::ZyncxError;

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

    /// CHECK: The Noir verifier program (mixer.so)
    #[account(executable)]
    pub verifier_program: AccountInfo<'info>,

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

    // Verify ZK proof via CPI to Noir verifier
    verify_noir_proof_cpi(
        &ctx.accounts.verifier_program,
        &proof,
        &root,
        &nullifier,
        &swap_param.recipient,
        swap_param.amount_in,
        &new_commitment,
    )?;
    
    msg!("ZK Proof verified successfully!");

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // For partial swaps, insert new commitment for remaining balance
    let is_partial = new_commitment != [0u8; 32];
    if is_partial {
        merkle_tree.insert(new_commitment)?;
        msg!("Partial swap: inserted change commitment into merkle tree");
    }

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

    /// CHECK: The Noir verifier program (mixer.so)
    #[account(executable)]
    pub verifier_program: AccountInfo<'info>,

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

    // Verify ZK proof via CPI to Noir verifier
    verify_noir_proof_cpi(
        &ctx.accounts.verifier_program,
        &proof,
        &root,
        &nullifier,
        &swap_param.recipient,
        swap_param.amount_in,
        &new_commitment,
    )?;
    
    msg!("ZK Proof verified successfully!");

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // For partial swaps, insert new commitment for remaining balance
    let is_partial = new_commitment != [0u8; 32];
    if is_partial {
        merkle_tree.insert(new_commitment)?;
        msg!("Partial swap: inserted change commitment into merkle tree");
    }

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

/// Verify Noir ZK proof via CPI to the deployed verifier program
/// 
/// Public inputs order (matching Noir circuit):
/// 1. root - Merkle tree root
/// 2. nullifier_hash - Prevents double-spending
/// 3. recipient - Bound to proof to prevent front-running
/// 4. withdraw_amount - Amount being swapped
/// 5. new_commitment - Change commitment for partial swaps
fn verify_noir_proof_cpi(
    verifier_program: &AccountInfo,
    proof: &[u8],
    root: &[u8; 32],
    nullifier: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
    new_commitment: &[u8; 32],
) -> Result<()> {
    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }

    // Build verifier instruction data
    let mut verifier_input = Vec::with_capacity(proof.len() + 160);
    
    // Proof bytes
    verifier_input.extend_from_slice(proof);
    
    // Public inputs (must match circuit order)
    verifier_input.extend_from_slice(root);
    verifier_input.extend_from_slice(nullifier);
    verifier_input.extend_from_slice(&recipient.to_bytes());
    
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    verifier_input.extend_from_slice(&amount_bytes);
    
    verifier_input.extend_from_slice(new_commitment);
    
    let instruction = Instruction {
        program_id: *verifier_program.key,
        accounts: vec![],
        data: verifier_input,
    };
    
    msg!("Verifying ZK proof ({} bytes)", proof.len());
    
    invoke(
        &instruction,
        &[verifier_program.clone()],
    ).map_err(|e| {
        msg!("ZK proof verification failed: {:?}", e);
        ZyncxError::InvalidZKProof
    })?;
    
    Ok(())
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
