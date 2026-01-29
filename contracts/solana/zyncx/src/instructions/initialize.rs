use anchor_lang::prelude::*;

use crate::state::{MerkleTreeState, VaultState, VaultType};

pub const NATIVE_MINT: Pubkey = Pubkey::new_from_array([0u8; 32]); // Represents SOL

#[derive(Accounts)]
#[instruction(asset_mint: Pubkey)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = VaultState::INIT_SPACE,
        seeds = [b"vault", asset_mint.as_ref()],
        bump
    )]
    pub vault: Box<Account<'info, VaultState>>,

    #[account(
        init,
        payer = authority,
        space = MerkleTreeState::INIT_SPACE,
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump
    )]
    pub merkle_tree: Box<Account<'info, MerkleTreeState>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVault>, asset_mint: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let merkle_tree = &mut ctx.accounts.merkle_tree;

    // Determine vault type based on asset
    let vault_type = if asset_mint == NATIVE_MINT {
        VaultType::Native
    } else {
        VaultType::Alternative
    };

    // Initialize vault state
    vault.bump = ctx.bumps.vault;
    vault.vault_type = vault_type;
    vault.asset_mint = asset_mint;
    vault.merkle_tree = merkle_tree.key();
    vault.nonce = 0;
    vault.authority = ctx.accounts.authority.key();
    vault.total_deposited = 0;

    // Initialize merkle tree state
    merkle_tree.bump = ctx.bumps.merkle_tree;
    merkle_tree.depth = 0;
    merkle_tree.size = 0;
    merkle_tree.current_root_index = 0;
    merkle_tree.root = [0u8; 32];
    merkle_tree.roots = [[0u8; 32]; crate::state::merkle_tree::ROOT_HISTORY_SIZE];
    merkle_tree.leaves = Vec::new();

    msg!("Vault initialized for asset: {:?}", asset_mint);
    msg!("Vault type: {:?}", vault_type as u8);

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeMultipleVaults<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
