use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

pub mod dex;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;
use state::{SwapParam, EncryptedVaultAccount};

// Computation definition offsets for Arcium MXE circuits
const COMP_DEF_OFFSET_INIT_VAULT: u32 = comp_def_offset("init_vault");
const COMP_DEF_OFFSET_PROCESS_DEPOSIT: u32 = comp_def_offset("process_deposit");
const COMP_DEF_OFFSET_CONFIDENTIAL_SWAP: u32 = comp_def_offset("confidential_swap");

declare_id!("7698BfsbJabinNT1jcmob9TxW7iD2gjtNCT4TbAkhyjH");

#[arcium_program]
pub mod zyncx {
    use super::*;

    // ========================================================================
    // PHASE 1: STANDARD VAULT OPERATIONS (ZK-SNARK based)
    // ========================================================================

    pub fn initialize_vault(ctx: Context<InitializeVault>, asset_mint: Pubkey) -> Result<()> {
        instructions::initialize::handler(ctx, asset_mint)
    }

    pub fn deposit_native(
        ctx: Context<DepositNative>,
        amount: u64,
        precommitment: [u8; 32],
    ) -> Result<[u8; 32]> {
        instructions::deposit::handler_native(ctx, amount, precommitment)
    }

    pub fn deposit_token(
        ctx: Context<DepositToken>,
        amount: u64,
        precommitment: [u8; 32],
    ) -> Result<[u8; 32]> {
        instructions::deposit::handler_token(ctx, amount, precommitment)
    }

    pub fn withdraw_native(
        ctx: Context<WithdrawNative>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::withdraw::handler_native(ctx, amount, nullifier, new_commitment, proof)
    }

    pub fn withdraw_token(
        ctx: Context<WithdrawToken>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::withdraw::handler_token(ctx, amount, nullifier, new_commitment, proof)
    }

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

    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        amount: u64,
        nullifier: [u8; 32],
        new_commitment: [u8; 32],
        proof: Vec<u8>,
    ) -> Result<bool> {
        instructions::verify::handler(ctx, amount, nullifier, new_commitment, proof)
    }

    pub fn check_root(ctx: Context<CheckRoot>, root: [u8; 32]) -> Result<bool> {
        instructions::verify::check_root_exists(ctx, root)
    }

    // ========================================================================
    // PHASE 2: ARCIUM MXE CONFIDENTIAL COMPUTATION
    // ========================================================================

    /// Initialize the init_vault computation definition
    pub fn init_vault_comp_def(ctx: Context<InitVaultCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// Initialize the process_deposit computation definition
    pub fn init_process_deposit_comp_def(ctx: Context<InitProcessDepositCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// Initialize the confidential_swap computation definition
    pub fn init_confidential_swap_comp_def(ctx: Context<InitConfidentialSwapCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// Create a new encrypted vault with Arcium MXE
    pub fn create_encrypted_vault(
        ctx: Context<CreateEncryptedVault>,
        computation_offset: u64,
        nonce: u128,
    ) -> Result<()> {
        msg!("Creating encrypted vault");

        ctx.accounts.vault.bump = ctx.bumps.vault;
        ctx.accounts.vault.token_mint = ctx.accounts.token_mint.key();
        ctx.accounts.vault.authority = ctx.accounts.payer.key();
        ctx.accounts.vault.nonce = nonce;
        ctx.accounts.vault.encrypted_state = [[0u8; 32]; 3];

        let args = ArgBuilder::new().plaintext_u128(nonce).build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![InitVaultCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.vault.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    /// Callback for init_vault computation
    #[arcium_callback(encrypted_ix = "init_vault")]
    pub fn init_vault_callback(
        ctx: Context<InitVaultCallback>,
        output: SignedComputationOutputs<InitVaultOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(InitVaultOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.vault.encrypted_state = o.ciphertexts;
        ctx.accounts.vault.nonce = o.nonce;

        emit!(VaultInitialized {
            vault: ctx.accounts.vault.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Queue an encrypted deposit via Arcium MXE
    pub fn queue_encrypted_deposit(
        ctx: Context<QueueEncryptedDeposit>,
        computation_offset: u64,
        deposit_amount: u64,
    ) -> Result<()> {
        msg!("Queueing encrypted deposit");

        let args = ArgBuilder::new()
            .plaintext_u64(deposit_amount)
            .plaintext_u128(ctx.accounts.vault.nonce)
            .account(
                ctx.accounts.vault.key(),
                8 + 1 + 32 + 32 + 16,
                32 * 3,
            )
            .build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ProcessDepositCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.vault.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        emit!(EncryptedDepositQueued {
            user: ctx.accounts.payer.key(),
            vault: ctx.accounts.vault.key(),
            computation_offset,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Callback for process_deposit computation
    #[arcium_callback(encrypted_ix = "process_deposit")]
    pub fn process_deposit_callback(
        ctx: Context<ProcessDepositCallback>,
        output: SignedComputationOutputs<ProcessDepositOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(ProcessDepositOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.vault.encrypted_state = o.ciphertexts;
        ctx.accounts.vault.nonce = o.nonce;

        emit!(DepositProcessed {
            vault: ctx.accounts.vault.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Queue a confidential swap via Arcium MXE
    pub fn queue_confidential_swap(
        ctx: Context<QueueConfidentialSwap>,
        computation_offset: u64,
        encrypted_min_out: [u8; 32],
        encryption_pubkey: [u8; 32],
        nonce: u128,
        current_output: u64,
    ) -> Result<()> {
        msg!("Queueing confidential swap");

        let args = ArgBuilder::new()
            .x25519_pubkey(encryption_pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_min_out)
            .plaintext_u64(current_output)
            .build();

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![ConfidentialSwapCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;

        emit!(ConfidentialSwapQueued {
            user: ctx.accounts.payer.key(),
            vault: ctx.accounts.vault.key(),
            computation_offset,
            current_output,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Callback for confidential_swap computation
    #[arcium_callback(encrypted_ix = "confidential_swap")]
    pub fn confidential_swap_callback(
        ctx: Context<ConfidentialSwapCallback>,
        output: SignedComputationOutputs<ConfidentialSwapOutput>,
    ) -> Result<()> {
        let should_execute = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(ConfidentialSwapOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        emit!(ConfidentialSwapResult {
            should_execute,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// ============================================================================
// ARCIUM COMPUTATION DEFINITION ACCOUNTS
// ============================================================================

#[init_computation_definition_accounts("init_vault", payer)]
#[derive(Accounts)]
pub struct InitVaultCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("process_deposit", payer)]
#[derive(Accounts)]
pub struct InitProcessDepositCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("confidential_swap", payer)]
#[derive(Accounts)]
pub struct InitConfidentialSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// ============================================================================
// QUEUE COMPUTATION ACCOUNTS
// ============================================================================

#[queue_computation_accounts("init_vault", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CreateEncryptedVault<'info> {
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
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VAULT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    /// CHECK: Token mint for the vault
    pub token_mint: AccountInfo<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + EncryptedVaultAccount::INIT_SPACE,
        seeds = [b"enc_vault", token_mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, EncryptedVaultAccount>,
}

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
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(mut)]
    pub vault: Account<'info, EncryptedVaultAccount>,
}

#[queue_computation_accounts("confidential_swap", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct QueueConfidentialSwap<'info> {
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
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: mempool_account
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: executing_pool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    #[account(mut)]
    pub vault: Account<'info, EncryptedVaultAccount>,
}

// ============================================================================
// CALLBACK ACCOUNTS
// ============================================================================

#[callback_accounts("init_vault")]
#[derive(Accounts)]
pub struct InitVaultCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VAULT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub vault: Account<'info, EncryptedVaultAccount>,
}

#[callback_accounts("process_deposit")]
#[derive(Accounts)]
pub struct ProcessDepositCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_PROCESS_DEPOSIT))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub vault: Account<'info, EncryptedVaultAccount>,
}

#[callback_accounts("confidential_swap")]
#[derive(Accounts)]
pub struct ConfidentialSwapCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    /// CHECK: computation_account
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

// ============================================================================
// ERROR CODES
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
    #[msg("Invalid authority")]
    InvalidAuthority,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EncryptedDepositQueued {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub computation_offset: u64,
    pub timestamp: i64,
}

#[event]
pub struct DepositProcessed {
    pub vault: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ConfidentialSwapQueued {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub computation_offset: u64,
    pub current_output: u64,
    pub timestamp: i64,
}

#[event]
pub struct ConfidentialSwapResult {
    pub should_execute: bool,
    pub timestamp: i64,
}
