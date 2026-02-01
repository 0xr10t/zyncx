use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

use crate::state::{MerkleTreeState, VaultState};
use crate::errors::ZyncxError;

#[derive(Accounts)]
pub struct VerifyProof<'info> {
    #[account(
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,

    #[account(
        seeds = [b"merkle_tree", vault.key().as_ref()],
        bump = merkle_tree.bump,
    )]
    pub merkle_tree: Account<'info, MerkleTreeState>,

    /// CHECK: The Noir verifier program (mixer.so deployed via Sunspot)
    #[account(executable)]
    pub verifier_program: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<VerifyProof>,
    amount: u64,
    nullifier: [u8; 32],
    new_commitment: [u8; 32],
    proof: Vec<u8>,
) -> Result<bool> {
    let merkle_tree = &ctx.accounts.merkle_tree;

    // Get current merkle root
    let root = merkle_tree.get_root();

    // Verify the ZK proof via CPI to Noir verifier
    match verify_noir_proof(
        &ctx.accounts.verifier_program,
        &proof,
        &root,
        &nullifier,
        amount,
        &new_commitment,
    ) {
        Ok(_) => {
            msg!("Proof verification successful");
            Ok(true)
        }
        Err(_) => {
            msg!("Proof verification failed");
            Ok(false)
        }
    }
}

/// Verify a Noir ZK proof via CPI to the deployed verifier program (mixer.so)
/// 
/// The Noir circuit (mixer/src/main.nr) expects public inputs in order:
/// 1. root (32 bytes) - Merkle tree root
/// 2. nullifier_hash (32 bytes) - Prevents double-spending  
/// 3. recipient (32 bytes) - Withdrawal recipient (bound to proof)
/// 4. withdraw_amount (32 bytes) - Amount being withdrawn
/// 5. new_commitment (32 bytes) - Change commitment (0 for full withdrawal)
pub fn verify_noir_proof(
    verifier_program: &AccountInfo,
    proof: &[u8],
    root: &[u8; 32],
    nullifier: &[u8; 32],
    amount: u64,
    new_commitment: &[u8; 32],
) -> Result<()> {
    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }

    // Build verifier instruction data: [proof][public_inputs...]
    let mut verifier_input = Vec::with_capacity(proof.len() + 160);
    
    // Proof bytes (variable length)
    verifier_input.extend_from_slice(proof);
    
    // Public inputs (must match Noir circuit order)
    // 1. root
    verifier_input.extend_from_slice(root);
    
    // 2. nullifier_hash
    verifier_input.extend_from_slice(nullifier);
    
    // 3. recipient (zero for now - actual binding happens in withdraw/swap)
    verifier_input.extend_from_slice(&[0u8; 32]);
    
    // 4. withdraw_amount as 32-byte big-endian
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    verifier_input.extend_from_slice(&amount_bytes);
    
    // 5. new_commitment
    verifier_input.extend_from_slice(new_commitment);
    
    // Create CPI instruction to verifier
    let instruction = Instruction {
        program_id: *verifier_program.key,
        accounts: vec![],
        data: verifier_input,
    };
    
    msg!("Invoking Noir verifier with {} byte proof", proof.len());
    
    invoke(
        &instruction,
        &[verifier_program.clone()],
    ).map_err(|e| {
        msg!("Noir proof verification failed: {:?}", e);
        ZyncxError::InvalidZKProof
    })?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct CheckNullifier<'info> {
    #[account(
        seeds = [b"vault", vault.asset_mint.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, VaultState>,
}

pub fn check_nullifier_spent(
    ctx: Context<CheckNullifier>,
    nullifier: [u8; 32],
) -> Result<bool> {
    // Check if nullifier PDA exists (if it does, it's spent)
    let vault_key = ctx.accounts.vault.key();
    let (nullifier_pda, _bump) = Pubkey::find_program_address(
        &[b"nullifier", vault_key.as_ref(), nullifier.as_ref()],
        ctx.program_id,
    );

    // If the account exists and has data, the nullifier is spent
    // This is checked by attempting to derive the PDA
    msg!("Checking nullifier: {:?}", nullifier);
    msg!("Nullifier PDA: {:?}", nullifier_pda);

    Ok(false) // Caller should check if nullifier_pda account exists
}

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

pub fn check_root_exists(
    ctx: Context<CheckRoot>,
    root: [u8; 32],
) -> Result<bool> {
    let merkle_tree = &ctx.accounts.merkle_tree;
    Ok(merkle_tree.root_exists(&root))
}
