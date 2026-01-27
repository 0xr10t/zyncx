use anchor_lang::prelude::*;
use light_poseidon::{Poseidon, PoseidonBytesHasher};
use ark_bn254::Fr;

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

        let new_root = self.compute_root()?;
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

    fn compute_root(&self) -> Result<[u8; 32]> {
        if self.leaves.is_empty() {
            return Ok([0u8; 32]);
        }

        // For single leaf, hash it with zero
        if self.leaves.len() == 1 {
            return simple_hash(&self.leaves[0], &[0u8; 32]);
        }

        // Use iterative approach with minimal stack usage
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
                let hash = simple_hash(left, right)?;
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

/// Simple keccak-like hash for merkle tree (uses less stack than Poseidon)
/// This is used internally for merkle tree computation to avoid stack overflow
#[inline(never)]
pub fn simple_hash(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32]> {
    use anchor_lang::solana_program::keccak;
    
    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(left);
    combined[32..].copy_from_slice(right);
    
    Ok(keccak::hash(&combined).to_bytes())
}

/// Poseidon hash for commitment generation (ZK-friendly)
#[inline(never)]
pub fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32]> {
    let mut hasher = Poseidon::<Fr>::new_circom(2)
        .map_err(|_| crate::errors::ZyncxError::PoseidonHashFailed)?;
    
    let result = hasher.hash_bytes_be(&[left.as_slice(), right.as_slice()])
        .map_err(|_| crate::errors::ZyncxError::PoseidonHashFailed)?;
    
    Ok(result)
}

/// Hash commitment using keccak (for testing - uses less stack)
/// In production with ZK proofs, use poseidon_hash_commitment_zk
#[inline(never)]
pub fn poseidon_hash_commitment(amount: u64, precommitment: [u8; 32]) -> Result<[u8; 32]> {
    use anchor_lang::solana_program::keccak;
    
    let mut data = [0u8; 40]; // 8 bytes for amount + 32 bytes for precommitment
    data[..8].copy_from_slice(&amount.to_le_bytes());
    data[8..].copy_from_slice(&precommitment);
    
    Ok(keccak::hash(&data).to_bytes())
}

/// Hash commitment using Poseidon (ZK-friendly, for production with real ZK proofs)
/// WARNING: This may cause stack overflow on Solana due to Poseidon's stack usage
#[inline(never)]
#[allow(dead_code)]
pub fn poseidon_hash_commitment_zk(amount: u64, precommitment: [u8; 32]) -> Result<[u8; 32]> {
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    
    poseidon_hash_two(&amount_bytes, &precommitment)
}
