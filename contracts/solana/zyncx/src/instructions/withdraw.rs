use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};
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

    /// CHECK: The Verifier Program deployed via Sunspot (mixer.so)
    pub verifier_program: AccountInfo<'info>,

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

    // Verify ZK proof via CPI to verifier program
    // Circuit expects public inputs: [root, nullifier_hash, recipient, amount]
    let mut verifier_input = Vec::new();
    
    // 1. Append proof bytes
    verifier_input.extend_from_slice(&proof);
    
    // 2. Public Input: Root (32 bytes)
    verifier_input.extend_from_slice(&root);
    
    // 3. Public Input: Nullifier Hash (32 bytes)
    verifier_input.extend_from_slice(&nullifier);
    
    // 4. Public Input: Recipient (32 bytes)
    verifier_input.extend_from_slice(&ctx.accounts.recipient.key().to_bytes());
    
    // 5. Public Input: Amount (32 bytes, Big Endian)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    verifier_input.extend_from_slice(&amount_bytes);
    
    // Invoke verifier program
    let instruction = Instruction {
        program_id: *ctx.accounts.verifier_program.key,
        accounts: vec![],
        data: verifier_input,
    };
    
    msg!("Invoking ZK Verifier...");
    invoke(
        &instruction,
        &[ctx.accounts.verifier_program.clone()],
    ).map_err(|_| ZyncxError::InvalidZKProof)?;
    
    msg!("ZK Proof Verified Successfully!");

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // For partial withdrawals, insert new commitment for remaining balance
    // If new_commitment is all zeros, it's a full withdrawal - no change to insert
    let is_partial_withdrawal = new_commitment != [0u8; 32];
    if is_partial_withdrawal {
        merkle_tree.insert(new_commitment)?;
        msg!("Partial withdrawal: inserted change commitment into merkle tree");
    } else {
        msg!("Full withdrawal: no change commitment needed");
    }

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
        is_partial: is_partial_withdrawal,
    });

    msg!("Withdrawn {} lamports (partial: {})", amount, is_partial_withdrawal);

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
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        mut,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Box<Account<'info, MerkleTreeState>>,

    #[account(mut)]
    pub recipient_token_account: Box<Account<'info, TokenAccount>>,

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

    /// CHECK: The Verifier Program deployed via Sunspot (mixer.so)
    pub verifier_program: AccountInfo<'info>,

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

    // Verify ZK proof via CPI to verifier program
    let mut verifier_input = Vec::new();
    
    // 1. Append proof bytes
    verifier_input.extend_from_slice(&proof);
    
    // 2. Public Input: Root (32 bytes)
    verifier_input.extend_from_slice(&root);
    
    // 3. Public Input: Nullifier Hash (32 bytes)
    verifier_input.extend_from_slice(&nullifier);
    
    // 4. Public Input: Recipient (32 bytes)
    verifier_input.extend_from_slice(&ctx.accounts.recipient.key().to_bytes());
    
    // 5. Public Input: Amount (32 bytes, Big Endian)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    verifier_input.extend_from_slice(&amount_bytes);
    
    // Invoke verifier program
    let instruction = Instruction {
        program_id: *ctx.accounts.verifier_program.key,
        accounts: vec![],
        data: verifier_input,
    };
    
    msg!("Invoking ZK Verifier...");
    invoke(
        &instruction,
        &[ctx.accounts.verifier_program.clone()],
    ).map_err(|_| ZyncxError::InvalidZKProof)?;
    
    msg!("ZK Proof Verified Successfully!");

    // Mark nullifier as spent
    nullifier_account.bump = ctx.bumps.nullifier_account;
    nullifier_account.nullifier = nullifier;
    nullifier_account.spent = true;
    nullifier_account.spent_at = Clock::get()?.unix_timestamp;
    nullifier_account.vault = vault.key();

    // For partial withdrawals, insert new commitment for remaining balance
    let is_partial_withdrawal = new_commitment != [0u8; 32];
    if is_partial_withdrawal {
        merkle_tree.insert(new_commitment)?;
        msg!("Partial withdrawal: inserted change commitment into merkle tree");
    } else {
        msg!("Full withdrawal: no change commitment needed");
    }

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
        is_partial: is_partial_withdrawal,
    });

    msg!("Withdrawn {} tokens (partial: {})", amount, is_partial_withdrawal);

    Ok(())
}



#[event]
pub struct WithdrawnEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub nullifier: [u8; 32],
    pub new_commitment: [u8; 32],
    pub is_partial: bool,
}
