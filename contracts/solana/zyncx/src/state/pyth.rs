use anchor_lang::prelude::*;

// ============================================================================
// PYTH PRICE FEED INTEGRATION
// ============================================================================
// Pyth Network provides real-time price feeds that Arcium can use for
// confidential price comparisons without revealing user's trading bounds.
// ============================================================================

/// Pyth Oracle Program ID (Devnet)
/// Address: gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s
pub const PYTH_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x0b, 0x58, 0x7b, 0x8d, 0x5c, 0x3c, 0x89, 0x7a,
    0x4e, 0x2a, 0x1f, 0x6b, 0x8d, 0x9c, 0x7e, 0x3f,
    0x5a, 0x4b, 0x2c, 0x8d, 0x9e, 0x1f, 0x6a, 0x7b,
    0x3c, 0x5d, 0x8e, 0x9f, 0x2a, 0x4b, 0x6c, 0x7d,
]);

/// SOL/USD Price Feed Account (Devnet)
/// Address: J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix
pub const SOL_USD_PRICE_FEED: [u8; 32] = [
    0xfe, 0x65, 0x0f, 0x04, 0x93, 0xa1, 0x66, 0xc8,
    0x39, 0x10, 0x5b, 0xc8, 0x5c, 0x98, 0x7e, 0x74,
    0x87, 0x4e, 0x5c, 0x76, 0x76, 0x2d, 0x3e, 0x3f,
    0x62, 0x47, 0x5a, 0x7d, 0x63, 0x1a, 0x00, 0x3e,
];

/// USDC/USD Price Feed Account (Devnet)  
/// Address: 5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7
pub const USDC_USD_PRICE_FEED: [u8; 32] = [
    0x41, 0xf3, 0x62, 0x5f, 0x12, 0x30, 0x83, 0x5c,
    0x38, 0x78, 0x6b, 0x8d, 0x4f, 0x85, 0x6d, 0x39,
    0x7a, 0x97, 0x36, 0x25, 0x66, 0xf7, 0x3b, 0x67,
    0x7a, 0xa1, 0xeb, 0x7f, 0xd5, 0x8c, 0x75, 0x7a,
];

/// Price data from Pyth oracle
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PriceData {
    /// Price in fixed-point representation
    pub price: i64,
    /// Confidence interval
    pub confidence: u64,
    /// Exponent (price = price * 10^exponent)
    pub exponent: i32,
    /// Unix timestamp of the price
    pub publish_time: i64,
}

impl PriceData {
    /// Get price as u64 with specified decimals
    pub fn get_price_with_decimals(&self, decimals: u8) -> Option<u64> {
        if self.price < 0 {
            return None;
        }
        
        let price = self.price as u64;
        let exp = self.exponent;
        let target_exp = -(decimals as i32);
        
        if exp == target_exp {
            Some(price)
        } else if exp > target_exp {
            // Need to multiply
            let diff = (exp - target_exp) as u32;
            price.checked_mul(10u64.pow(diff))
        } else {
            // Need to divide
            let diff = (target_exp - exp) as u32;
            Some(price / 10u64.pow(diff))
        }
    }

    /// Check if price is stale (older than max_age seconds)
    pub fn is_stale(&self, max_age_seconds: i64) -> bool {
        let now = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        now - self.publish_time > max_age_seconds
    }
}

/// Cached price feed account for quick lookups
#[account]
pub struct CachedPriceFeed {
    /// Bump seed for PDA
    pub bump: u8,
    /// Token mint this price feed is for
    pub token_mint: Pubkey,
    /// Pyth price feed account
    pub pyth_feed: Pubkey,
    /// Cached price data
    pub price_data: PriceData,
    /// Last update timestamp
    pub last_updated: i64,
    /// Symbol (e.g., "SOL/USD")
    pub symbol: [u8; 16],
}

impl CachedPriceFeed {
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +   // bump
        32 +  // token_mint
        32 +  // pyth_feed
        8 +   // price (i64)
        8 +   // confidence
        4 +   // exponent
        8 +   // publish_time
        8 +   // last_updated
        16;   // symbol
}

/// Parameters for price comparison in Arcium
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceComparisonParams {
    /// Price feed to use
    pub price_feed: Pubkey,
    /// Encrypted price bound (FHE ciphertext)
    pub encrypted_bound: Vec<u8>,
    /// Comparison operator (encoded)
    /// 0 = greater than, 1 = less than, 2 = equal, 3 = greater or equal, 4 = less or equal
    pub operator: u8,
}

/// Parse Pyth price from account data
pub fn parse_pyth_price(data: &[u8]) -> Result<PriceData> {
    // Pyth price account structure (simplified)
    // Full structure: https://github.com/pyth-network/pyth-sdk-solana
    
    if data.len() < 48 {
        return Err(crate::errors::ZyncxError::InvalidPriceFeed.into());
    }

    // Skip magic number and version (8 bytes)
    // Price is at offset 208 in the full structure
    // For simplified parsing, we use a subset
    
    let price = i64::from_le_bytes(data[32..40].try_into().unwrap());
    let confidence = u64::from_le_bytes(data[40..48].try_into().unwrap());
    let exponent = i32::from_le_bytes(data[20..24].try_into().unwrap());
    let publish_time = i64::from_le_bytes(data[24..32].try_into().unwrap());

    Ok(PriceData {
        price,
        confidence,
        exponent,
        publish_time,
    })
}

/// Common token price feed mappings
pub mod price_feeds {
    use super::*;

    pub fn get_feed_for_token(mint: &Pubkey) -> Option<[u8; 32]> {
        // Native SOL (represented as zero pubkey in our system)
        if *mint == Pubkey::default() {
            return Some(SOL_USD_PRICE_FEED);
        }
        
        // Add more token mappings as needed
        // USDC, USDT, etc.
        
        None
    }
}
