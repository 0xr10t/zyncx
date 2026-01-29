use anchor_lang::prelude::*;

pub mod dex;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::{SwapParam, ConfidentialSwapParams};

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

    /// Swap native SOL via DEX (Jupiter) with ZK proof
    pub fn swap_native<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapNative<'info>>,
        swap_param: SwapParam,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::swap::handler_native(ctx, swap_param, nullifier, new_commitment, proof, swap_data)
    }

    /// Swap SPL tokens via DEX (Jupiter) with ZK proof
    pub fn swap_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapToken<'info>>,
        swap_param: SwapParam,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::swap::handler_token(ctx, swap_param, nullifier, new_commitment, proof, swap_data)
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

    // ========================================================================
    // PHASE 2: ARCIUM CONFIDENTIAL COMPUTATION
    // ========================================================================

    /// Initialize Arcium MXE configuration
    pub fn initialize_arcium_config(
        ctx: Context<InitializeArciumConfig>,
        mxe_address: Pubkey,
        computation_fee: u64,
        timeout_seconds: i64,
    ) -> Result<()> {
        instructions::confidential::handler_init_arcium_config(ctx, mxe_address, computation_fee, timeout_seconds)
    }

    /// Create a nullifier account for confidential operations
    /// Must be called before queue_confidential_swap
    pub fn create_nullifier(
        ctx: Context<CreateNullifier>,
        nullifier: [u8; 32],
    ) -> Result<()> {
        instructions::confidential::handler_create_nullifier(ctx, nullifier)
    }

    /// Queue a confidential swap to Arcium MXE
    /// User sends encrypted trading bounds; Arcium processes without seeing plaintext
    /// Note: Nullifier account must already exist (prevents stack overflow)
    pub fn queue_confidential_swap(
        ctx: Context<QueueConfidentialSwap>,
        params: ConfidentialSwapParams,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::confidential::handler_queue_confidential_swap(ctx, params, proof)
    }

    /// Callback from Arcium MXE after confidential computation completes
    /// Only callable by Arcium MXE program
    pub fn confidential_swap_callback<'info>(
        ctx: Context<'_, '_, 'info, 'info, ConfidentialSwapCallback<'info>>,
        request_id: u64,
        computation_success: bool,
        encrypted_result: Vec<u8>,
        node_signature: [u8; 64],
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::confidential::handler_confidential_swap_callback(
            ctx, request_id, computation_success, encrypted_result, node_signature, swap_data
        )
    }

    /// Cancel an expired computation request
    pub fn cancel_computation(
        ctx: Context<CancelComputation>,
        request_id: u64,
    ) -> Result<()> {
        instructions::confidential::handler_cancel_computation(ctx, request_id)
    }
}
