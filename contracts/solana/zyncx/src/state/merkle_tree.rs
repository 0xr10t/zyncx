use anchor_lang::prelude::*;
use light_poseidon::{Poseidon, PoseidonBytesHasher};
use ark_bn254::Fr;

pub const MAX_DEPTH: u32 = 32;
pub const ROOT_HISTORY_SIZE: u32 = 100;

#[account]
pub struct MerkleTreeState {
    pub bump: u8,
    pub depth: u32,
    pub size: u64,
    pub current_root_index: u32,
    pub root: [u8; 32],
    pub roots: Vec<[u8; 32]>,
    pub leaves: Vec<[u8; 32]>,
}

impl MerkleTreeState {
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +  // bump
        4 +  // depth
        8 +  // size
        4 +  // current_root_index
        32 + // root
        (32 * 100) + // roots history
        4 + (32 * 1024); // leaves vec (initial capacity for ~1024 leaves)

    pub fn get_root(&self) -> [u8; 32] {
        self.root
    }

    pub fn get_depth(&self) -> u32 {
        self.depth
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn insert(&mut self, leaf: [u8; 32]) -> Result<[u8; 32]> {
        require!(self.depth < MAX_DEPTH, crate::errors::ZyncxError::MaxDepthReached);

        self.leaves.push(leaf);
        self.size += 1;

        let new_root = self.compute_root()?;
        self.root = new_root;

        self.current_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
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
            index = if index == 0 { ROOT_HISTORY_SIZE - 1 } else { index - 1 };
        }
        false
    }

    fn compute_root(&self) -> Result<[u8; 32]> {
        if self.leaves.is_empty() {
            return Ok([0u8; 32]);
        }

        let mut current_level: Vec<[u8; 32]> = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    poseidon_hash_two(&chunk[0], &chunk[1])?
                } else {
                    poseidon_hash_two(&chunk[0], &[0u8; 32])?
                };
                next_level.push(hash);
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
            self.depth = (64 - (size - 1).leading_zeros()) as u32;
        }
    }
}

pub fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> Result<[u8; 32]> {
    let mut hasher = Poseidon::<Fr>::new_circom(2)
        .map_err(|_| crate::errors::ZyncxError::PoseidonHashFailed)?;
    
    let result = hasher.hash_bytes_be(&[left.as_slice(), right.as_slice()])
        .map_err(|_| crate::errors::ZyncxError::PoseidonHashFailed)?;
    
    Ok(result)
}

pub fn poseidon_hash_commitment(amount: u64, precommitment: [u8; 32]) -> Result<[u8; 32]> {
    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    
    poseidon_hash_two(&amount_bytes, &precommitment)
}
