use anchor_lang::prelude::*;

// ============================================================================
// ARCIUM MXE (Multi-party eXecution Environment) STATE
// ============================================================================
// Arcium enables confidential computation on encrypted data using MPC/FHE.
// The protocol follows a Call-and-Callback pattern:
// 1. User queues a computation with encrypted strategy
// 2. Arcium nodes process in the "Fortress" (MXE) without seeing plaintext
// 3. Arcium calls back with the encrypted result
// ============================================================================

/// Arcium MXE Program ID (placeholder - replace with actual deployed address)
pub const ARCIUM_MXE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x0a, 0x0c, 0x01, 0x0d, 0x0e, 0x0f, 0x10, 0x11,
    0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
    0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21,
    0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
]);

/// Computation status in the Arcium MXE
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComputationStatus {
    /// Computation queued, waiting for Arcium nodes
    Pending,
    /// Computation is being processed by Arx nodes
    Processing,
    /// Computation completed successfully
    Completed,
    /// Computation failed
    Failed,
    /// Computation expired (timeout)
    Expired,
}

impl Default for ComputationStatus {
    fn default() -> Self {
        ComputationStatus::Pending
    }
}

/// Type of confidential computation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComputationType {
    /// Private swap with hidden slippage/price bounds
    ConfidentialSwap,
    /// Private limit order
    ConfidentialLimitOrder,
    /// Private DCA (Dollar Cost Averaging)
    ConfidentialDCA,
    /// Custom computation
    Custom,
}

impl Default for ComputationType {
    fn default() -> Self {
        ComputationType::ConfidentialSwap
    }
}

/// Encrypted trading strategy sent to Arcium
/// The ciphertext contains FHE-encrypted bounds that only Arcium nodes can process
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EncryptedStrategy {
    /// FHE ciphertext containing the trading parameters
    /// For swaps: encrypted min_price, max_slippage, etc.
    pub ciphertext: Vec<u8>,
    /// Nonce used for encryption (for replay protection)
    pub nonce: [u8; 12],
    /// Public key used for encryption (Arcium cluster key)
    pub encryption_pubkey: [u8; 32],
}

impl EncryptedStrategy {
    pub const MAX_CIPHERTEXT_SIZE: usize = 512;
    
    pub fn size(&self) -> usize {
        4 + self.ciphertext.len() + 12 + 32
    }
}

/// State account tracking a queued Arcium computation
#[account]
pub struct ComputationRequest {
    /// Bump seed for PDA
    pub bump: u8,
    /// Unique request ID
    pub request_id: u64,
    /// User who initiated the request
    pub user: Pubkey,
    /// Vault associated with this computation
    pub vault: Pubkey,
    /// Type of computation
    pub computation_type: ComputationType,
    /// Current status
    pub status: ComputationStatus,
    /// Encrypted strategy (FHE ciphertext)
    pub encrypted_strategy: Vec<u8>,
    /// Callback instruction name
    pub callback_instruction: [u8; 32],
    /// Amount involved in the computation
    pub amount: u64,
    /// Source token mint
    pub src_token: Pubkey,
    /// Destination token mint  
    pub dst_token: Pubkey,
    /// Nullifier for the privacy proof
    pub nullifier: [u8; 32],
    /// New commitment after operation
    pub new_commitment: [u8; 32],
    /// Timestamp when queued
    pub queued_at: i64,
    /// Timestamp when completed (0 if not completed)
    pub completed_at: i64,
    /// Result from Arcium (encrypted or status code)
    pub result: Vec<u8>,
    /// Expiry timestamp
    pub expires_at: i64,
}

impl ComputationRequest {
    pub const BASE_SPACE: usize = 8 + // discriminator
        1 +   // bump
        8 +   // request_id
        32 +  // user
        32 +  // vault
        1 +   // computation_type
        1 +   // status
        4 +   // encrypted_strategy vec prefix
        32 +  // callback_instruction
        8 +   // amount
        32 +  // src_token
        32 +  // dst_token
        32 +  // nullifier
        32 +  // new_commitment
        8 +   // queued_at
        8 +   // completed_at
        4 +   // result vec prefix
        8;    // expires_at

    pub fn space_with_strategy(strategy_size: usize, result_size: usize) -> usize {
        Self::BASE_SPACE + strategy_size + result_size
    }

    // Reduced max space to fit stack constraints (256 + 64 instead of 512 + 256)
    pub const MAX_SPACE: usize = Self::BASE_SPACE + 256 + 64;
}

/// Global state for Arcium integration
#[account]
pub struct ArciumConfig {
    /// Bump seed for PDA
    pub bump: u8,
    /// Authority that can update config
    pub authority: Pubkey,
    /// Arcium MXE cluster address
    pub mxe_address: Pubkey,
    /// Fee for using confidential computation (in lamports)
    pub computation_fee: u64,
    /// Request counter for unique IDs
    pub request_counter: u64,
    /// Computation timeout in seconds
    pub timeout_seconds: i64,
    /// Whether confidential swaps are enabled
    pub swaps_enabled: bool,
    /// Whether confidential limit orders are enabled
    pub limit_orders_enabled: bool,
    /// Minimum amount for confidential operations
    pub min_amount: u64,
    /// Maximum amount for confidential operations
    pub max_amount: u64,
}

impl ArciumConfig {
    pub const INIT_SPACE: usize = 8 + // discriminator
        1 +   // bump
        32 +  // authority
        32 +  // mxe_address
        8 +   // computation_fee
        8 +   // request_counter
        8 +   // timeout_seconds
        1 +   // swaps_enabled
        1 +   // limit_orders_enabled
        8 +   // min_amount
        8;    // max_amount

    pub fn next_request_id(&mut self) -> u64 {
        let id = self.request_counter;
        self.request_counter += 1;
        id
    }
}

/// Parameters for a confidential swap request
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConfidentialSwapParams {
    /// Source token mint (what user is selling)
    pub src_token: Pubkey,
    /// Destination token mint (what user is buying)
    pub dst_token: Pubkey,
    /// Amount to swap (from shielded balance)
    pub amount: u64,
    /// Encrypted trading bounds (FHE ciphertext)
    /// Contains: min_price, max_slippage, deadline
    pub encrypted_bounds: Vec<u8>,
    /// Recipient of swapped tokens (can be shielded)
    pub recipient: Pubkey,
    /// Nullifier for this operation
    pub nullifier: [u8; 32],
    /// New commitment after operation
    pub new_commitment: [u8; 32],
}

impl ConfidentialSwapParams {
    pub const MAX_SIZE: usize = 32 + 32 + 8 + 4 + 256 + 32;
}

/// Result returned by Arcium after computation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComputationResult {
    /// Whether the computation succeeded
    pub success: bool,
    /// Status code (0 = success, others = error codes)
    pub status_code: u8,
    /// Encrypted result data
    pub encrypted_result: Vec<u8>,
    /// Signature from Arcium nodes (threshold signature)
    pub node_signature: [u8; 64],
    /// Timestamp of computation
    pub computed_at: i64,
}

impl ComputationResult {
    pub const BASE_SIZE: usize = 1 + 1 + 4 + 64 + 8;
}
