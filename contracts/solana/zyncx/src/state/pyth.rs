use anchor_lang::prelude::*;

// ============================================================================
// PYTH PRICE FEED INTEGRATION
// ============================================================================
// Pyth Network provides real-time price feeds that Arcium can use for
// confidential price comparisons without revealing user's trading bounds.
// ============================================================================

/// Pyth Oracle Program ID (Mainnet)
pub const PYTH_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x77, 0x1d, 0x89, 0xdc, 0x0e, 0x89, 0x4f, 0x27,
    0x68, 0xfe, 0x7c, 0x3b, 0x84, 0x89, 0x75, 0xa5,
    0x0c, 0x50, 0x25, 0x5d, 0x9b, 0x46, 0x6e, 0x6e,
    0x2f, 0x41, 0xc7, 0x8f, 0x00, 0x00, 0x00, 0x00,
]);

/// SOL/USD Price Feed ID (Pyth)
pub const SOL_USD_PRICE_FEED: [u8; 32] = [
    0xef, 0x0d, 0x8b, 0x6f, 0xda, 0x2c, 0xeb, 0xa4,
    0x1d, 0xa1, 0x5d, 0x40, 0x95, 0xd1, 0xda, 0x39,
    0x2a, 0x0d, 0x2f, 0x8e, 0xd0, 0xc6, 0xc7, 0xbc,
    0x0f, 0x4c, 0xfa, 0xc8, 0xc2, 0x80, 0xb5, 0x6d,
];

/// USDC/USD Price Feed ID (Pyth)
pub const USDC_USD_PRICE_FEED: [u8; 32] = [
    0xea, 0xa0, 0x20, 0xc6, 0x1c, 0xc4, 0x79, 0x71,
    0x2d, 0x82, 0xc1, 0x2b, 0x61, 0x1a, 0xd1, 0x3b,
    0xcc, 0x99, 0x4d, 0x36, 0xf0, 0xd5, 0x92, 0x2e,
    0xd4, 0x6c, 0x63, 0x6b, 0x67, 0x47, 0xdb, 0x63,
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
