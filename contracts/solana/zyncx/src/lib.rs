use anchor_lang::prelude::*;
// TODO: Re-enable when Arcium SDK is stabilized
// use arcium_anchor::prelude::*;

pub mod dex;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::{SwapParam, ConfidentialSwapParams};

declare_id!("HkwC2dfNAwgcsPtmMmwoD3AGXYMXJBwfY72WB9VMwDY5");

// TODO: Re-enable #[arcium_program] when Arcium SDK is stabilized
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

    /// Cross-token swap (SOL → USDC, etc.) with ZK proof
    /// Uses swap_circuit from Noir - nullifies in source vault, commits in destination vault
    pub fn cross_token_swap<'info>(
        ctx: Context<'_, '_, 'info, 'info, CrossTokenSwap<'info>>,
        swap_param: SwapParam,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::swap::handler_cross_token(ctx, swap_param, nullifier, new_commitment, proof, swap_data)
    }

    /// Check if a root exists in the merkle tree history
    /// Used by clients to verify their withdrawal proof will be accepted
    pub fn check_root(ctx: Context<CheckRoot>, root: [u8; 32]) -> Result<bool> {
        instructions::verify::check_root_exists(ctx, root)
    }

    // ========================================================================
    // PHASE 2: ARCIUM CONFIDENTIAL COMPUTATION (Legacy API)
    // ========================================================================

    /// Initialize Arcium MXE configuration (legacy)
    pub fn initialize_arcium_config(
        ctx: Context<InitializeArciumConfig>,
        mxe_address: Pubkey,
        computation_fee: u64,
        timeout_seconds: i64,
    ) -> Result<()> {
        instructions::confidential::handler_init_arcium_config(ctx, mxe_address, computation_fee, timeout_seconds)
    }

    /// Create a nullifier account for confidential operations
    pub fn create_nullifier(
        ctx: Context<CreateNullifier>,
        nullifier: [u8; 32],
    ) -> Result<()> {
        instructions::confidential::handler_create_nullifier(ctx, nullifier)
    }

    /// Queue a confidential swap to Arcium MXE (legacy)
    pub fn queue_confidential_swap(
        ctx: Context<QueueConfidentialSwap>,
        params: ConfidentialSwapParams,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::confidential::handler_queue_confidential_swap(ctx, params, proof)
    }

    /// Cancel an expired computation request
    pub fn cancel_computation(
        ctx: Context<CancelComputation>,
        request_id: u64,
    ) -> Result<()> {
        instructions::confidential::handler_cancel_computation(ctx, request_id)
    }

    // ========================================================================
    // ARCIUM MXE INTEGRATION (Three-Instruction Pattern)
    // ========================================================================
    // TODO: Re-enable after Arcium SDK stabilizes
    // These instructions provide encrypted trading strategy execution.
    // Commented out due to SDK compatibility issues.
    //
    // Available when re-enabled:
    // - init_vault_comp_def, init_deposit_comp_def, init_swap_comp_def
    // - create_encrypted_vault
    // - queue_encrypted_deposit, queue_confidential_swap_mxe
    // - deposit_callback, confidential_swap_callback_mxe
    // ========================================================================
}
