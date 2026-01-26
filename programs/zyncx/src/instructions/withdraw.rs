use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{MerkleTreeState, VaultState, VaultType, NullifierState};
use crate::errors::ZyncxError;

#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct WithdrawNative<'info> {
    #[account(mut)]
    pub recipient: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Account<'info, MerkleTreeState>,

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

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler_native(
    ctx: Context<WithdrawNative>,
    amount: u64,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],
    proof: Vec<u8>,
) -> Result<()> {
    require!(amount > 0, ZyncxError::InvalidWithdrawalAmount);

    let vault = &ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    require!(vault.vault_type == VaultType::Native, ZyncxError::VaultNotFound);

    // Get current merkle root
    let root = merkle_tree.get_root();

    // Verify ZK proof
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    
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

    // Transfer SOL from vault treasury to recipient
    let treasury_lamports = ctx.accounts.vault_treasury.lamports();
    require!(treasury_lamports >= amount, ZyncxError::InvalidWithdrawalAmount);

    **ctx.accounts.vault_treasury.try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.recipient.try_borrow_mut_lamports()? += amount;

    // Emit event
    emit!(WithdrawnEvent {
        recipient: ctx.accounts.recipient.key(),
        amount,
        nullifier,
        new_commitment,
    });

    msg!("Withdrawn {} lamports", amount);

    Ok(())
}

#[derive(Accounts)]
#[instruction(nullifier: [u8; 32])]
pub struct WithdrawToken<'info> {
    /// CHECK: Recipient account for tokens
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Account<'info, MerkleTreeState>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault_token_account", vault.key().as_ref()],
        bump,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = NullifierState::INIT_SPACE,
        seeds = [b"nullifier", vault.key().as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler_token(
    ctx: Context<WithdrawToken>,
    amount: u64,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],
    proof: Vec<u8>,
) -> Result<()> {
    require!(amount > 0, ZyncxError::InvalidWithdrawalAmount);

    let vault = &ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;
    let nullifier_account = &mut ctx.accounts.nullifier_account;

    require!(vault.vault_type == VaultType::Alternative, ZyncxError::VaultNotFound);

    // Get current merkle root
    let root = merkle_tree.get_root();

    // Verify ZK proof
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    
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

    // Transfer tokens from vault to recipient
    let vault_key = vault.key();
    let bump = &[ctx.bumps.vault_token_account];
    let seeds = &[
        b"vault_token_account".as_ref(),
        vault_key.as_ref(),
        bump.as_ref(),
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.vault_token_account.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    // Emit event
    emit!(WithdrawnEvent {
        recipient: ctx.accounts.recipient.key(),
        amount,
        nullifier,
        new_commitment,
    });

    msg!("Withdrawn {} tokens", amount);

    Ok(())
}

#[allow(unused_variables)]
fn verify_groth16_proof(proof: &[u8], public_inputs: &[[u8; 32]; 4]) -> Result<()> {
    // TODO: Integrate with groth16-solana crate for actual verification
    // Placeholder implementation
    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }

    msg!("ZK Proof verification placeholder - implement with groth16-solana");
    Ok(())
}

#[event]
pub struct WithdrawnEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub nullifier: [u8; 32],
    pub new_commitment: [u8; 32],
}
