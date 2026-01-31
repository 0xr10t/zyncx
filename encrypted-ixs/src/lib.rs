// ============================================================================
// ZYNCX ARCIS MPC CIRCUITS
// ============================================================================
// Arcis is Arcium's Rust-based framework for writing MPC circuits that run 
// on encrypted data. These circuits compile to fixed circuit structures 
// before any data flows through.
//
// Key Types:
// - Enc<Shared, T>: Client + MXE can decrypt (user inputs/outputs)
// - Enc<Mxe, T>: Only MXE can decrypt (protocol state)
// - Plaintext: Public parameters visible to ARX nodes
// ============================================================================

use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    // ========================================================================
    // DATA STRUCTURES
    // ========================================================================

    /// Encrypted trading strategy bounds
    /// User encrypts these values - ARX nodes never see the plaintext
    #[derive(Copy, Clone)]
    pub struct SwapBounds {
        /// Minimum acceptable output amount (slippage protection)
        pub min_out: u64,
        /// Maximum price impact allowed (basis points)
        pub max_slippage_bps: u16,
        /// Whether to use aggressive execution
        pub aggressive: bool,
    }

    /// Encrypted limit order parameters
    #[derive(Copy, Clone)]
    pub struct LimitOrderParams {
        /// Target price (scaled by 1e9)
        pub target_price: u64,
        /// Amount to swap
        pub amount: u64,
        /// Direction: true = buy, false = sell
        pub is_buy: bool,
        /// Expiration timestamp
        pub expires_at: u64,
    }

    /// Vault state stored encrypted on-chain
    /// Only MXE can read/update this state
    #[derive(Copy, Clone)]
    pub struct VaultState {
        /// Total pending deposits awaiting processing
        pub pending_deposits: u64,
        /// Total liquidity in the vault
        pub total_liquidity: u64,
        /// Total deposited amount
        pub total_deposited: u64,
    }

    /// User position state stored encrypted on-chain
    #[derive(Copy, Clone)]
    pub struct UserPosition {
        /// Amount deposited by this user
        pub deposited: u64,
        /// User's LP share (scaled by 1e9)
        pub lp_share: u64,
    }

    /// Input for deposit operations
    #[derive(Copy, Clone)]
    pub struct DepositInput {
        /// Amount being deposited
        pub amount: u64,
    }

    /// Input for withdrawal computation
    #[derive(Copy, Clone)]
    pub struct WithdrawInput {
        /// LP share to redeem
        pub lp_share_to_redeem: u64,
    }

    /// Swap computation output
    #[derive(Copy, Clone)]
    pub struct SwapResult {
        /// Whether the swap should execute
        pub should_execute: bool,
        /// Computed minimum output amount
        pub min_amount_out: u64,
    }

    /// Encrypted swap input - contains the swap amount (hidden from everyone)
    #[derive(Copy, Clone)]
    pub struct SwapInput {
        /// Amount to swap (encrypted - only user and MXE can see)
        pub amount: u64,
    }

    /// DCA configuration
    #[derive(Copy, Clone)]
    pub struct DCAConfig {
        /// Amount to swap per interval
        pub amount_per_swap: u64,
        /// Number of swaps remaining
        pub swaps_remaining: u16,
        /// Minimum acceptable price
        pub min_price: u64,
    }

    /// Input for balance verification
    #[derive(Copy, Clone)]
    pub struct BalanceCheckInput {
        /// Encrypted balance to check
        pub encrypted_balance: u64,
        /// Required amount to verify against
        pub required_amount: u64,
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    /// Calculate minimum output with slippage protection
    pub fn calculate_min_output(amount: u64, price: u64, slippage_bps: u16) -> u64 {
        let expected = (amount as u128 * price as u128) / 1_000_000_000;
        let factor = 10000 - slippage_bps as u128;
        (expected * factor / 10000) as u64
    }

    // ========================================================================
    // MXE INITIALIZATION INSTRUCTIONS
    // ========================================================================

    /// Initialize a new vault with zeroed encrypted state
    /// Called once when setting up a new vault
    #[instruction]
    pub fn init_vault(mxe: Mxe) -> Enc<Mxe, VaultState> {
        let initial_state = VaultState {
            pending_deposits: 0,
            total_liquidity: 0,
            total_deposited: 0,
        };
        mxe.from_arcis(initial_state)
    }

    /// Initialize a new user position
    #[instruction]
    pub fn init_position(mxe: Mxe) -> Enc<Mxe, UserPosition> {
        let initial_position = UserPosition {
            deposited: 0,
            lp_share: 0,
        };
        mxe.from_arcis(initial_position)
    }

    // ========================================================================
    // DEPOSIT OPERATIONS
    // ========================================================================

    /// Process a deposit: update vault state and user position
    /// 
    /// - deposit_input: Client's encrypted deposit amount
    /// - vault_state: Current vault state (MXE-encrypted)
    /// - user_position: Current user position (MXE-encrypted)
    /// 
    /// Returns updated (VaultState, UserPosition)
    #[instruction]
    pub fn process_deposit(
        deposit_input: Enc<Shared, DepositInput>,
        vault_state: Enc<Mxe, VaultState>,
        user_position: Enc<Mxe, UserPosition>,
    ) -> (Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>) {
        // Decrypt inputs to secret shares
        let input = deposit_input.to_arcis();
        let mut vault = vault_state.to_arcis();
        let mut position = user_position.to_arcis();

        // Update vault state
        vault.pending_deposits = vault.pending_deposits + input.amount;
        vault.total_deposited = vault.total_deposited + input.amount;

        // Calculate LP share (proportional to total, 1e9 scale)
        // If first deposit, LP share = amount * 1e9
        // Otherwise, LP share = (amount * total_lp_shares) / total_deposited
        let lp_share = if vault.total_deposited == input.amount {
            input.amount * 1_000_000_000
        } else {
            // Simplified: equal share for now
            // In production: (input.amount * total_lp) / (vault.total_deposited - input.amount)
            input.amount * 1_000_000_000
        };

        // Update user position
        position.deposited = position.deposited + input.amount;
        position.lp_share = position.lp_share + lp_share;

        // Re-encrypt for MXE
        (
            vault_state.owner.from_arcis(vault),
            user_position.owner.from_arcis(position),
        )
    }

    // ========================================================================
    // CONFIDENTIAL SWAP OPERATIONS
    // ========================================================================

    /// Evaluate whether a swap should execute based on encrypted bounds
    /// 
    /// This is the core privacy primitive: the user's trading strategy
    /// (min_out, slippage) is never revealed to anyone - not even the
    /// execution nodes. The MXE computes the decision on encrypted values.
    /// 
    /// - swap_bounds: User's encrypted price bounds
    /// - current_price: Current market price (from oracle, plaintext)
    /// - expected_out: Expected output based on current price (plaintext)
    /// 
    /// Returns encrypted decision and computed min_out
    #[instruction]
    pub fn evaluate_swap(
        swap_bounds: Enc<Shared, SwapBounds>,
        current_price: u64,         // Plaintext from Pyth oracle
        expected_out: u64,          // Plaintext computed from price
    ) -> Enc<Shared, SwapResult> {
        let bounds = swap_bounds.to_arcis();

        // Calculate effective minimum with slippage
        // min_with_slippage = expected_out * (10000 - max_slippage_bps) / 10000
        let slippage_factor = 10000 - bounds.max_slippage_bps as u64;
        let min_with_slippage = (expected_out * slippage_factor) / 10000;

        // Use the more conservative (higher) of user's min_out or slippage-adjusted min
        let effective_min = if bounds.min_out > min_with_slippage {
            bounds.min_out
        } else {
            min_with_slippage
        };

        // Determine if swap should execute
        // Execute if expected output meets user's minimum requirements
        let should_execute = expected_out >= bounds.min_out;

        let result = SwapResult {
            should_execute,
            min_amount_out: effective_min,
        };

        // Re-encrypt result for the client
        swap_bounds.owner.from_arcis(result)
    }

    /// Confidential swap with vault state update
    /// 
    /// - swap_input: User's encrypted swap amount (HIDDEN from everyone except user + MXE)
    /// - swap_bounds: User's encrypted trading bounds
    /// - vault_state: Current vault state
    /// - user_position: User's position
    /// - current_price: Oracle price (plaintext)
    /// 
    /// Returns (should_execute, min_out, updated_vault, updated_position)
    #[instruction]
    pub fn confidential_swap(
        swap_input: Enc<Shared, SwapInput>,   // Encrypted swap amount
        swap_bounds: Enc<Shared, SwapBounds>,
        vault_state: Enc<Mxe, VaultState>,
        user_position: Enc<Mxe, UserPosition>,
        current_price: u64,         // Plaintext from oracle
    ) -> (Enc<Shared, SwapResult>, Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>) {
        let input = swap_input.to_arcis();
        let bounds = swap_bounds.to_arcis();
        let mut vault = vault_state.to_arcis();
        let mut position = user_position.to_arcis();

        // Get the encrypted swap amount
        let swap_amount = input.amount;

        // Calculate expected output
        let expected_out = (swap_amount * current_price) / 1_000_000_000;

        // Calculate effective minimum
        let slippage_factor = 10000 - bounds.max_slippage_bps as u64;
        let min_with_slippage = (expected_out * slippage_factor) / 10000;
        let effective_min = if bounds.min_out > min_with_slippage {
            bounds.min_out
        } else {
            min_with_slippage
        };

        let should_execute = expected_out >= bounds.min_out;

        // Update state only if swap executes
        if should_execute {
            // Reduce user's deposited amount
            if position.deposited >= swap_amount {
                position.deposited = position.deposited - swap_amount;
            }
            // Update vault liquidity
            if vault.total_deposited >= swap_amount {
                vault.total_deposited = vault.total_deposited - swap_amount;
            }
        }

        let result = SwapResult {
            should_execute,
            min_amount_out: effective_min,
        };

        (
            swap_input.owner.from_arcis(result),
            vault_state.owner.from_arcis(vault),
            user_position.owner.from_arcis(position),
        )
    }

    // ========================================================================
    // LIMIT ORDER OPERATIONS
    // ========================================================================

    /// Evaluate a limit order against current price
    /// Returns whether the order should trigger
    #[instruction]
    pub fn evaluate_limit_order(
        order: Enc<Shared, LimitOrderParams>,
        current_price: u64,         // Plaintext from oracle
        current_time: u64,          // Plaintext timestamp
    ) -> Enc<Shared, bool> {
        let params = order.to_arcis();

        // Check if order has expired
        let not_expired = current_time < params.expires_at;

        // Check if price condition is met
        let price_met = if params.is_buy {
            current_price <= params.target_price
        } else {
            current_price >= params.target_price
        };

        let should_trigger = not_expired && price_met;
        
        order.owner.from_arcis(should_trigger)
    }

    // ========================================================================
    // WITHDRAWAL OPERATIONS
    // ========================================================================

    /// Compute withdrawal amount based on user's LP share
    /// 
    /// - user_position: User's encrypted position
    /// - vault_state: Vault state for calculating redemption value
    /// - user_pubkey: User's X25519 pubkey for output encryption
    /// 
    /// Returns encrypted withdrawal amount that only the user can decrypt
    #[instruction]
    pub fn compute_withdrawal(
        user_position: Enc<Mxe, UserPosition>,
        vault_state: Enc<Mxe, VaultState>,
        user_pubkey: Shared,
    ) -> Enc<Shared, u64> {
        let position = user_position.to_arcis();
        let _vault = vault_state.to_arcis();

        // Calculate redemption amount based on LP share
        // withdrawal_amount = (lp_share * total_liquidity) / total_lp_shares
        // Simplified: return deposited amount directly
        let withdrawal_amount = position.deposited;

        // Encrypt for user
        user_pubkey.from_arcis(withdrawal_amount)
    }

    /// Clear a user's position after withdrawal
    /// 
    /// - user_position: User's position to clear
    /// - withdraw_amount: Amount being withdrawn (plaintext, validated)
    /// - vault_state: Vault state to update
    /// 
    /// Returns updated (UserPosition, VaultState)
    #[instruction]
    pub fn clear_position(
        user_position: Enc<Mxe, UserPosition>,
        withdraw_amount: u64,
        vault_state: Enc<Mxe, VaultState>,
    ) -> (Enc<Mxe, UserPosition>, Enc<Mxe, VaultState>) {
        let mut position = user_position.to_arcis();
        let mut vault = vault_state.to_arcis();

        // Clear user position
        if position.deposited >= withdraw_amount {
            position.deposited = position.deposited - withdraw_amount;
        } else {
            position.deposited = 0;
        }
        position.lp_share = 0;

        // Update vault
        if vault.total_deposited >= withdraw_amount {
            vault.total_deposited = vault.total_deposited - withdraw_amount;
        }

        (
            user_position.owner.from_arcis(position),
            vault_state.owner.from_arcis(vault),
        )
    }

    // ========================================================================
    // DCA (Dollar Cost Averaging) OPERATIONS
    // ========================================================================

    /// Process DCA swap - returns swap result only (config update handled separately)
    #[instruction]
    pub fn process_dca(
        dca_config: Enc<Shared, DCAConfig>,
        current_price: u64,
    ) -> Enc<Shared, SwapResult> {
        let config = dca_config.to_arcis();

        // Check if price is acceptable
        let price_ok = current_price >= config.min_price;
        let swaps_available = config.swaps_remaining > 0;
        let should_execute = price_ok && swaps_available;

        let result = SwapResult {
            should_execute,
            min_amount_out: if should_execute {
                (config.amount_per_swap * current_price) / 1_000_000_000
            } else {
                0
            },
        };

        dca_config.owner.from_arcis(result)
    }

    /// Update DCA config after successful swap
    #[instruction]
    pub fn update_dca_config(
        dca_config: Enc<Shared, DCAConfig>,
    ) -> Enc<Shared, DCAConfig> {
        let mut config = dca_config.to_arcis();
        
        // Decrement remaining swaps
        if config.swaps_remaining > 0 {
            config.swaps_remaining = config.swaps_remaining - 1;
        }

        dca_config.owner.from_arcis(config)
    }

    // ========================================================================
    // BALANCE VERIFICATION
    // ========================================================================

    /// Checks if user has sufficient balance without revealing actual balance
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
