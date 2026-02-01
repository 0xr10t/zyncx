use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use anchor_spl::token::{Token, TokenAccount};

use crate::errors::ZyncxError;
use super::types::{SwapRoute, SwapResult};

/// Jupiter V6 Program ID (same on mainnet, devnet, and testnet)
/// Address: JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4
pub const JUPITER_V6_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    4, 121, 213, 48, 116, 81, 157, 101, 44, 107, 87, 187, 156, 14, 46, 133,
    234, 70, 27, 233, 81, 253, 66, 216, 115, 137, 101, 85, 18, 37, 59, 194
]);

/// Execute a swap through Jupiter aggregator
/// 
/// This function uses Jupiter's shared accounts model where swap instructions
/// are constructed off-chain and passed via remaining_accounts.
/// 
/// # Arguments
/// * `vault_treasury` - The PDA holding the source funds
/// * `destination` - The account to receive swapped tokens  
/// * `jupiter_program` - Jupiter V6 program account
/// * `swap_data` - Serialized Jupiter swap instruction data (from Jupiter API)
/// * `remaining_accounts` - All accounts required by Jupiter swap
/// * `vault_key` - The vault's public key (for PDA signing)
/// * `treasury_bump` - Bump seed for vault treasury PDA
pub fn execute_jupiter_swap<'info>(
    vault_treasury: &AccountInfo<'info>,
    destination: &AccountInfo<'info>,
    jupiter_program: &AccountInfo<'info>,
    swap_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    treasury_bump: u8,
) -> Result<SwapResult> {
    // Verify Jupiter program ID
    require!(
        jupiter_program.key() == JUPITER_V6_PROGRAM_ID,
        ZyncxError::InvalidSwapRouter
    );

    // Build account metas for Jupiter instruction
    let mut account_metas: Vec<AccountMeta> = Vec::with_capacity(remaining_accounts.len() + 2);
    
    // Add vault treasury as source (signer via PDA)
    account_metas.push(AccountMeta {
        pubkey: vault_treasury.key(),
        is_signer: true,
        is_writable: true,
    });

    // Add destination account
    account_metas.push(AccountMeta {
        pubkey: destination.key(),
        is_signer: false,
        is_writable: true,
    });

    // Add all remaining accounts from Jupiter route
    for account in remaining_accounts {
        account_metas.push(AccountMeta {
            pubkey: account.key(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        });
    }

    // Create Jupiter swap instruction
    let jupiter_ix = Instruction {
        program_id: jupiter_program.key(),
        accounts: account_metas,
        data: swap_data,
    };

    // PDA signer seeds for vault treasury
    let treasury_seeds = &[
        b"vault_treasury",
        vault_key.as_ref(),
        &[treasury_bump],
    ];
    let signer_seeds = &[&treasury_seeds[..]];

    // Collect all account infos for CPI
    let mut account_infos: Vec<AccountInfo> = Vec::with_capacity(remaining_accounts.len() + 3);
    account_infos.push(jupiter_program.clone());
    account_infos.push(vault_treasury.clone());
    account_infos.push(destination.clone());
    account_infos.extend(remaining_accounts.iter().cloned());

    // Execute Jupiter swap via CPI
    invoke_signed(&jupiter_ix, &account_infos, signer_seeds)?;

    msg!("Jupiter swap executed successfully");

    // Return placeholder result - actual amounts come from Jupiter's return data
    Ok(SwapResult {
        amount_in: 0,  // Would parse from return data
        amount_out: 0, // Would parse from return data
        fee_amount: 0,
    })
}

/// Execute a simple SOL transfer from vault treasury to recipient
/// Used when no swap is needed (withdrawing same token)
pub fn transfer_sol_from_treasury<'info>(
    vault_treasury: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    amount: u64,
    _vault_key: &Pubkey,
    _treasury_bump: u8,
) -> Result<()> {
    // Verify sufficient balance
    let treasury_balance = vault_treasury.lamports();
    require!(treasury_balance >= amount, ZyncxError::InsufficientFunds);

    // Transfer SOL
    **vault_treasury.try_borrow_mut_lamports()? -= amount;
    **recipient.try_borrow_mut_lamports()? += amount;

    msg!("Transferred {} lamports from treasury to recipient", amount);
    Ok(())
}

/// Execute a token transfer from vault token account to recipient
pub fn transfer_tokens_from_vault<'info>(
    vault_token_account: &Account<'info, TokenAccount>,
    recipient_token_account: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
    vault_key: &Pubkey,
    token_account_bump: u8,
) -> Result<()> {
    use anchor_spl::token::{transfer, Transfer};

    let seeds = &[
        b"vault_token_account",
        vault_key.as_ref(),
        &[token_account_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: vault_token_account.to_account_info(),
                to: recipient_token_account.clone(),
                authority: vault_token_account.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    msg!("Transferred {} tokens from vault to recipient", amount);
    Ok(())
}

/// Swap SOL to SPL token via Jupiter
/// This wraps SOL to WSOL, executes the swap, then unwraps if needed
pub fn swap_sol_to_token<'info>(
    vault_treasury: &AccountInfo<'info>,
    _wsol_account: &AccountInfo<'info>,
    destination_token_account: &AccountInfo<'info>,
    jupiter_program: &AccountInfo<'info>,
    _token_program: &AccountInfo<'info>,
    _swap_route: &SwapRoute,
    swap_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    treasury_bump: u8,
) -> Result<SwapResult> {
    // For SOL -> Token swaps:
    // 1. Wrap SOL to WSOL (sync native)
    // 2. Execute Jupiter swap WSOL -> Target token
    // Jupiter handles the wrapping internally in most cases

    execute_jupiter_swap(
        vault_treasury,
        destination_token_account,
        jupiter_program,
        swap_data,
        remaining_accounts,
        vault_key,
        treasury_bump,
    )
}

/// Swap SPL token to SOL via Jupiter  
pub fn swap_token_to_sol<'info>(
    vault_token_account: &AccountInfo<'info>,
    _wsol_account: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    jupiter_program: &AccountInfo<'info>,
    _token_program: &AccountInfo<'info>,
    _swap_route: &SwapRoute,
    swap_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    token_account_bump: u8,
) -> Result<SwapResult> {
    // For Token -> SOL swaps:
    // 1. Execute Jupiter swap Token -> WSOL
    // 2. Close WSOL account to unwrap to SOL
    // Jupiter handles the unwrapping internally in most cases

    execute_jupiter_swap(
        vault_token_account,
        recipient,
        jupiter_program,
        swap_data,
        remaining_accounts,
        vault_key,
        token_account_bump,
    )
}

/// Swap between two SPL tokens via Jupiter
pub fn swap_token_to_token<'info>(
    vault_token_account: &AccountInfo<'info>,
    destination_token_account: &AccountInfo<'info>,
    jupiter_program: &AccountInfo<'info>,
    _swap_route: &SwapRoute,
    swap_data: Vec<u8>,
    remaining_accounts: &[AccountInfo<'info>],
    vault_key: &Pubkey,
    token_account_bump: u8,
) -> Result<SwapResult> {
    execute_jupiter_swap(
        vault_token_account,
        destination_token_account,
        jupiter_program,
        swap_data,
        remaining_accounts,
        vault_key,
        token_account_bump,
    )
}
