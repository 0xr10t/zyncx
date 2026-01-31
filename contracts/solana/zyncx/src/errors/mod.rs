use anchor_lang::prelude::*;

#[error_code]
pub enum ZyncxError {
    #[msg("Invalid deposit amount - must be greater than zero")]
    InvalidDepositAmount,

    #[msg("Invalid withdrawal amount - must be greater than zero")]
    InvalidWithdrawalAmount,

    #[msg("Invalid swap amount - must be greater than zero")]
    InvalidSwapAmount,

    #[msg("Invalid fee amount - must be greater than zero")]
    InvalidFeeAmount,

    #[msg("Vault not found for the specified asset")]
    VaultNotFound,

    #[msg("Vault already registered for this asset")]
    VaultAlreadyRegistered,

    #[msg("Zero address provided")]
    ZeroAddress,

    #[msg("Nullifier has already been spent")]
    NullifierAlreadySpent,

    #[msg("Invalid ZK proof verification failed")]
    InvalidZKProof,

    #[msg("Amount mismatch between expected and received")]
    AmountMismatch,

    #[msg("Native SOL transfer failed")]
    NativeTransferFailed,

    #[msg("Native fund received for alternative asset deposit")]
    NativeFundReceived,

    #[msg("Merkle tree has reached maximum depth")]
    MaxDepthReached,

    #[msg("Invalid Merkle proof")]
    InvalidMerkleProof,

    #[msg("Root not found in history")]
    RootNotFound,

    #[msg("Poseidon hash computation failed")]
    PoseidonHashFailed,

    #[msg("Invalid commitment - cannot be zero")]
    InvalidCommitment,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Invalid public inputs for ZK proof")]
    InvalidPublicInputs,

    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid swap router program")]
    InvalidSwapRouter,

    #[msg("Insufficient funds in vault")]
    InsufficientFunds,

    #[msg("Swap slippage exceeded")]
    SlippageExceeded,

    #[msg("Invalid swap route")]
    InvalidSwapRoute,

    #[msg("DEX swap execution failed")]
    SwapExecutionFailed,

    // ========================================================================
    // Arcium / Confidential Computation Errors
    // ========================================================================
    
    #[msg("Confidential swaps are currently disabled")]
    ConfidentialSwapsDisabled,

    #[msg("Amount is below minimum threshold")]
    AmountTooSmall,

    #[msg("Amount exceeds maximum threshold")]
    AmountTooLarge,

    #[msg("Invalid computation status for this operation")]
    InvalidComputationStatus,

    #[msg("Computation request has expired")]
    ComputationExpired,

    #[msg("Computation has not expired yet - cannot cancel")]
    ComputationNotExpired,

    #[msg("Invalid Arcium callback signature")]
    InvalidArciumSignature,

    #[msg("Invalid encrypted strategy format")]
    InvalidEncryptedStrategy,

    #[msg("Invalid token mint for operation")]
    InvalidMint,

    #[msg("Invalid price feed data")]
    InvalidPriceFeed,

    #[msg("Price feed is stale")]
    StalePriceFeed,

    #[msg("Price condition not met")]
    PriceConditionNotMet,

    // ========================================================================
    // Arcium MXE Specific Errors
    // ========================================================================
    
    #[msg("Arcium cluster not set for this MXE")]
    ClusterNotSet,

    #[msg("Computation was aborted by the MXE")]
    AbortedComputation,

    #[msg("Invalid MXE account")]
    InvalidMXEAccount,

    #[msg("Invalid computation definition")]
    InvalidComputationDef,

    #[msg("Invalid callback from MXE")]
    InvalidCallback,

    #[msg("Encrypted state is corrupted")]
    CorruptedEncryptedState,

    #[msg("User position not found")]
    PositionNotFound,

    #[msg("Invalid encryption parameters")]
    InvalidEncryptionParams,

    // ========================================================================
    // Cross-Token Swap Errors
    // ========================================================================

    #[msg("Cannot swap within same vault - use withdraw instead")]
    SameVaultSwap,

    #[msg("Token mint does not match vault")]
    TokenMintMismatch,

    #[msg("Destination vault not found")]
    DestinationVaultNotFound,
}
