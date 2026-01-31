use anchor_lang::prelude::*;

// ============================================================================
// ARCIUM MXE ENCRYPTED STATE ACCOUNTS
// ============================================================================
// Following the Arcium three-instruction pattern:
// 1. init_comp_def: Initialize computation definition (one-time)
// 2. queue_computation: Queue encrypted computation to MXE
// 3. callback: Receive MPC computation results
//
// Encrypted fields use [[u8; 32]; N] format - each u64/u128 becomes a 32-byte
// ciphertext after encryption. Nonce (u128) is stored for re-encryption.
// ============================================================================

/// Encrypted vault state - stores MXE-encrypted aggregate data
/// 
/// Memory layout:
/// [0..8]     Anchor discriminator
/// [8]        bump (1 byte)
/// [9..41]    authority (Pubkey, 32 bytes)
/// [41..73]   token_mint (Pubkey, 32 bytes)
/// [73..169]  vault_state (3 × 32 bytes = 96 bytes encrypted state)
/// [169..185] nonce (u128, 16 bytes)
/// [185..193] meta_nonce (u64, 8 bytes)
/// [193..201] created_at (i64, 8 bytes)
#[account]
#[derive(InitSpace)]
pub struct EncryptedVaultAccount {
    /// PDA bump seed
    pub bump: u8,
    /// Vault authority
    pub authority: Pubkey,
    /// Token mint this vault manages
    pub token_mint: Pubkey,
    
    /// Encrypted vault state: [pending_deposits, total_liquidity, total_deposited]
    /// Each is an Enc<Mxe, u64> - 32 bytes per ciphertext
    #[max_len(3)]
    pub vault_state: [[u8; 32]; 3],
    
    /// Nonce for MXE re-encryption (updated by every callback)
    pub nonce: u128,
    
    /// Application-level nonce for replay protection
    pub meta_nonce: u64,
    
    /// Timestamp when vault was created
    pub created_at: i64,
}

impl EncryptedVaultAccount {
    /// Byte offset to encrypted state (for ArgBuilder .account())
    /// = 8 (discriminator) + 1 (bump) + 32 (authority) + 32 (token_mint)
    pub const ENCRYPTED_STATE_OFFSET: usize = 8 + 1 + 32 + 32;
    
    /// Size of encrypted state in bytes (3 ciphertexts × 32 bytes)
    pub const ENCRYPTED_STATE_SIZE: usize = 32 * 3;
}

/// Encrypted user position - stores MXE-encrypted user-specific data
/// 
/// Memory layout:
/// [0..8]     Anchor discriminator
/// [8]        bump (1 byte)
/// [9..41]    owner (Pubkey, 32 bytes)
/// [41..73]   vault (Pubkey, 32 bytes)
/// [73..137]  position_state (2 × 32 bytes = 64 bytes encrypted state)
/// [137..153] nonce (u128, 16 bytes)
/// [153..161] created_at (i64, 8 bytes)
/// [161]      is_active (bool, 1 byte)
#[account]
#[derive(InitSpace)]
pub struct EncryptedUserPosition {
    /// PDA bump seed
    pub bump: u8,
    /// User who owns this position
    pub owner: Pubkey,
    /// Vault this position is for
    pub vault: Pubkey,
    
    /// Encrypted position state: [deposited_amount, lp_share]
    /// Each is an Enc<Mxe, u64> - 32 bytes per ciphertext
    #[max_len(2)]
    pub position_state: [[u8; 32]; 2],
    
    /// Nonce for MXE re-encryption
    pub nonce: u128,
    
    /// Timestamp when position was created
    pub created_at: i64,
    
    /// Whether this position is active
    pub is_active: bool,
}

impl EncryptedUserPosition {
    /// Byte offset to encrypted state
    /// = 8 (discriminator) + 1 (bump) + 32 (owner) + 32 (vault)
    pub const ENCRYPTED_STATE_OFFSET: usize = 8 + 1 + 32 + 32;
    
    /// Size of encrypted state in bytes (2 ciphertexts × 32 bytes)
    pub const ENCRYPTED_STATE_SIZE: usize = 32 * 2;
}

/// Encrypted swap request - queued computation waiting for MPC execution
#[account]
#[derive(InitSpace)]
pub struct EncryptedSwapRequest {
    /// PDA bump seed
    pub bump: u8,
    /// User who initiated the swap
    pub user: Pubkey,
    /// Source vault (what user is selling from)
    pub source_vault: Pubkey,
    /// Destination vault (what user is buying into)
    pub dest_vault: Pubkey,
    /// Computation offset (unique identifier)
    pub computation_offset: u64,
    
    /// Encrypted swap bounds: [min_out, max_slippage_bps, aggressive_flag]
    /// This is the user's encrypted trading strategy
    #[max_len(3)]
    pub encrypted_bounds: [[u8; 32]; 3],
    
    /// Nonce used for client encryption
    pub bounds_nonce: u128,
    
    /// Client's X25519 public key for result encryption
    pub client_pubkey: [u8; 32],
    
    /// Swap amount (plaintext - validated by ZK proof)
    pub amount: u64,
    
    /// Nullifier to prevent double-spending (from ZK proof)
    pub nullifier: [u8; 32],
    
    /// New commitment after operation (for Merkle tree)
    pub new_commitment: [u8; 32],
    
    /// Request status
    pub status: SwapRequestStatus,
    
    /// Timestamp when queued
    pub queued_at: i64,
    
    /// Timestamp when completed (0 if pending)
    pub completed_at: i64,
    
    /// Encrypted result from MPC (filled by callback)
    #[max_len(2)]
    pub encrypted_result: [[u8; 32]; 2],
    
    /// Result nonce
    pub result_nonce: u128,
}

impl EncryptedSwapRequest {
    pub const ENCRYPTED_BOUNDS_OFFSET: usize = 8 + 1 + 32 + 32 + 32 + 8;
    pub const ENCRYPTED_BOUNDS_SIZE: usize = 32 * 3;
}

/// Status of an encrypted swap request
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum SwapRequestStatus {
    /// Computation queued, waiting for ARX nodes
    Pending,
    /// Computation in progress
    Processing,
    /// Computation completed successfully
    Completed,
    /// Computation failed (check result for error)
    Failed,
    /// Request expired
    Expired,
    /// Request cancelled by user
    Cancelled,
}

impl Default for SwapRequestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Encrypted limit order
#[account]
#[derive(InitSpace)]
pub struct EncryptedLimitOrder {
    /// PDA bump seed
    pub bump: u8,
    /// User who created the order
    pub user: Pubkey,
    /// Vault for the source token
    pub source_vault: Pubkey,
    /// Vault for the destination token
    pub dest_vault: Pubkey,
    
    /// Encrypted order params: [target_price, amount, is_buy (as u64)]
    #[max_len(3)]
    pub encrypted_params: [[u8; 32]; 3],
    
    /// Nonce for encryption
    pub params_nonce: u128,
    
    /// Client's X25519 public key
    pub client_pubkey: [u8; 32],
    
    /// Expiration timestamp (plaintext)
    pub expires_at: i64,
    
    /// Order status
    pub status: LimitOrderStatus,
    
    /// Created timestamp
    pub created_at: i64,
}

impl EncryptedLimitOrder {
    pub const ENCRYPTED_PARAMS_OFFSET: usize = 8 + 1 + 32 + 32 + 32;
    pub const ENCRYPTED_PARAMS_SIZE: usize = 32 * 3;
}

/// Status of a limit order
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum LimitOrderStatus {
    /// Order is active and waiting for price trigger
    Active,
    /// Order triggered and executed
    Executed,
    /// Order cancelled by user
    Cancelled,
    /// Order expired
    Expired,
}

impl Default for LimitOrderStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// DCA (Dollar Cost Averaging) encrypted configuration
#[account]
#[derive(InitSpace)]
pub struct EncryptedDCAConfig {
    /// PDA bump seed
    pub bump: u8,
    /// User who created the DCA
    pub user: Pubkey,
    /// Source vault
    pub source_vault: Pubkey,
    /// Destination vault
    pub dest_vault: Pubkey,
    
    /// Encrypted DCA params: [amount_per_swap, swaps_remaining (as u64), min_price]
    #[max_len(3)]
    pub encrypted_params: [[u8; 32]; 3],
    
    /// Nonce for encryption
    pub params_nonce: u128,
    
    /// Client's X25519 public key
    pub client_pubkey: [u8; 32],
    
    /// Interval between swaps (seconds)
    pub interval_seconds: u64,
    
    /// Next execution timestamp
    pub next_execution_at: i64,
    
    /// DCA status
    pub status: DCAStatus,
    
    /// Created timestamp
    pub created_at: i64,
    
    /// Total swaps executed
    pub swaps_executed: u16,
}

impl EncryptedDCAConfig {
    pub const ENCRYPTED_PARAMS_OFFSET: usize = 8 + 1 + 32 + 32 + 32;
    pub const ENCRYPTED_PARAMS_SIZE: usize = 32 * 3;
}

/// Status of a DCA configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum DCAStatus {
    /// DCA is active
    Active,
    /// DCA completed all swaps
    Completed,
    /// DCA paused by user
    Paused,
    /// DCA cancelled
    Cancelled,
}

impl Default for DCAStatus {
    fn default() -> Self {
        Self::Active
    }
}
