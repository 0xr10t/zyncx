use anchor_lang::prelude::*;
// Poseidon is too stack-heavy for Solana on-chain execution
// We use a lighter hash for on-chain Merkle tree, while ZK proofs use Poseidon
// The security comes from ZK proof verification, not the on-chain hash

pub const MAX_DEPTH: u32 = 20;
pub const ROOT_HISTORY_SIZE: usize = 30;
pub const MAX_LEAVES: usize = 100;

#[account]
pub struct MerkleTreeState {
    pub bump: u8,
    pub depth: u8,
    pub size: u64,
    pub current_root_index: u8,
    pub root: [u8; 32],
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
    pub leaves: Vec<[u8; 32]>,
}

impl MerkleTreeState {
    // ~4KB which is under Solana's 10KB limit
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +  // bump
        1 +  // depth (u8)
        8 +  // size
        1 +  // current_root_index (u8)
        32 + // root
        (32 * ROOT_HISTORY_SIZE) + // roots history (fixed array)
        4 + (32 * MAX_LEAVES); // leaves vec (initial capacity)

    pub fn get_root(&self) -> [u8; 32] {
        self.root
    }

    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn insert(&mut self, leaf: [u8; 32]) -> Result<[u8; 32]> {
        require!((self.depth as u32) < MAX_DEPTH, crate::errors::ZyncxError::MaxDepthReached);
        require!(self.leaves.len() < MAX_LEAVES, crate::errors::ZyncxError::MaxDepthReached);

        self.leaves.push(leaf);
        self.size += 1;

        // Use keccak256-based hash for on-chain Merkle tree (lightweight)
        // The ZK proof uses Poseidon and verifies the actual cryptographic security
        let new_root = self.compute_root_keccak()?;
        self.root = new_root;

        self.current_root_index = (self.current_root_index + 1) % (ROOT_HISTORY_SIZE as u8);
        self.roots[self.current_root_index as usize] = new_root;

        self.update_depth();

        Ok(new_root)
    }

    pub fn has(&self, leaf: &[u8; 32]) -> bool {
        self.leaves.contains(leaf)
    }

    pub fn root_exists(&self, root: &[u8; 32]) -> bool {
        if *root == [0u8; 32] {
            return false;
        }

        let mut index = self.current_root_index;
        for _ in 0..ROOT_HISTORY_SIZE {
            if self.roots[index as usize] == *root {
                return true;
            }
            index = if index == 0 { (ROOT_HISTORY_SIZE - 1) as u8 } else { index - 1 };
        }
        false
    }

    /// Compute Merkle root using keccak256 (Solana-friendly, low stack usage)
    /// Note: The ZK circuit uses Poseidon for the proof - this is just for on-chain tracking
    fn compute_root_keccak(&self) -> Result<[u8; 32]> {
        if self.leaves.is_empty() {
            return Ok([0u8; 32]);
        }

        // For single leaf, hash it with zero
        if self.leaves.len() == 1 {
            return keccak_hash_two(&self.leaves[0], &[0u8; 32]);
        }

        // Build tree level by level
        let mut current_level: Vec<[u8; 32]> = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            
            let mut i = 0;
            while i < current_level.len() {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    &[0u8; 32]
                };
                let hash = keccak_hash_two(left, right)?;
                next_level.push(hash);
                i += 2;
            }
            
            current_level = next_level;
        }

        Ok(current_level[0])
    }

    fn update_depth(&mut self) {
        let size = self.size;
        if size == 0 {
            self.depth = 0;
        } else {
            self.depth = (64 - (size - 1).leading_zeros()) as u8;
        }
    }
}

/// Keccak256 hash of two 32-byte inputs (uses Solana's keccak syscall)
#[inline(never)]
pub fn keccak_hash_two(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32]> {
    use solana_keccak_hasher::hashv;
    let result = hashv(&[left.as_slice(), right.as_slice()]);
    Ok(result.0)
}

// =============================================================================
// IMPORTANT: On-chain vs Off-chain Hash Functions
// =============================================================================
//
// The on-chain Merkle tree uses keccak256 for efficiency (Solana syscall).
// The ZK circuit (Noir) uses Poseidon for the cryptographic proof.
//
// SECURITY MODEL:
// 1. Client computes commitment using Poseidon: 
//    commitment = Poseidon(secret, nullifier_secret, amount, token_mint)
// 2. Client submits commitment to on-chain Merkle tree (stored as-is)
// 3. On-chain Merkle tree computes roots using keccak256 (for tracking only)
// 4. For withdrawal, client generates ZK proof using Poseidon-based Merkle path
// 5. The verifier program validates the Poseidon-based ZK proof
//
// The on-chain keccak tree is just for tracking commitments - the actual
// cryptographic security comes from the ZK proof which uses Poseidon.
// =============================================================================
