# Zyncx Protocol: Privacy-Preserving DeFi on Solana

> **A comprehensive guide to what Zyncx hides, how it works, and its limitations**

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [What Zyncx Hides](#what-zyncx-hides)
3. [What Zyncx Does NOT Hide](#what-zyncx-does-not-hide)
4. [How Privacy Works](#how-privacy-works)
5. [Technical Architecture](#technical-architecture)
6. [File Structure & Components](#file-structure--components)
7. [User Flows](#user-flows)
8. [Limitations & Known Trade-offs](#limitations--known-trade-offs)
9. [Security Considerations](#security-considerations)
10. [Comparison with Other Privacy Protocols](#comparison-with-other-privacy-protocols)

---

## Executive Summary

Zyncx is a privacy-preserving DeFi protocol on Solana that combines:

| Technology | Purpose |
|------------|---------|
| **Noir ZK Circuits** | Prove ownership without revealing which deposit is yours |
| **Arcium MXE** | Execute trading logic in encrypted Multi-Party Computation |
| **Poseidon Hashes** | Efficient cryptographic commitments (ZK-friendly) |
| **Merkle Trees** | Efficient membership proofs (is my deposit in the pool?) |
| **Nullifiers** | Prevent double-spending without revealing identity |

**The core privacy guarantee:** Once funds enter the shielded pool, all operations (swaps, balance changes, partial withdrawals) are hidden from observers until final withdrawal.

---

## What Zyncx Hides

### ✅ Fully Hidden

| What's Hidden | How It's Hidden | Who Can't See It |
|---------------|-----------------|------------------|
| **Withdrawal-to-Deposit Link** | ZK proofs only reveal "I own *some* deposit" without revealing which one | Everyone |
| **Trading Strategy** | Executed inside Arcium MXE encrypted enclave | Everyone (including Arcium nodes) |
| **Swap Parameters** | Min/max bounds, slippage tolerance encrypted | Validators, MEV bots |
| **Swap Amount** | Encrypted via `Enc<Shared, SwapInput>` - only user + MXE can decrypt | Everyone except user |
| **Internal Balance Changes** | All state changes happen in encrypted memory | Everyone |
| **Number of Operations** | Multiple swaps between deposit and withdrawal are invisible | Everyone |
| **Partial Withdrawal Pattern** | Each partial withdrawal creates fresh commitment | Blockchain observers |

### 🔐 How Link Breaking Works

```
DEPOSIT PHASE:
  User deposits 10 SOL → Commitment A inserted into Merkle tree
  Commitment A = Poseidon(secret_A, nullifier_secret_A, 10 SOL)
  
  On-chain: "Someone deposited 10 SOL" (amount visible at deposit)
  Hidden: secret_A, nullifier_secret_A

INTERNAL OPERATIONS (inside Arcium MXE):
  Swap 5 SOL → USDC → SOL (arbitrage)
  Balance changes: 10 SOL → 12 SOL
  
  On-chain: Nothing visible!
  All computation in encrypted MPC
  Swap amount is ENCRYPTED (Enc<Shared, SwapInput>)

WITHDRAWAL PHASE:
  User generates ZK proof:
    "I know secret_A and nullifier_secret_A for SOME commitment in the tree
     that has sufficient balance for this withdrawal"
  
  Proof reveals: nullifier_hash, withdraw_amount, new_commitment (for change)
  Proof HIDES: which commitment, what the original secret was
  
  On-chain: "Someone withdrew 5 SOL" (amount visible at withdrawal)
  Hidden: Which deposit this came from
```

---

## What Zyncx Does NOT Hide

### ❌ Visible On-Chain (Fundamental L1 Limitations)

| What's Visible | Why It's Visible | Mitigation Strategy |
|----------------|------------------|---------------------|
| **Deposit Amount** | SOL transfer requires amount in transaction | Use multiple deposits of common amounts |
| **Withdrawal Amount** | SOL transfer requires amount in transaction | Use multiple partial withdrawals |
| **Deposit Timestamp** | Block timestamp is public | Wait variable times before withdrawing |
| **Withdrawal Timestamp** | Block timestamp is public | Use relayers, random delays |
| **Gas Fee Patterns** | Transaction metadata is public | Batch with other users |

### Why Can't We Hide Deposit/Withdrawal Amounts?

Solana (and all L1 blockchains) require explicit amounts in transfer instructions:

```rust
// This is unavoidable - SOL transfers need explicit amounts
system_program::transfer(
    CpiContext::new(/* ... */),
    amount,  // ← Visible in transaction
)?;
```

**However:** Internal swap amounts ARE hidden via Arcium MXE encryption!

---

## How Privacy Works

### 1. Commitment Scheme

Every deposit creates a **commitment** that hides the deposit details:

```
commitment = Poseidon(secret, nullifier_secret, amount)
```

| Component | Description | Privacy Role |
|-----------|-------------|---------------|
| `secret` | Random 256-bit value | Proves ownership |
| `nullifier_secret` | Random 256-bit value | Prevents double-spend |
| `amount` | Deposit amount in lamports | Binds value to commitment |

The commitment is inserted into the on-chain Merkle tree for that token's vault.

### 2. Merkle Tree Membership

To withdraw, you prove your commitment is in the tree WITHOUT revealing which leaf:

```
                    ROOT (public)
                   /            \
                 H1              H2
                /  \            /  \
              H3    H4        H5    H6
             / \   / \       / \   / \
            L0 L1 L2 L3    L4 L5 L6 L7  (your leaf is one of these)
```

The ZK proof shows: "I know a path from SOME leaf to this root" without revealing which leaf.

### 3. Nullifier System

The nullifier prevents spending the same deposit twice:

```
nullifier_hash = Poseidon(nullifier_secret)
```

- When you withdraw, you reveal `nullifier_hash`
- The contract stores it in a PDA (Program Derived Address)
- If you try to withdraw again with the same commitment, the nullifier already exists → rejected

**Privacy property:** Seeing two nullifier_hashes tells you nothing about whether they came from the same depositor.

### 4. Partial Withdrawals

Zyncx supports withdrawing part of your balance:

```
BEFORE: Commitment A = Poseidon(secret_A, null_A, 10 SOL)

WITHDRAW 3 SOL:
  - Reveal: nullifier_hash_A (marks commitment A as spent)
  - Receive: 3 SOL
  - Create: Commitment B = Poseidon(secret_B, null_B, 7 SOL)  ← NEW secrets!
  
AFTER: Commitment A is spent, Commitment B is live with 7 SOL
```

The ZK circuit verifies:
1. You know the secrets for commitment A
2. Commitment A has at least 3 SOL
3. The new commitment B is correctly formed with (original - withdrawal) amount

### 5. Arcium MXE (Multi-Party Computation)

Trading logic runs inside Arcium's encrypted execution environment:

```
USER → Encrypted Inputs → [ARCIUM MXE] → Encrypted Outputs → USER
                              ↓
                     (Computation happens here)
                     (No single node sees plaintext)
                     (Only threshold of nodes together)
                     (Even they don't see your data)
```

**What runs inside MXE:**
- Balance checking (do you have enough?)
- Swap amount validation (encrypted!)
- Swap bound verification (is min_out met?)
- Slippage calculations
- Fee computations

**What Arcium nodes see:** Encrypted blobs. Even with collusion, they only see that "some computation happened."

### 6. Encrypted Swap Amounts

The `confidential_swap` function now encrypts the swap amount:

```rust
#[instruction]
pub fn confidential_swap(
    swap_input: Enc<Shared, SwapInput>,   // ← ENCRYPTED swap amount!
    swap_bounds: Enc<Shared, SwapBounds>, // ← ENCRYPTED trading bounds
    vault_state: Enc<Mxe, VaultState>,
    user_position: Enc<Mxe, UserPosition>,
    current_price: u64,                    // Plaintext from oracle
) -> (Enc<Shared, SwapResult>, Enc<Mxe, VaultState>, Enc<Mxe, UserPosition>)
```

| Parameter | Encryption | Who Can Decrypt |
|-----------|------------|-----------------|
| `swap_input.amount` | `Enc<Shared, _>` | User + MXE only |
| `swap_bounds` | `Enc<Shared, _>` | User + MXE only |
| `vault_state` | `Enc<Mxe, _>` | MXE only |
| `user_position` | `Enc<Mxe, _>` | MXE only |
| `current_price` | Plaintext | Everyone (from Pyth oracle) |

---

## Technical Architecture

### System Components

```
┌──────────────────────────────────────────────────────────────────────┐
│                           ZYNCX PROTOCOL                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────┐     ┌─────────────────┐     ┌────────────────┐ │
│  │   USER CLIENT   │────▶│  SOLANA PROGRAM │────▶│  ARCIUM MXE    │ │
│  │  (SDK/Frontend) │◀────│    (Anchor)     │◀────│  (Encrypted)   │ │
│  └─────────────────┘     └─────────────────┘     └────────────────┘ │
│          │                       │                       │          │
│          │                       │                       │          │
│          ▼                       ▼                       ▼          │
│  ┌─────────────────┐     ┌─────────────────┐     ┌────────────────┐ │
│  │  NOIR CIRCUIT   │     │  MERKLE TREE    │     │  ENCRYPTED     │ │
│  │  (ZK Proofs)    │     │  (On-Chain)     │     │  VAULT STATE   │ │
│  └─────────────────┘     └─────────────────┘     └────────────────┘ │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
1. DEPOSIT
   User ──(amount, precommitment)──▶ Solana Program
                                          │
                                          ├──▶ Insert commitment into Merkle tree
                                          ├──▶ Transfer SOL to vault
                                          └──▶ Emit event (amount visible)

2. CONFIDENTIAL SWAP (Inside Arcium)
   User ──(encrypted params)──▶ Queue Computation ──▶ Arcium MXE
                                                          │
                                                          ├──▶ Decrypt in secure enclave
                                                          ├──▶ Execute swap logic
                                                          ├──▶ Re-encrypt results
                                                          └──▶ Callback to Solana

3. WITHDRAWAL
   User ──(ZK proof, nullifier, amount)──▶ Solana Program
                                                │
                                                ├──▶ Verify ZK proof (Sunspot CPI)
                                                ├──▶ Check nullifier not spent
                                                ├──▶ Insert new_commitment (if partial)
                                                ├──▶ Transfer SOL to user
                                                └──▶ Mark nullifier as spent
```

---

## File Structure & Components

### Directory Overview

```
zyncx/
├── contracts/solana/zyncx/    # Anchor smart contract
│   └── src/
│       ├── lib.rs             # Program entry point
│       ├── instructions/      # Transaction handlers
│       ├── state/             # On-chain account structures
│       ├── dex/               # Jupiter DEX integration
│       └── errors/            # Custom error types
├── mixer/                     # Noir ZK circuits
│   └── src/
│       └── main.nr            # Withdrawal proof circuit
├── encrypted-ixs/             # Arcium Arcis circuits (MPC logic)
│   └── src/
│       └── lib.rs             # MPC circuit definitions
└── app/                       # Next.js frontend
```

### Detailed File Reference

#### Solana Program (`contracts/solana/zyncx/src/`)

| File | Purpose | Key Functions |
|------|---------|---------------|
| `lib.rs` | Program entry, instruction routing | `deposit_native`, `withdraw_native`, `queue_confidential_swap` |
| `instructions/initialize.rs` | Vault setup | Creates vault PDA, Merkle tree account |
| `instructions/deposit.rs` | Handle deposits | Transfers SOL, inserts commitment |
| `instructions/withdraw.rs` | Handle withdrawals | Verifies proof, checks nullifier, sends SOL |
| `instructions/swap.rs` | DEX swaps | Jupiter CPI calls |
| `instructions/confidential.rs` | Confidential ops | Queue computations, handle callbacks |
| `state/vault.rs` | Vault state | Total deposits, asset mint |
| `state/merkle_tree.rs` | Merkle tree | Leaves, root history, insert logic |
| `state/nullifier.rs` | Nullifier PDA | Prevents double-spending |
| `state/arcium.rs` | Arcium config | Computation request tracking |
| `state/arcium_mxe.rs` | MXE state | Encrypted vault, user positions |
| `state/pyth.rs` | Oracle integration | Price feed structures |
| `dex/jupiter.rs` | Jupiter SDK | Swap routing, CPI |

#### Arcis MPC Circuits (`encrypted-ixs/src/lib.rs`)

| Circuit | Purpose | Encrypted Inputs |
|---------|---------|------------------|
| `init_vault` | Initialize encrypted vault state | None (MXE only) |
| `init_position` | Initialize user position | None (MXE only) |
| `process_deposit` | Process deposit, update vault | `DepositInput` |
| `evaluate_swap` | Check swap without state change | `SwapBounds` |
| `confidential_swap` | Full swap with state update | `SwapInput`, `SwapBounds` |
| `evaluate_limit_order` | Check limit order trigger | `LimitOrderParams` |
| `compute_withdrawal` | Calculate withdrawal amount | `UserPosition` |
| `clear_position` | Clear position after withdrawal | `UserPosition` |
| `process_dca` | DCA interval execution | `DCAConfig` |
| `update_dca_config` | Update DCA after swap | `DCAConfig` |
| `verify_sufficient_balance` | Balance check | `BalanceCheckInput` |

#### Noir Circuit (`mixer/src/main.nr`)

**Circuit Inputs/Outputs:**

```noir
// PRIVATE (prover knows, verifier doesn't see)
secret: Field                      // Original deposit secret
nullifier_secret: Field            // Original nullifier secret
total_amount: Field                // Full commitment amount
merkle_path: [Field; TREE_HEIGHT]  // Sibling hashes
path_indices: [Field; TREE_HEIGHT] // Left/right indicators
new_secret: Field                  // Fresh secret for change
new_nullifier_secret: Field        // Fresh nullifier for change

// PUBLIC (verified on-chain)
root: Field                        // Merkle tree root
nullifier_hash: Field              // Marks old commitment spent
recipient: Field                   // Withdrawal recipient
withdraw_amount: Field             // Amount being withdrawn
new_commitment: Field              // Change commitment (or 0 for full withdrawal)
```

---

## User Flows

### Flow 1: Private Deposit

```
User                    SDK                     Solana Program         Merkle Tree
  │                      │                            │                     │
  │──deposit(10 SOL)────▶│                            │                     │
  │                      │──generate secrets──────────│                     │
  │                      │  secret, nullifier_secret  │                     │
  │                      │                            │                     │
  │                      │──commitment = Poseidon(...)│                     │
  │                      │                            │                     │
  │                      │──deposit_native(10 SOL,────▶                     │
  │                      │   commitment)              │                     │
  │                      │                            │──Transfer 10 SOL───▶│
  │                      │                            │                     │
  │                      │                            │──Insert commitment─▶│
  │                      │                            │                     │
  │◀─────────────────────│◀───Return commitment───────│                     │
  │  Store secrets       │                            │                     │
  │  locally (CRITICAL!) │                            │                     │
```

### Flow 2: Confidential Swap (with Encrypted Amount)

```
User                    SDK                     Solana Program         Arcium MXE
  │                      │                            │                     │
  │──swap(SOL→USDC)─────▶│                            │                     │
  │                      │                            │                     │
  │                      │──Encrypt with MXE pubkey:  │                     │
  │                      │  - swap_amount (HIDDEN!)   │                     │
  │                      │  - min_out, slippage       │                     │
  │                      │                            │                     │
  │                      │──queue_confidential_swap───▶                     │
  │                      │  (encrypted_params)        │                     │
  │                      │                            │──Forward encrypted──▶│
  │                      │                            │   request           │
  │                      │                            │                     │
  │                      │                            │    ┌────────────────┤
  │                      │                            │    │ Decrypt in     │
  │                      │                            │    │ secure enclave │
  │                      │                            │    │ Execute swap   │
  │                      │                            │    │ Re-encrypt     │
  │                      │                            │    └────────────────┤
  │                      │                            │                     │
  │                      │                            │◀──Callback with─────│
  │                      │                            │   encrypted result  │
  │                      │                            │                     │
  │◀─────────────────────│◀───Swap complete───────────│                     │
```

### Flow 3: Partial Withdrawal

```
User                    SDK                     Solana Program         ZK Verifier
  │                      │                            │                     │
  │──withdraw(3 SOL)────▶│                            │                     │
  │  from 10 SOL balance │                            │                     │
  │                      │                            │                     │
  │                      │──Generate new secrets──────│                     │
  │                      │  new_secret, new_null_sec  │                     │
  │                      │                            │                     │
  │                      │──new_commitment = ─────────│                     │
  │                      │  Poseidon(new_s, new_n, 7) │                     │
  │                      │                            │                     │
  │                      │──Generate ZK proof─────────│                     │
  │                      │                            │                     │
  │                      │──withdraw_native(3 SOL,────▶                     │
  │                      │   nullifier, new_commit,   │                     │
  │                      │   proof)                   │──Verify proof──────▶│
  │                      │                            │                     │
  │                      │                            │◀──Proof valid───────│
  │                      │                            │                     │
  │                      │                            │──Check nullifier────│
  │                      │                            │  not spent          │
  │                      │                            │                     │
  │                      │                            │──Insert new_commit──│
  │                      │                            │  into Merkle tree   │
  │                      │                            │                     │
  │◀─────────────────────│◀───Transfer 3 SOL──────────│                     │
  │                      │                            │                     │
  │  Update local secrets│                            │                     │
  │  (old spent, new)    │                            │                     │
```

---

## Limitations & Known Trade-offs

### Fundamental Limitations

| Limitation | Cause | Impact | Mitigation |
|------------|-------|--------|------------|
| **Visible deposit amounts** | L1 transaction transparency | Observers know how much you deposited | Use standard amounts (1, 10, 100 SOL) |
| **Visible withdrawal amounts** | L1 transaction transparency | Observers know how much you withdrew | Multiple partial withdrawals |
| **Timing correlation** | Blockchain timestamps | Deposit at T1, withdraw at T2 → possible link | Wait random periods |
| **Gas patterns** | Transaction metadata | Repeated patterns can be fingerprinted | Use relayers |
| **Anonymity set size** | Number of other deposits | Few deposits = easier correlation | Protocol adoption |

### Protocol-Specific Limitations

| Limitation | Description |
|------------|-------------|
| **Merkle tree capacity** | 2^20 (~1M) commitments per vault |
| **Root history** | Only last 30 roots stored; old roots expire |
| **Computation timeout** | Arcium computations expire after configured timeout |
| **Single asset per vault** | Each vault handles one asset type |

### What an Adversary Can Learn

| Attack Vector | What They See | What They Can't Determine |
|---------------|---------------|---------------------------|
| On-chain observation | Deposit/withdrawal amounts, timestamps | Which deposit maps to which withdrawal |
| Arcium node compromise* | Encrypted blobs | Swap amounts, parameters, balances |
| Merkle tree analysis | Tree structure, root changes | Which leaf belongs to whom |
| Transaction graph | Gas payer addresses | True owner of shielded funds |

*Requires compromising threshold of nodes AND breaking encryption

---

## Security Considerations

### Trust Assumptions

| Component | Trust Assumption | What If Violated |
|-----------|------------------|------------------|
| **Noir ZK Proofs** | Cryptographic soundness | Fake proofs could drain funds |
| **Sunspot Verifier** | Correct implementation | Invalid proofs accepted |
| **Arcium MXE** | Honest threshold of nodes | Swap params could leak (but not funds) |
| **Solana Runtime** | Correct execution | State could be corrupted |
| **Poseidon Hash** | Collision resistance | Commitment forgery |

### Operational Security for Users

1. **NEVER lose your secrets** - Without `secret` and `nullifier_secret`, funds are unrecoverable
2. **Store secrets offline** - Encrypted backup, never in browser localStorage
3. **Use fresh addresses** - Withdraw to new addresses to prevent correlation
4. **Wait before withdrawing** - Immediate withdrawal links deposit to you
5. **Use standard amounts** - 1, 10, 100 SOL instead of 7.3284 SOL

### Known Attack Vectors & Mitigations

| Attack | Description | Mitigation |
|--------|-------------|------------|
| **Front-running** | MEV bot replaces recipient | Recipient bound to proof |
| **Double-spend** | Withdraw same deposit twice | Nullifier system |
| **Replay** | Use old proof on new root | Root checked against history |
| **Timing attack** | Correlate by timestamp | User-controlled delays |
| **Dusting attack** | Send micro-deposits to track | Merge commitments in MXE |

---

## Comparison with Other Privacy Protocols

| Feature | Zyncx | Tornado Cash | Aztec | Zcash |
|---------|-------|--------------|-------|-------|
| **Blockchain** | Solana | Ethereum | Ethereum L2 | Own chain |
| **Proof System** | Noir (PLONK) | Groth16 | PLONK | Groth16 |
| **Confidential Compute** | Arcium MXE | None | Rollup | None |
| **Partial Withdrawals** | ✅ Yes | ❌ No (fixed amounts) | ✅ Yes | ✅ Yes |
| **DeFi Integration** | ✅ Yes (Jupiter) | ❌ No | ⚠️ Limited | ❌ No |
| **Trading Privacy** | ✅ Full (MPC) | N/A | ⚠️ Partial | N/A |
| **Encrypted Swap Amounts** | ✅ Yes | N/A | ⚠️ Partial | N/A |
| **Speed** | ~400ms | ~15s | ~minutes | ~75s |

---

## Arcis MPC Circuit Reference

### Data Structures

```rust
// User's encrypted trading strategy
pub struct SwapBounds {
    pub min_out: u64,           // Minimum output (slippage protection)
    pub max_slippage_bps: u16,  // Max slippage in basis points
    pub aggressive: bool,        // Aggressive execution flag
}

// Encrypted swap amount (NEW!)
pub struct SwapInput {
    pub amount: u64,            // Amount to swap (HIDDEN)
}

// MXE-only vault state
pub struct VaultState {
    pub pending_deposits: u64,
    pub total_liquidity: u64,
    pub total_deposited: u64,
}

// MXE-only user position
pub struct UserPosition {
    pub deposited: u64,
    pub lp_share: u64,
}
```

### Encryption Types

| Type | Description | Who Can Decrypt |
|------|-------------|-----------------|
| `Enc<Shared, T>` | Client + MXE encrypted | User who created it + MXE |
| `Enc<Mxe, T>` | MXE-only encrypted | Only MXE (protocol state) |
| Plaintext | Public parameter | Everyone |

---

## Glossary

| Term | Definition |
|------|------------|
| **Commitment** | `Poseidon(secret, nullifier_secret, amount)` - hides deposit details |
| **Nullifier** | `Poseidon(nullifier_secret)` - unique ID revealed at withdrawal |
| **Merkle Path** | Sibling hashes needed to compute root from leaf |
| **Anonymity Set** | Number of deposits that could plausibly be the source of a withdrawal |
| **MXE** | Multi-Party Execution - Arcium's encrypted computation environment |
| **Arcis** | Arcium's Rust DSL for writing MPC circuits |
| **Sunspot** | Noir verifier program deployed on Solana |
| **PDA** | Program Derived Address - deterministic Solana account |
| **Enc<Shared, T>** | Encrypted type decryptable by user + MXE |
| **Enc<Mxe, T>** | Encrypted type decryptable only by MXE |

---

## References

- [Noir Language Documentation](https://noir-lang.org/docs)
- [Arcium Developer Documentation](https://docs.arcium.com)
- [Tornado Cash Whitepaper](https://tornado.cash/Tornado.cash_whitepaper_v1.4.pdf)
- [Solana Program Library](https://spl.solana.com)
- [Poseidon Hash Function](https://www.poseidon-hash.info)
- [Jupiter Aggregator](https://station.jup.ag/docs)

---

*Last Updated: February 2026*
*Protocol Version: 0.2.0*
