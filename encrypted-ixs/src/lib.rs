// ZYNCX Encrypted Instructions for Arcium MXE
// These Arcis circuits define confidential computations processed by Arcium nodes

use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    // ============================================================================
    // STRUCT DEFINITIONS (must use #[derive(Copy, Clone)])
    // ============================================================================

    // Input for confidential swap validation
    #[derive(Copy, Clone)]
    pub struct ConfidentialSwapInput {
        pub min_price: u64,
        pub max_slippage: u16,
        pub current_price: u64,
        pub amount: u64,
    }

    // Output for confidential swap
    #[derive(Copy, Clone)]
    pub struct ConfidentialSwapOutput {
        pub should_execute: bool,
        pub min_output: u64,
    }

    // Input for limit order validation
    #[derive(Copy, Clone)]
    pub struct LimitOrderInput {
        pub target_price: u64,
        pub is_buy: bool,
        pub current_price: u64,
        pub amount: u64,
    }

    // Output for limit order
    #[derive(Copy, Clone)]
    pub struct LimitOrderOutput {
        pub should_fill: bool,
        pub execution_price: u64,
    }

    // Input for DCA (Dollar Cost Averaging) validation
    #[derive(Copy, Clone)]
    pub struct DCAInput {
        pub max_price: u64,
        pub current_price: u64,
        pub amount_per_interval: u64,
        pub interval_index: u32,
        pub total_intervals: u32,
    }

    // Output for DCA execution
    #[derive(Copy, Clone)]
    pub struct DCAOutput {
        pub should_execute: bool,
        pub swap_amount: u64,
        pub remaining_intervals: u32,
    }

    // Input for balance verification
    #[derive(Copy, Clone)]
    pub struct BalanceCheckInput {
        pub encrypted_balance: u64,
        pub required_amount: u64,
    }

    // ============================================================================
    // HELPER FUNCTIONS (non-MPC, can be unit tested)
    // ============================================================================

    pub fn calculate_min_output(amount: u64, price: u64, slippage_bps: u16) -> u64 {
        let expected = (amount as u128 * price as u128) / 1_000_000_000;
        let factor = 10000 - slippage_bps as u128;
        (expected * factor / 10000) as u64
    }

    // ============================================================================
    // ENCRYPTED INSTRUCTIONS (require MPC)
    // ============================================================================

    // Validates swap conditions without revealing trading bounds
    // Returns encrypted result that only the user can decrypt
    #[instruction]
    pub fn validate_confidential_swap(
        input: Enc<Shared, ConfidentialSwapInput>,
        current_price: u64  // Plaintext from Pyth oracle
    ) -> Enc<Shared, ConfidentialSwapOutput> {
        let data = input.to_arcis();
        
        // Check if current price meets minimum price requirement
        let price_ok = current_price >= data.min_price;
        
        // Calculate minimum output with slippage protection
        let min_output = calculate_min_output(data.amount, current_price, data.max_slippage);
        
        let result = ConfidentialSwapOutput {
            should_execute: price_ok,
            min_output,
        };
        
        input.owner.from_arcis(result)
    }

    // Checks if limit order conditions are met
    #[instruction]
    pub fn check_limit_order(
        input: Enc<Shared, LimitOrderInput>,
        current_price: u64  // Plaintext market price
    ) -> (bool, u64) {
        let data = input.to_arcis();
        
        // For buy orders: execute if current price <= target price
        // For sell orders: execute if current price >= target price
        let should_fill = if data.is_buy {
            current_price <= data.target_price
        } else {
            current_price >= data.target_price
        };
        
        (should_fill.reveal(), current_price)
    }

    // Validates DCA execution conditions
    #[instruction]
    pub fn validate_dca_interval(
        input: Enc<Shared, DCAInput>,
        current_price: u64  // Plaintext market price
    ) -> (bool, u64, u32) {
        let data = input.to_arcis();
        
        // Check if we haven't exceeded total intervals
        let intervals_ok = data.interval_index < data.total_intervals;
        
        // Check if price is acceptable
        let price_ok = current_price <= data.max_price;
        
        let should_execute = intervals_ok && price_ok;
        let remaining = data.total_intervals - data.interval_index - 1;
        
        (should_execute.reveal(), data.amount_per_interval.reveal(), remaining.reveal())
    }

    // Checks if user has sufficient balance without revealing actual balance
    #[instruction]
    pub fn verify_sufficient_balance(
        input: Enc<Shared, BalanceCheckInput>,
        required: u64  // Plaintext required amount
    ) -> bool {
        let data = input.to_arcis();
        let has_sufficient = data.encrypted_balance >= required;
        has_sufficient.reveal()
    }
}
