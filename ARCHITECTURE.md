# Zyncx Protocol - Complete Architecture & Documentation

## 🎯 Overview

**Zyncx** is a privacy-preserving DeFi protocol built on Solana that combines **Zero-Knowledge Proofs (Noir)** with **Multi-Party Computation (Arcium MXE)** to enable fully confidential financial operations.

### Core Privacy Guarantees

| Feature | Technology | What's Hidden |
|---------|------------|---------------|
| Shielded Deposits | Noir ZK Proofs | Deposit amounts, depositor identity |
| Private Withdrawals | Noir ZK Proofs | Withdrawal amounts, link to deposits |
| Confidential Swaps | Arcium MPC | Trading strategy, slippage tolerance |
| Hidden Limit Orders | Arcium MPC | Target price, order size |

---

## 🏗️ Project Structure

```
zyncx/
├── contracts/solana/zyncx/    # Anchor Solana program
│   └── src/
│       ├── lib.rs             # Program entry point (#[arcium_program])
│       ├── instructions/      # Transaction handlers
│       ├── state/             # Account structures
│       ├── dex/               # Jupiter DEX integration
│       └── errors/            # Custom error codes
├── encrypted-ixs/             # Arcium MPC circuits (Arcis)
│   └── src/lib.rs             # MPC computation logic
├── mixer/                     # Noir ZK circuits
│   └── src/main.nr            # Deposit/withdrawal proofs
├── app/                       # Next.js frontend
│   ├── app/                   # App router pages
│   └── components/            # React components
├── tests/                     # Integration tests
├── Anchor.toml                # Anchor configuration
└── Cargo.toml                 # Workspace configuration
```

---

## 📁 File-by-File Documentation

### Root Configuration Files

| File | Purpose |
|------|---------|
| `Anchor.toml` | Anchor framework config - sets Anchor 0.32.1, program ID, cluster settings |
| `Cargo.toml` | Rust workspace - includes `contracts/solana/*` and `encrypted-ixs` |
| `package.json` | Node.js deps for testing (Mocha, Chai, Anchor TS) |
| `tsconfig.json` | TypeScript configuration for tests |

---

### 📦 Solana Program (`contracts/solana/zyncx/src/`)

#### `lib.rs` - Program Entry Point
The main Solana program using `#[arcium_program]` macro (extends Anchor's `#[program]`).

**Instructions Exposed:**
```rust
// Phase 1: ZK-based operations
initialize_vault       // Create new vault for SOL/SPL token
deposit_native         // Deposit SOL with commitment
deposit_token          // Deposit SPL token with commitment
withdraw_native        // Withdraw SOL with ZK proof
withdraw_token         // Withdraw SPL token with ZK proof
swap_native            // Swap via Jupiter with ZK proof
swap_token             // Swap tokens via Jupiter

// Phase 2: Arcium MXE operations
init_vault_comp_def    // Initialize MPC circuit definition
init_deposit_comp_def  // Initialize deposit circuit
init_swap_comp_def     // Initialize swap circuit
create_encrypted_vault // Create MXE-encrypted vault
queue_encrypted_deposit// Queue deposit to MPC cluster
queue_confidential_swap_mxe // Queue swap to MPC cluster
deposit_callback       // Receive MPC results
confidential_swap_callback_mxe // Receive swap results
```

---

#### `instructions/` Directory

| File | Purpose | Key Functions |
|------|---------|---------------|
| `initialize.rs` | Vault creation | Creates vault PDAs, initializes Merkle tree |
| `deposit.rs` | Handle deposits | `handler_native()`, `handler_token()` - insert commitment into Merkle tree |
| `withdraw.rs` | Handle withdrawals | Verify ZK proof, check nullifier, transfer funds |
| `swap.rs` | DEX swaps | Verify proof, execute Jupiter swap, update state |
| `verify.rs` | Proof verification | `handler()` validates Noir proofs against verifier |
| `confidential.rs` | Legacy Arcium API | Queue computation, handle callbacks (placeholder) |
| `arcium_mxe.rs` | **New Arcium Integration** | Three-instruction pattern with ArgBuilder |

##### `arcium_mxe.rs` Key Components:

```rust
// Computation Definition Initializers
InitVaultCompDef     // Registers init_vault circuit with MXE
InitDepositCompDef   // Registers process_deposit circuit
InitSwapCompDef      // Registers confidential_swap circuit

// Queue Computation Instructions  
QueueEncryptedDeposit {
    // Uses ArgBuilder to construct encrypted payload:
    // .x25519_pubkey(user_key)
    // .plaintext_u128(nonce)
    // .encrypted_u64(amount_ciphertext)
    // .account(vault_key, offset, size)  // Read MXE state
}

QueueConfidentialSwapMxe {
    // Encrypted swap bounds: min_out, max_slippage, aggressive
    // Plaintext: amount (validated by ZK), current_price (oracle)
}

// Callbacks
DepositCallback      // Updates encrypted vault_state & user_position
ConfidentialSwapCallbackMxe // Updates swap_request, vault, position
```

---

#### `state/` Directory

| File | Purpose | Key Accounts |
|------|---------|--------------|
| `vault.rs` | Basic vault state | `VaultState` - type, mint, merkle_tree, authority |
| `merkle_tree.rs` | Commitment tree | `MerkleTreeState` - depth, leaves, root history |
| `nullifier.rs` | Double-spend prevention | `NullifierState` - spent flag, timestamp |
| `verifier.rs` | ZK proof verification | Verifier key data, proof validation |
| `arcium.rs` | Legacy Arcium types | `ComputationRequest`, `ArciumConfig` |
| `arcium_mxe.rs` | **MXE Encrypted State** | New encrypted account types |
| `pyth.rs` | Oracle integration | Pyth price feed structures |

##### `arcium_mxe.rs` Encrypted Accounts:

```rust
// Encrypted vault (MXE-only decryption)
EncryptedVaultAccount {
    vault_state: [[u8; 32]; 3],  // [pending, liquidity, deposited]
    nonce: u128,                  // For re-encryption
}

// Encrypted user position
EncryptedUserPosition {
    position_state: [[u8; 32]; 2], // [deposited, lp_share]
    nonce: u128,
}

// Swap request tracking
EncryptedSwapRequest {
    encrypted_bounds: [[u8; 32]; 3], // [min_out, slippage, aggressive]
    encrypted_result: [[u8; 32]; 2], // [should_execute, min_amount]
    status: SwapRequestStatus,
}
```

**Memory Layout for ArgBuilder:**
```
Byte 0-7:    Anchor discriminator
Byte 8:      bump
Byte 9-40:   authority (Pubkey)
Byte 41-72:  token_mint (Pubkey)
Byte 73-168: vault_state (3 × 32 bytes)  ← ENCRYPTED_STATE_OFFSET = 73
Byte 169-184: nonce (u128)
```

---

#### `dex/` Directory

| File | Purpose |
|------|---------|
| `jupiter.rs` | Jupiter V6 integration - `execute_jupiter_swap()` |
| `types.rs` | `SwapRoute`, `SwapResult`, WSOL_MINT constant |
| `mod.rs` | Module exports |

**Jupiter Integration:**
- Uses Jupiter V6 program (`JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`)
- Swap data constructed off-chain via Jupiter API
- Remaining accounts passed dynamically for route execution
- Vault treasury PDA signs the swap instruction

---

#### `errors/mod.rs` - Error Codes

```rust
// Deposit/Withdraw Errors
InvalidDepositAmount, InvalidWithdrawalAmount
NullifierAlreadySpent, InvalidZKProof, RootNotFound

// Swap Errors
InvalidSwapRouter, SlippageExceeded, SwapExecutionFailed

// Arcium Errors
ClusterNotSet, AbortedComputation, InvalidMXEAccount
CorruptedEncryptedState, InvalidEncryptionParams
```

---

### 🔐 Noir ZK Circuits (`mixer/src/main.nr`)

**Purpose:** Prove ownership of shielded funds without revealing deposit details.

```noir
fn main(
    // Private inputs (user knows, verifier doesn't)
    secret: Field,
    nullifier_secret: Field,
    merkle_path: [Field; TREE_DEPTH],  // 20 levels
    path_indices: [Field; TREE_DEPTH],

    // Public inputs (verified on-chain)
    root: pub Field,           // Merkle root
    nullifier_hash: pub Field, // Prevents double-spend
    recipient: pub Field,      // Prevents front-running
    amount: pub Field,
) {
    // 1. Compute commitment = Poseidon(secret, nullifier_secret, amount)
    // 2. Verify nullifier = Poseidon(nullifier_secret)
    // 3. Verify Merkle membership (commitment is in tree)
    // 4. Constrain recipient ≠ 0
}
```

**Circuit Properties:**
- Tree depth: 20 (supports ~1M deposits)
- Hash: Poseidon (ZK-friendly, BN254 curve)
- Proof system: Generated by Noir compiler

**Compiled Artifacts:**
- `mixer/target/mixer.json` - Circuit ABI
- `mixer/target/mixer.vk` - Verification key
- `mixer/target/mixer.pk` - Proving key

---

### 🔮 Arcium MPC Circuits (`encrypted-ixs/src/lib.rs`)

**Purpose:** Execute computations on encrypted data without revealing values.

**Key Encryption Types:**
| Type | Who Decrypts | Use Case |
|------|--------------|----------|
| `Enc<Shared, T>` | Client + MXE | User inputs/outputs |
| `Enc<Mxe, T>` | MXE only | Protocol state |
| Plaintext | Everyone | Public parameters |

**Circuits Implemented:**

```rust
// Initialization
init_vault(mxe: Mxe) -> Enc<Mxe, VaultState>
init_position(mxe: Mxe) -> Enc<Mxe, UserPosition>

// Deposit Processing
process_deposit(
    deposit_input: Enc<Shared, DepositInput>,
    vault_state: Enc<Mxe, VaultState>,
    user_position: Enc<Mxe, UserPosition>,
) -> (Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>)

// Confidential Swap
confidential_swap(
    swap_bounds: Enc<Shared, SwapBounds>,  // User's hidden strategy
    vault_state: Enc<Mxe, VaultState>,
    user_position: Enc<Mxe, UserPosition>,
    swap_amount: u64,                       // Public (from ZK proof)
    current_price: u64,                     // Public (from Pyth)
) -> (Enc<Shared, SwapResult>, Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>)

// Limit Orders
evaluate_limit_order(
    order: Enc<Shared, LimitOrderParams>,
    current_price: u64,
    current_time: u64,
) -> Enc<Shared, bool>

// Withdrawals
compute_withdrawal(
    user_position: Enc<Mxe, UserPosition>,
    vault_state: Enc<Mxe, VaultState>,
    user_pubkey: Shared,
) -> Enc<Shared, u64>
```

---

### 🖥️ Frontend (`app/`)

| File | Purpose |
|------|---------|
| `app/page.tsx` | Main landing page |
| `app/layout.tsx` | Root layout with providers |
| `app/globals.css` | Tailwind styles |
| `components/Navbar.tsx` | Navigation bar |
| `components/HeroSection.tsx` | Landing hero |
| `components/HowItWorks.tsx` | Feature explanation |
| `components/PrivacyVault.tsx` | Main deposit/withdraw UI |
| `components/WalletProvider.tsx` | Solana wallet adapter |
| `components/Footer.tsx` | Page footer |

**Stack:** Next.js 14, Tailwind CSS, @solana/wallet-adapter

---

## 🔄 Privacy Flow Architecture

### Flow 1: Shielded Deposit (ZK)

```
┌─────────────────────────────────────────────────────────────────┐
│ USER                           │ SOLANA                         │
├────────────────────────────────┼─────────────────────────────────
│ 1. Generate secret, nullifier  │                                │
│ 2. Compute commitment =        │                                │
│    Poseidon(secret, null, amt) │                                │
│ 3. Call deposit_native(        │                                │
│      amount, commitment)       │                                │
│                                │ 4. Insert commitment into tree │
│                                │ 5. Transfer SOL to vault       │
│                                │ 6. Emit DepositEvent           │
│ 7. Store secret locally        │                                │
└────────────────────────────────┴─────────────────────────────────
```

### Flow 2: Private Withdrawal (ZK)

```
┌─────────────────────────────────────────────────────────────────┐
│ USER                           │ SOLANA                         │
├────────────────────────────────┼─────────────────────────────────
│ 1. Generate ZK proof:          │                                │
│    - Prove knowledge of secret │                                │
│    - Prove commitment in tree  │                                │
│    - Reveal nullifier hash     │                                │
│ 2. Call withdraw_native(       │                                │
│      amount, nullifier, proof) │                                │
│                                │ 3. Verify ZK proof on-chain    │
│                                │ 4. Check nullifier not spent   │
│                                │ 5. Mark nullifier as spent     │
│                                │ 6. Transfer SOL to recipient   │
└────────────────────────────────┴─────────────────────────────────
```

### Flow 3: Confidential Swap (MPC)

```
┌───────────────────────────────────────────────────────────────────────────┐
│ USER                    │ SOLANA              │ ARCIUM MXE                │
├─────────────────────────┼─────────────────────┼───────────────────────────
│ 1. Encrypt swap bounds: │                     │                           │
│    - min_output         │                     │                           │
│    - max_slippage       │                     │                           │
│ 2. Generate ZK proof    │                     │                           │
│    (proves ownership)   │                     │                           │
│ 3. Call queue_swap_mxe( │                     │                           │
│      encrypted_bounds,  │                     │                           │
│      proof)             │                     │                           │
│                         │ 4. Verify ZK proof  │                           │
│                         │ 5. Queue to MXE     │                           │
│                         │                     │ 6. ARX nodes decrypt      │
│                         │                     │ 7. Run MPC protocol       │
│                         │                     │ 8. Compute: should_exec?  │
│                         │                     │    min_amount_out?        │
│                         │ 9. Callback with    │                           │
│                         │    encrypted result │                           │
│                         │ 10. If execute:     │                           │
│                         │     Jupiter swap    │                           │
│ 11. Decrypt result      │                     │                           │
└─────────────────────────┴─────────────────────┴───────────────────────────
```

---

## 🛠️ Technology Stack

| Layer | Technology | Version |
|-------|------------|---------|
| **Blockchain** | Solana | 1.18.x |
| **Smart Contracts** | Anchor | 0.32.1 |
| **ZK Proofs** | Noir | Latest |
| **MPC** | Arcium | 0.6.3 |
| **DEX** | Jupiter V6 | Mainnet |
| **Oracle** | Pyth Network | V2 |
| **Frontend** | Next.js | 14.x |
| **Styling** | Tailwind CSS | 3.x |
| **Wallet** | @solana/wallet-adapter | 0.19.x |

---

## 🔐 Security Model

### Cryptographic Primitives

| Component | Algorithm | Security Level |
|-----------|-----------|----------------|
| Commitment Hash | Poseidon (BN254) | ~128-bit |
| Merkle Tree | Keccak256 | 256-bit |
| ZK Proofs | Noir/Barretenberg | 128-bit |
| MPC Encryption | X25519 + AES-GCM | 128-bit |
| MPC Protocol | Cerberus (Arcium) | t-of-n threshold |

### Trust Assumptions

1. **Solana validators** - Standard Solana security model
2. **Noir verifier** - Trusted compilation (can be verified)
3. **Arcium MXE cluster** - Honest majority (t+1 of n nodes)
4. **Jupiter aggregator** - Non-custodial (funds never leave user control)
5. **Pyth oracle** - Decentralized price feeds with staleness checks

---

## 📊 Account Space Requirements

| Account | Size (bytes) | Rent (SOL) |
|---------|--------------|------------|
| VaultState | ~120 | ~0.001 |
| MerkleTreeState | ~4,000 | ~0.028 |
| NullifierState | ~80 | ~0.001 |
| EncryptedVaultAccount | ~200 | ~0.002 |
| EncryptedUserPosition | ~170 | ~0.002 |
| EncryptedSwapRequest | ~350 | ~0.003 |

---

## 🚀 Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          MAINNET/DEVNET                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │   Zyncx     │  │   Arcium    │  │   Jupiter   │                 │
│  │  Program    │  │   Program   │  │   V6        │                 │
│  │  (Anchor)   │  │   (MXE)     │  │   Program   │                 │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │
│         │                │                │                         │
│         │    CPI         │    CPI         │                         │
│         ▼                ▼                ▼                         │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                     SOLANA RUNTIME                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Off-chain:                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │   Arcium    │  │   Pyth      │  │   Circuit   │                 │
│  │   ARX Nodes │  │   Oracle    │  │   Storage   │                 │
│  │   (Docker)  │  │   Network   │  │   (GitHub)  │                 │
│  └─────────────┘  └─────────────┘  └─────────────┘                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📝 Build & Deploy Commands

```bash
# Build everything with Arcium CLI
arcium build

# Run tests (starts local ARX nodes)
arcium test

# Deploy to devnet
arcium deploy \
  --cluster-offset 456 \
  --rpc-url devnet \
  --program-name zyncx

# Initialize computation definitions (one-time)
anchor run init-comp-defs

# Deploy Noir verifier
solana program deploy mixer/target/mixer.so
```

---

## 📚 Further Reading

- [ROADMAP.md](ROADMAP.md) - Remaining tasks and integration guide
- [arcium-dev-skill/](arcium-dev-skill/) - Arcium SDK documentation
- [mixer/](mixer/) - Noir circuit documentation
- [Arcium Docs](https://docs.arcium.com) - Official Arcium documentation
- [Noir Docs](https://noir-lang.org/docs) - Noir language reference
- [Anchor Docs](https://www.anchor-lang.com) - Anchor framework guide

---

*Last Updated: January 31, 2026*
