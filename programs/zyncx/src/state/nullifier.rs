use anchor_lang::prelude::*;

#[account]
pub struct NullifierState {
    pub bump: u8,
    pub nullifier: [u8; 32],
    pub spent: bool,
    pub spent_at: i64,
    pub vault: Pubkey,
}

impl NullifierState {
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +  // bump
        32 + // nullifier
        1 +  // spent
        8 +  // spent_at
        32;  // vault
}
