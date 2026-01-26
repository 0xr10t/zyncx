use anchor_lang::prelude::*;

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

    // Prepare public inputs for verification
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    
    let public_inputs = [
        amount_bytes,
        root,
        new_commitment,
        nullifier,
    ];

    // Verify the ZK proof
    match verify_groth16_proof(&proof, &public_inputs) {
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

#[allow(unused_variables)]
fn verify_groth16_proof(proof: &[u8], public_inputs: &[[u8; 32]; 4]) -> Result<()> {
    // TODO: Integrate with groth16-solana crate for actual verification
    // 
    // The verification key would be stored on-chain or embedded in the program
    // Public inputs: [amount, merkle_root, new_commitment, nullifier]
    //
    // Example structure for groth16-solana:
    // ```
    // use groth16_solana::groth16::Groth16Verifier;
    // 
    // let vk = get_verification_key();
    // let proof_a = &proof[0..64];
    // let proof_b = &proof[64..192];
    // let proof_c = &proof[192..256];
    // 
    // let mut verifier = Groth16Verifier::new(
    //     proof_a,
    //     proof_b,
    //     proof_c,
    //     public_inputs,
    //     &vk,
    // )?;
    // 
    // verifier.verify()?;
    // ```

    if proof.is_empty() {
        return Err(ZyncxError::InvalidZKProof.into());
    }

    // Placeholder verification
    msg!("ZK Proof verification placeholder");
    
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
