use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VaultType {
    Native,      // SOL
    Alternative, // SPL Token
}

#[account]
pub struct VaultState {
    pub bump: u8,
    pub vault_type: VaultType,
    pub asset_mint: Pubkey,
    pub merkle_tree: Pubkey,
    pub nonce: u64,
    pub authority: Pubkey,
    pub total_deposited: u64,
}

impl VaultState {
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +  // bump
        1 +  // vault_type
        32 + // asset_mint
        32 + // merkle_tree
        8 +  // nonce
        32 + // authority
        8;   // total_deposited
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SwapParam {
    pub src_token: Pubkey,
    pub dst_token: Pubkey,
    pub recipient: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub fee: u32, // basis points (1e-4)
}

impl SwapParam {
    pub const SIZE: usize = 32 + 32 + 32 + 8 + 8 + 4;
}
