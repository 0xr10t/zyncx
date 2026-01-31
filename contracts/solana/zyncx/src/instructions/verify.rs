use anchor_lang::prelude::*;

use crate::state::{MerkleTreeState, VaultState};

// ============================================================================
// MERKLE ROOT VERIFICATION
// ============================================================================
//
// This module provides utilities to check Merkle tree root history.
// ZK proof verification is handled by the Sunspot Noir verifier program
// via CPI calls, so Groth16 verification code has been removed.
//
// ============================================================================

#[derive(Accounts)]
pub struct CheckRoot<'info> {
    #[account(
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Account<'info, MerkleTreeState>,

    #[account(
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,
}

/// Check if a Merkle root exists in the tree's root history.
/// This is used during withdrawals to verify the user's commitment
/// was part of the tree at some point (even if new deposits have been added since).
pub fn check_root_exists(
    ctx: Context<CheckRoot>,
    root: [u8; 32],
) -> Result<bool> {
    let merkle_tree = &ctx.accounts.merkle_tree;
    Ok(merkle_tree.root_exists(&root))
}
