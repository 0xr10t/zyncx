# Zyncx Architecture & Technical Reference

> **Complete technical documentation for Zyncx v0.3.0 with Arcium MXE Integration**

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Current Implementation Status](#current-implementation-status)
3. [Component Architecture](#component-architecture)
4. [Circuit Definitions](#circuit-definitions)
5. [On-Chain State Accounts](#on-chain-state-accounts)
6. [Instruction Reference](#instruction-reference)
7. [Data Flow Diagrams](#data-flow-diagrams)
8. [Security Model](#security-model)

---

## System Overview

Zyncx is a privacy-preserving DeFi protocol combining:

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Privacy Layer** | Noir ZK Circuits | Prove deposit ownership without revealing which one |
| **Confidential Compute** | Arcium MXE | Execute swap logic on encrypted data |
| **Smart Contract** | Anchor/Solana | On-chain vault management, Merkle trees |
| **DEX Integration** | Jupiter | Token swaps with MEV protection |

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           ZYNCX PROTOCOL v0.3                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐    ┌──────────────────┐    ┌────────────────────┐   │
│  │   Frontend   │───▶│  Solana Program  │───▶│    Arcium MXE      │   │
│  │  (Next.js)   │◀───│    (Anchor)      │◀───│  (MPC Circuits)    │   │
│  └──────────────┘    └──────────────────┘    └────────────────────┘   │
│         │                    │                        │               │
│         │                    │                        │               │
│         ▼                    ▼                        ▼               │
│  ┌──────────────┐    ┌──────────────────┐    ┌────────────────────┐   │
│  │ Noir Circuit │    │   Merkle Tree    │    │  Encrypted State   │   │
│  │ (ZK Proofs)  │    │   (On-Chain)     │    │  (Vault/Position)  │   │
│  └──────────────┘    └──────────────────┘    └────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Current Implementation Status

### ✅ Fully Implemented

| Component | File | Description |
|-----------|------|-------------|
| Noir ZK Circuit | `mixer/src/main.nr` | Withdrawal proofs with partial withdrawal support |
| Vault Management | `instructions/initialize.rs` | Create vaults for SOL/SPL tokens |
| Deposits | `instructions/deposit.rs` | Native SOL and SPL token deposits |
| Withdrawals | `instructions/withdraw.rs` | ZK-verified withdrawals |
| Swaps | `instructions/swap.rs` | Jupiter-integrated swaps |
| Merkle Tree | `state/merkle_tree.rs` | On-chain commitment storage |
| Nullifier System | `state/nullifier.rs` | Double-spend prevention |
| Arcium MXE Init | `lib.rs` | Computation definition setup |
| Arcium Vault Init | `encrypted-ixs/src/lib.rs::init_vault` | Initialize encrypted vault state |
| Arcium Deposits | `encrypted-ixs/src/lib.rs::process_deposit` | Process deposits in MXE |
| Confidential Swaps | `encrypted-ixs/src/lib.rs::confidential_swap` | Encrypted swap evaluation |

### 📋 Circuits Comparison (PROTOCOL.md vs Current)

| Circuit (PROTOCOL.md) | Current Status | Notes |
|-----------------------|----------------|-------|
| `init_vault` | ✅ Implemented | Initializes encrypted vault state |
| `init_position` | ❌ Removed | Simplified - positions handled off-chain |
| `process_deposit` | ✅ Implemented | Updates vault state with deposit |
| `evaluate_swap` | ⚠️ Merged | Combined into `confidential_swap` |
| `confidential_swap` | ✅ Implemented | Returns bool for swap execution |
| `evaluate_limit_order` | ❌ Removed | Future enhancement |
| `compute_withdrawal` | ❌ Removed | Handled by Noir ZK circuit |
| `clear_position` | ❌ Removed | Simplified architecture |
| `process_dca` | ❌ Removed | Future enhancement |
| `update_dca_config` | ❌ Removed | Future enhancement |
| `verify_sufficient_balance` | ❌ Removed | Handled in `process_deposit` |

### Rationale for Simplification

The current implementation focuses on the **core privacy flow**:
1. **Deposits** → Merkle tree + encrypted vault state
2. **Confidential Swaps** → Encrypted bound checking via MXE
3. **Withdrawals** → ZK proofs via Noir circuit

Advanced features (DCA, limit orders, positions) can be added incrementally.

---

## Component Architecture

### 1. Solana Program (`contracts/solana/zyncx/`)

```
src/
├── lib.rs                    # Program entry, Arcium macros
├── instructions/
│   ├── mod.rs               # Module exports
│   ├── initialize.rs        # Vault creation
│   ├── deposit.rs           # SOL/SPL deposits
│   ├── withdraw.rs          # ZK-verified withdrawals
│   ├── swap.rs              # Jupiter swaps
│   └── verify.rs            # Proof verification helpers
├── state/
│   ├── mod.rs               # Module exports
│   ├── vault.rs             # VaultState account
│   ├── merkle_tree.rs       # MerkleTreeState account
│   ├── nullifier.rs         # NullifierState account
│   ├── arcium.rs            # Legacy Arcium types
│   ├── arcium_mxe.rs        # EncryptedVaultAccount, etc.
│   ├── verifier.rs          # ZK verifier state
│   └── pyth.rs              # Oracle integration
├── dex/
│   ├── mod.rs               # DEX module
│   ├── jupiter.rs           # Jupiter integration
│   └── types.rs             # Swap types
└── errors/
    └── mod.rs               # Custom errors
```

### 2. Arcium MPC Circuits (`encrypted-ixs/`)

```
encrypted-ixs/
├── Cargo.toml               # Arcis dependencies
└── src/
    └── lib.rs               # Circuit definitions
```

**Current Circuits:**

```rust
// VaultState - encrypted aggregate data
pub struct VaultState {
    pub pending_deposits: u64,
    pub total_liquidity: u64,
    pub total_deposited: u64,
}

// init_vault - Initialize encrypted vault
pub fn init_vault(mxe: Mxe) -> Enc<Mxe, VaultState>

// process_deposit - Update vault with deposit
pub fn process_deposit(
    deposit_amount: u64,
    vault_state: Enc<Mxe, VaultState>,
) -> Enc<Mxe, VaultState>

// confidential_swap - Check if swap should execute
pub fn confidential_swap(
    encrypted_min_out: Enc<Shared, u64>,
    current_output: u64,
) -> bool
```

### 3. Noir ZK Circuit (`mixer/`)

```
mixer/
├── Nargo.toml               # Noir project config
├── Prover.toml              # Prover parameters
└── src/
    └── main.nr              # Withdrawal circuit
```

**Circuit Inputs:**

| Input | Type | Visibility | Purpose |
|-------|------|------------|---------|
| `secret` | Field | Private | Deposit ownership proof |
| `nullifier_secret` | Field | Private | Double-spend prevention |
| `new_secret` | Field | Private | Change commitment secret |
| `new_nullifier_secret` | Field | Private | Change nullifier secret |
| `merkle_path` | [Field; 20] | Private | Merkle proof path |
| `path_indices` | [Field; 20] | Private | Left/right indicators |
| `total_amount` | Field | Private | Original deposit amount |
| `root` | Field | Public | Merkle root (verified on-chain) |
| `nullifier_hash` | Field | Public | Prevents reuse |
| `recipient` | Field | Public | Withdrawal recipient |
| `withdraw_amount` | Field | Public | Amount to withdraw |
| `new_commitment` | Field | Public | Change commitment (0 if full) |

---

## On-Chain State Accounts

### VaultState (Traditional)

```rust
#[account]
pub struct VaultState {
    pub bump: u8,              // PDA bump
    pub vault_type: VaultType, // Native or SPL
    pub asset_mint: Pubkey,    // Token mint
    pub merkle_tree: Pubkey,   // Associated Merkle tree
    pub nonce: u64,            // Replay protection
    pub authority: Pubkey,     // Vault authority
    pub total_deposited: u64,  // Total deposits
}
// Seeds: [b"vault", asset_mint]
```

### EncryptedVaultAccount (Arcium MXE)

```rust
#[account]
pub struct EncryptedVaultAccount {
    pub bump: u8,                      // PDA bump
    pub token_mint: Pubkey,            // Token mint
    pub authority: Pubkey,             // Vault authority
    pub nonce: u128,                   // MXE re-encryption nonce
    pub encrypted_state: [[u8; 32]; 3], // [pending, liquidity, deposited]
}
// Seeds: [b"enc_vault", token_mint]
```

### MerkleTreeState

```rust
#[account]
pub struct MerkleTreeState {
    pub bump: u8,
    pub vault: Pubkey,
    pub depth: u8,                      // Tree depth (20)
    pub next_index: u32,                // Next leaf index
    pub current_root: [u8; 32],         // Current root
    pub root_history: [[u8; 32]; 30],   // Last 30 roots
    pub root_history_index: u8,
    pub leaves: Vec<[u8; 32]>,          // All commitments
}
// Seeds: [b"merkle_tree", vault]
```

### NullifierState

```rust
#[account]
pub struct NullifierState {
    pub bump: u8,
    pub nullifier: [u8; 32],   // The nullifier hash
    pub spent: bool,           // Whether spent
    pub spent_at: i64,         // Timestamp
    pub vault: Pubkey,         // Associated vault
}
// Seeds: [b"nullifier", vault, nullifier_bytes]
```

---

## Instruction Reference

### Phase 1: Standard Operations

| Instruction | Accounts | Args | Description |
|-------------|----------|------|-------------|
| `initialize_vault` | authority, vault, merkle_tree | asset_mint | Create new vault |
| `deposit_native` | user, vault, merkle_tree, vault_treasury | amount, precommitment | Deposit SOL |
| `deposit_token` | user, vault, merkle_tree, token_accounts | amount, precommitment | Deposit SPL |
| `withdraw_native` | user, vault, merkle_tree, nullifier | amount, nullifier, new_commitment, proof | Withdraw SOL |
| `withdraw_token` | user, vault, merkle_tree, nullifier, token_accounts | amount, nullifier, new_commitment, proof | Withdraw SPL |
| `swap_native` | user, vault, merkle_tree, jupiter_accounts | swap_param, nullifier, new_commitment, proof, swap_data | Swap from vault |

### Phase 2: Arcium MXE Operations

| Instruction | Accounts | Args | Description |
|-------------|----------|------|-------------|
| `init_vault_comp_def` | payer, mxe_account, comp_def | - | Register init_vault circuit |
| `init_process_deposit_comp_def` | payer, mxe_account, comp_def | - | Register process_deposit circuit |
| `init_confidential_swap_comp_def` | payer, mxe_account, comp_def | - | Register confidential_swap circuit |
| `create_encrypted_vault` | payer, arcium_accounts, vault | computation_offset, nonce | Create MXE vault |
| `queue_encrypted_deposit` | payer, arcium_accounts, vault | computation_offset, deposit_amount | Queue deposit to MXE |
| `queue_confidential_swap` | payer, arcium_accounts, vault | computation_offset, encrypted_min_out, encryption_pubkey, nonce, current_output | Queue swap check |

### Callbacks (Called by Arcium)

| Callback | Receives | Updates |
|----------|----------|---------|
| `init_vault_callback` | `InitVaultOutput` | vault.encrypted_state |
| `process_deposit_callback` | `ProcessDepositOutput` | vault.encrypted_state |
| `confidential_swap_callback` | `ConfidentialSwapOutput` | Emits result event |

---

## Data Flow Diagrams

### Deposit Flow

```
User                     SDK                      Solana                    Arcium
  │                       │                         │                         │
  │──deposit(10 SOL)─────▶│                         │                         │
  │                       │──gen secrets────────────│                         │
  │                       │  secret, null_secret    │                         │
  │                       │                         │                         │
  │                       │──commitment = ──────────│                         │
  │                       │  keccak(s, ns, amt)     │                         │
  │                       │                         │                         │
  │                       │──deposit_native(───────▶│                         │
  │                       │  10 SOL, commitment)    │                         │
  │                       │                         │──Insert to Merkle──────▶│
  │                       │                         │                         │
  │                       │                         │──queue_encrypted_deposit▶│
  │                       │                         │                         │
  │                       │                         │      ┌──────────────────┤
  │                       │                         │      │ MPC: Update vault│
  │                       │                         │      │ encrypted state  │
  │                       │                         │      └──────────────────┤
  │                       │                         │                         │
  │                       │                         │◀──callback with state───│
  │                       │                         │                         │
  │◀──────────────────────│◀──success + leaf_idx───│                         │
  │  STORE SECRETS!       │                         │                         │
```

### Confidential Swap Flow

```
User                     SDK                      Solana                    Arcium
  │                       │                         │                         │
  │──swap(SOL→USDC)──────▶│                         │                         │
  │  min_out=100 USDC     │                         │                         │
  │                       │──encrypt min_out───────▶│                         │
  │                       │  with MXE pubkey        │                         │
  │                       │                         │                         │
  │                       │──queue_confidential────▶│                         │
  │                       │  swap(enc_min_out,      │                         │
  │                       │  current_output=95)     │──forward to MXE────────▶│
  │                       │                         │                         │
  │                       │                         │      ┌──────────────────┤
  │                       │                         │      │ Decrypt min_out  │
  │                       │                         │      │ Compare: 95 >= 100│
  │                       │                         │      │ Result: false    │
  │                       │                         │      └──────────────────┤
  │                       │                         │                         │
  │                       │                         │◀──callback: false───────│
  │                       │                         │                         │
  │◀──────────────────────│◀──swap rejected────────│                         │
  │  (slippage too high)  │                         │                         │
```

### Withdrawal Flow (ZK)

```
User                     SDK                      Solana                    Verifier
  │                       │                         │                         │
  │──withdraw(5 SOL)─────▶│                         │                         │
  │  from 10 SOL deposit  │                         │                         │
  │                       │──gen new secrets────────│                         │
  │                       │  new_s, new_ns          │                         │
  │                       │                         │                         │
  │                       │──new_commit = ──────────│                         │
  │                       │  keccak(new_s, 5 SOL)   │                         │
  │                       │                         │                         │
  │                       │──generate ZK proof──────│                         │
  │                       │  (Noir prover)          │                         │
  │                       │                         │                         │
  │                       │──withdraw_native(──────▶│                         │
  │                       │  5 SOL, nullifier,      │──verify proof──────────▶│
  │                       │  new_commit, proof)     │                         │
  │                       │                         │◀──valid─────────────────│
  │                       │                         │                         │
  │                       │                         │──check nullifier────────│
  │                       │                         │  not spent              │
  │                       │                         │                         │
  │                       │                         │──insert new_commit──────│
  │                       │                         │                         │
  │                       │                         │──transfer 5 SOL─────────│
  │                       │                         │                         │
  │◀──────────────────────│◀──success──────────────│                         │
  │  UPDATE SECRETS       │                         │                         │
```

---

## Security Model

### Trust Assumptions

| Component | Trust Level | Failure Impact |
|-----------|-------------|----------------|
| Solana Runtime | Full | Protocol broken |
| Anchor Framework | Full | State corruption |
| Noir Proofs | Cryptographic | Fake withdrawals |
| Arcium MXE | Threshold | Swap params leak (not funds) |
| Poseidon/Keccak | Cryptographic | Commitment forgery |

### Privacy Guarantees

| What's Hidden | From Whom | How |
|---------------|-----------|-----|
| Deposit-Withdrawal Link | Everyone | ZK Merkle proof |
| Swap Min Output | Validators, MEV | MXE encryption |
| Internal Balances | Everyone | MXE encrypted state |
| Trading Strategy | Everyone | MXE computation |

### Known Limitations

| Limitation | Reason | Mitigation |
|------------|--------|------------|
| Deposit amount visible | L1 transparency | Use standard amounts |
| Withdrawal amount visible | L1 transparency | Multiple partial withdrawals |
| Timing correlation | Timestamps public | Random delays |
| Anonymity set size | Protocol adoption | Grow user base |

---

*Architecture Document v0.3.0 - February 2026*
