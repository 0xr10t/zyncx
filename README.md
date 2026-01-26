# Zyncx

Privacy-preserving vault system with confidential computation on Solana, powered by Arcium MPC and Noir circuits.

## Overview

Zyncx enables private deposits, withdrawals, and swaps using confidential computation. Users can:

1. **Deposit** - Add funds (SOL or SPL tokens) to a vault and receive a commitment
2. **Withdraw** - Withdraw funds privately without revealing the original deposit
3. **Swap** - Swap tokens via DEX while maintaining privacy

## Architecture

```
contracts/solana/zyncx/src/
├── lib.rs                 # Program entrypoint
├── errors/
│   └── mod.rs            # Custom error types
├── instructions/
│   ├── mod.rs            # Instruction exports
│   ├── initialize.rs     # Vault initialization
│   ├── deposit.rs        # Native SOL and SPL token deposits
│   ├── withdraw.rs       # Private withdrawals
│   ├── swap.rs           # Private swaps via DEX
│   └── verify.rs         # Verification utilities
└── state/
    ├── mod.rs            # State exports
    ├── merkle_tree.rs    # Incremental Merkle tree with Poseidon hashing
    ├── vault.rs          # Vault state (Native/Alternative)
    ├── nullifier.rs      # Nullifier tracking to prevent double-spending
    └── verifier.rs       # Proof verification structures
```

## Key Components

### Merkle Tree
- Incremental Merkle tree using Poseidon hash (BN254 curve)
- Stores up to 100 historical roots for flexibility
- Maximum depth of 32 levels

### Vault Types
- **NativeVault**: Handles SOL deposits/withdrawals
- **AlternativeVault**: Handles SPL token deposits/withdrawals

### Confidential Computation
- Integrates with Arcium for MPC-based private computation
- Noir circuits for proof generation
- Nullifiers tracked on-chain to prevent double-spending

## Building

```bash
# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

## Program Instructions

### `initialize_vault`
Initialize a new vault for a specific asset.

```typescript
await program.methods
  .initializeVault(assetMint)
  .accounts({
    authority: wallet.publicKey,
    vault: vaultPda,
    merkleTree: merkleTreePda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### `deposit_native` / `deposit_token`
Deposit funds and receive a commitment.

```typescript
const commitment = await program.methods
  .depositNative(amount, precommitment)
  .accounts({
    depositor: wallet.publicKey,
    vault: vaultPda,
    merkleTree: merkleTreePda,
    vaultTreasury: vaultTreasuryPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### `withdraw_native` / `withdraw_token`
Withdraw funds privately.

```typescript
await program.methods
  .withdrawNative(amount, nullifier, newCommitment, proof)
  .accounts({
    recipient: recipientPubkey,
    vault: vaultPda,
    merkleTree: merkleTreePda,
    vaultTreasury: vaultTreasuryPda,
    nullifierAccount: nullifierPda,
    payer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### `swap_native` / `swap_token`
Swap tokens via DEX privately.

```typescript
await program.methods
  .swapNative(swapParam, nullifier, newCommitment, proof)
  .accounts({
    recipient: recipientPubkey,
    vault: vaultPda,
    merkleTree: merkleTreePda,
    vaultTreasury: vaultTreasuryPda,
    nullifierAccount: nullifierPda,
    swapRouter: dexRouterPubkey,
    payer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## Dependencies

- `anchor-lang` - Anchor framework
- `anchor-spl` - SPL token integration
- `light-poseidon` - Poseidon hash for Merkle tree

## License

MIT
