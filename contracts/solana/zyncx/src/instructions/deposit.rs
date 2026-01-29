use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::{MerkleTreeState, VaultState, VaultType, poseidon_hash_commitment};
use crate::errors::ZyncxError;

#[derive(Accounts)]
pub struct DepositNative<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

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

    pub system_program: Program<'info, System>,
}

pub fn handler_native(
    ctx: Context<DepositNative>,
    amount: u64,
    precommitment: [u8; 32],
) -> Result<[u8; 32]> {
    require!(amount > 0, ZyncxError::InvalidDepositAmount);

    let vault = &mut ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;

    require!(vault.vault_type == VaultType::Native, ZyncxError::VaultNotFound);

    // Transfer SOL from depositor to vault treasury
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.depositor.to_account_info(),
                to: ctx.accounts.vault_treasury.to_account_info(),
            },
        ),
        amount,
    )?;

    // Generate commitment = hash(amount, precommitment)
    let commitment = poseidon_hash_commitment(amount, precommitment)?;

    // Insert commitment into merkle tree
    merkle_tree.insert(commitment)?;

    // Update vault state
    vault.nonce += 1;
    vault.total_deposited = vault.total_deposited
        .checked_add(amount)
        .ok_or(ZyncxError::ArithmeticOverflow)?;

    // Emit event
    emit!(DepositedEvent {
        depositor: ctx.accounts.depositor.key(),
        amount,
        commitment,
        precommitment,
    });

    msg!("Deposited {} lamports", amount);
    msg!("Commitment: {:?}", commitment);

    Ok(commitment)
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

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

    #[account(mut)]
    pub depositor_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"vault_token_account", vault.key().as_ref()],
        bump,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler_token(
    ctx: Context<DepositToken>,
    amount: u64,
    precommitment: [u8; 32],
) -> Result<[u8; 32]> {
    require!(amount > 0, ZyncxError::InvalidDepositAmount);

    let vault = &mut ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;

    require!(vault.vault_type == VaultType::Alternative, ZyncxError::VaultNotFound);

    // Transfer tokens from depositor to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount,
    )?;

    // Generate commitment = hash(amount, precommitment)
    let commitment = poseidon_hash_commitment(amount, precommitment)?;

    // Insert commitment into merkle tree
    merkle_tree.insert(commitment)?;

    // Update vault state
    vault.nonce += 1;
    vault.total_deposited = vault.total_deposited
        .checked_add(amount)
        .ok_or(ZyncxError::ArithmeticOverflow)?;

    // Emit event
    emit!(DepositedEvent {
        depositor: ctx.accounts.depositor.key(),
        amount,
        commitment,
        precommitment,
    });

    msg!("Deposited {} tokens", amount);
    msg!("Commitment: {:?}", commitment);

    Ok(commitment)
}

#[event]
pub struct DepositedEvent {
    pub depositor: Pubkey,
    pub amount: u64,
    pub commitment: [u8; 32],
    pub precommitment: [u8; 32],
}
