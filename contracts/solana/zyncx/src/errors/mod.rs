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
}
