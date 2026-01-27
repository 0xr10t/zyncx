use anchor_lang::prelude::*;

/// Represents a swap route for DEX execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapRoute {
    /// Input token mint
    pub input_mint: Pubkey,
    /// Output token mint
    pub output_mint: Pubkey,
    /// Amount of input tokens to swap
    pub amount_in: u64,
    /// Minimum amount of output tokens expected (slippage protection)
    pub min_amount_out: u64,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage_bps: u16,
}

impl SwapRoute {
    pub const SIZE: usize = 32 + 32 + 8 + 8 + 2;
}

/// Result of a swap execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapResult {
    /// Actual amount of input tokens consumed
    pub amount_in: u64,
    /// Actual amount of output tokens received
    pub amount_out: u64,
    /// Fee paid (in input token)
    pub fee_amount: u64,
}

/// Supported DEX protocols
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DexProtocol {
    /// Jupiter Aggregator
    Jupiter,
    /// Raydium AMM
    Raydium,
    /// Orca Whirlpools
    Orca,
    /// Direct transfer (no swap, same token)
    Direct,
}

/// Native SOL mint address (all zeros represents SOL)
pub const NATIVE_SOL_MINT: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// WSOL mint address (So11111111111111111111111111111111111111112)
pub const WSOL_MINT: Pubkey = Pubkey::new_from_array([
    6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53,
    218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1
]);

/// Check if a mint represents native SOL
pub fn is_native_sol(mint: &Pubkey) -> bool {
    *mint == NATIVE_SOL_MINT || *mint == WSOL_MINT
}
