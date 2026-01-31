use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_macros::comp_def_offset;

pub mod dex;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::{SwapParam, ConfidentialSwapParams};

declare_id!("HkwC2dfNAwgcsPtmMmwoD3AGXYMXJBwfY72WB9VMwDY5");

#[arcium_program]
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
    // ARCIUM MXE INTEGRATION (New Three-Instruction Pattern)
    // ========================================================================

    // --- Computation Definition Initializers (one-time setup) ---

    /// Initialize init_vault computation definition
    pub fn init_vault_comp_def(ctx: Context<InitVaultCompDef>) -> Result<()> {
        instructions::arcium_mxe::handler_init_vault_comp_def(ctx)
    }

    /// Initialize process_deposit computation definition
    pub fn init_deposit_comp_def(ctx: Context<InitDepositCompDef>) -> Result<()> {
        instructions::arcium_mxe::handler_init_deposit_comp_def(ctx)
    }

    /// Initialize confidential_swap computation definition
    pub fn init_swap_comp_def(ctx: Context<InitSwapCompDef>) -> Result<()> {
        instructions::arcium_mxe::handler_init_swap_comp_def(ctx)
    }

    /// Initialize compute_withdrawal computation definition
    pub fn init_withdrawal_comp_def(ctx: Context<InitWithdrawalCompDef>) -> Result<()> {
        instructions::arcium_mxe::handler_init_withdrawal_comp_def(ctx)
    }

    // --- Encrypted Vault Management ---

    /// Create an encrypted vault account for MXE state storage
    pub fn create_encrypted_vault(ctx: Context<CreateEncryptedVault>) -> Result<()> {
        instructions::arcium_mxe::handler_create_encrypted_vault(ctx)
    }

    // --- Queue Computations ---

    /// Queue an encrypted deposit to the MXE
    /// User's deposit amount is encrypted; MXE processes without revealing it
    pub fn queue_encrypted_deposit(
        ctx: Context<QueueEncryptedDeposit>,
        computation_offset: u64,
        params: EncryptedDepositParams,
    ) -> Result<()> {
        instructions::arcium_mxe::handler_queue_encrypted_deposit(ctx, computation_offset, params)
    }

    /// Queue a confidential swap to the MXE
    /// Trading bounds (min_out, slippage) are encrypted; execution is private
    pub fn queue_confidential_swap_mxe(
        ctx: Context<QueueConfidentialSwapMxe>,
        computation_offset: u64,
        params: ConfidentialSwapMxeParams,
    ) -> Result<()> {
        instructions::arcium_mxe::handler_queue_confidential_swap_mxe(ctx, computation_offset, params)
    }

    // --- Callbacks (called by MXE after computation) ---

    /// Callback for deposit computation
    pub fn deposit_callback(
        ctx: Context<DepositCallback>,
        output: SignedComputationOutputs<DepositOutput>,
    ) -> Result<()> {
        instructions::arcium_mxe::deposit_callback(ctx, output)
    }

    /// Callback for confidential swap computation
    pub fn confidential_swap_callback_mxe(
        ctx: Context<ConfidentialSwapCallbackMxe>,
        output: SignedComputationOutputs<SwapOutput>,
    ) -> Result<()> {
        instructions::arcium_mxe::confidential_swap_callback(ctx, output)
    }
}
