use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::SwapParam;

declare_id!("6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6");

#[program]
pub mod zyncx {
    use super::*;

    /// Initialize a new vault for a specific asset (SOL or SPL token)
    pub fn initialize_vault(ctx: Context<InitializeVault>, asset_mint: Pubkey) -> Result<()> {
        instructions::initialize::handler(ctx, asset_mint)
    }

    /// Deposit native SOL into the vault
    pub fn deposit_native(
        ctx: Context<DepositNative>,
        amount: u64,
        precommitment: [u8; 32],
    ) -> Result<[u8; 32]> {
        instructions::deposit::handler_native(ctx, amount, precommitment)
    }

    /// Deposit SPL tokens into the vault
    pub fn deposit_token(
        ctx: Context<DepositToken>,
        amount: u64,
        precommitment: [u8; 32],
    ) -> Result<[u8; 32]> {
        instructions::deposit::handler_token(ctx, amount, precommitment)
    }

    /// Withdraw native SOL from the vault with ZK proof
    pub fn withdraw_native(
        ctx: Context<WithdrawNative>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::withdraw::handler_native(ctx, amount, nullifier, new_commitment, proof)
    }

    /// Withdraw SPL tokens from the vault with ZK proof
    pub fn withdraw_token(
        ctx: Context<WithdrawToken>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::withdraw::handler_token(ctx, amount, nullifier, new_commitment, proof)
    }

    /// Swap native SOL via DEX with ZK proof
    pub fn swap_native(
        ctx: Context<SwapNative>,
        swap_param: SwapParam,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::swap::handler_native(ctx, swap_param, nullifier, new_commitment, proof)
    }

    /// Swap SPL tokens via DEX with ZK proof
    pub fn swap_token(
        ctx: Context<SwapToken>,
        swap_param: SwapParam,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::swap::handler_token(ctx, swap_param, nullifier, new_commitment, proof)
    }

    /// Verify a ZK proof without executing withdrawal
    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<bool> {
        instructions::verify::handler(ctx, amount, nullifier, new_commitment, proof)
    }

    /// Check if a root exists in the merkle tree history
    pub fn check_root(ctx: Context<CheckRoot>, root: [u8; 32]) -> Result<bool> {
        instructions::verify::check_root_exists(ctx, root)
    }
}
