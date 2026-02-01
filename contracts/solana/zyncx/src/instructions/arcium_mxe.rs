use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

use crate::state::EncryptedVaultAccount;

// ============================================================================
// ARCIUM MXE INTEGRATION (Full SDK Implementation)
// ============================================================================
// This module provides full Arcium SDK integration for confidential DeFi
// operations using Multi-party eXecution Environment (MXE).
//
// Pattern: init_comp_def → queue_computation → callback
// ============================================================================

// Computation definition offsets - derived from circuit names
pub const COMP_DEF_OFFSET_PROCESS_DEPOSIT: u32 = comp_def_offset("process_deposit");
pub const COMP_DEF_OFFSET_CONFIDENTIAL_SWAP: u32 = comp_def_offset("confidential_swap");
pub const COMP_DEF_OFFSET_COMPUTE_WITHDRAWAL: u32 = comp_def_offset("compute_withdrawal");

// ============================================================================
// ERROR CODES
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}

// ============================================================================
// 1. INIT COMPUTATION DEFINITIONS (one-time setup)
// ============================================================================

/// Initialize process_deposit computation definition
#[init_computation_definition_accounts("process_deposit", payer)]
#[derive(Accounts)]
pub struct InitProcessDepositCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

/// Initialize confidential_swap computation definition
#[init_computation_definition_accounts("confidential_swap", payer)]
#[derive(Accounts)]
pub struct InitConfidentialSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

/// Initialize compute_withdrawal computation definition
#[init_computation_definition_accounts("compute_withdrawal", payer)]
#[derive(Accounts)]
pub struct InitComputeWithdrawalCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// 2. QUEUE COMPUTATION INSTRUCTIONS
// ============================================================================

/// Parameters for encrypted deposit
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EncryptedDepositParams {
    /// Client's X25519 public key for encryption
    pub encryption_pubkey: [u8; 32],
    /// Nonce for the encrypted amount
    pub amount_nonce: u128,
    /// Encrypted deposit amount (ciphertext)
    pub encrypted_amount: [u8; 32],
}

/// Queue an encrypted deposit computation
#[queue_computation_accounts("process_deposit", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct QueueEncryptedDeposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(
        mut,
        address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,

    pub arcium_program: Program<'info, Arcium>,

    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,

    pub system_program: Program<'info, System>,
}

/// Parameters for confidential swap
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConfidentialSwapMxeParams {
    /// Client's X25519 public key
    pub encryption_pubkey: [u8; 32],
    /// Nonce for encrypted amount
    pub amount_nonce: u128,
    /// Encrypted swap amount (ciphertext) - HIDDEN from everyone!
    pub encrypted_amount: [u8; 32],
    /// Nonce for encrypted bounds
    pub bounds_nonce: u128,
    /// Encrypted minimum output
    pub encrypted_min_out: [u8; 32],
    /// Encrypted max slippage bps
    pub encrypted_max_slippage: [u8; 32],
    /// Current price from oracle (plaintext)
    pub current_price: u64,
    /// Nullifier from ZK proof
    pub nullifier: [u8; 32],
    /// New commitment for Merkle tree
    pub new_commitment: [u8; 32],
    /// ZK proof bytes
    pub proof: Vec<u8>,
}

/// Queue a confidential swap computation
#[queue_computation_accounts("confidential_swap", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct QueueConfidentialSwapMxe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,

    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(
        mut,
        address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,

    pub arcium_program: Program<'info, Arcium>,

    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,

    pub system_program: Program<'info, System>,
}

// ============================================================================
// 3. CALLBACK INSTRUCTIONS
// ============================================================================

/// Callback for process_deposit computation
#[callback_accounts("process_deposit")]
#[derive(Accounts)]
pub struct ProcessDepositCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: computation_account, checked by arcium program via constraints in the callback context.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
}

/// Callback for confidential_swap computation
#[callback_accounts("confidential_swap")]
#[derive(Accounts)]
pub struct ConfidentialSwapCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: computation_account, checked by arcium program via constraints in the callback context.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
}

/// Callback for compute_withdrawal computation
#[callback_accounts("compute_withdrawal")]
#[derive(Accounts)]
pub struct ComputeWithdrawalCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_COMPUTE_WITHDRAWAL)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,

    /// CHECK: computation_account, checked by arcium program via constraints in the callback context.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    #[account(mut)]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
}

// ============================================================================
// HELPER ACCOUNTS
// ============================================================================

/// Create encrypted vault account
#[derive(Accounts)]
pub struct CreateEncryptedVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + EncryptedVaultAccount::INIT_SPACE,
        seeds = [b"enc_vault", token_mint.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, EncryptedVaultAccount>>,
    
    /// CHECK: Token mint
    pub token_mint: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct EncryptedDepositQueued {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub computation_offset: u64,
    pub queued_at: i64,
}

#[event]
pub struct ConfidentialSwapQueuedMxe {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub computation_offset: u64,
    pub current_price: u64,
    pub queued_at: i64,
}

#[event]
pub struct DepositCallbackCompleted {
    pub vault: Pubkey,
    pub completed_at: i64,
}

#[event]
pub struct SwapCallbackCompleted {
    pub vault: Pubkey,
    pub completed_at: i64,
}
