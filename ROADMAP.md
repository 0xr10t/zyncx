# Zyncx Privacy Protocol - Complete Integration Guide

> **Last Updated:** Core ZK circuits and Solana program working. Arcium integration blocked by Rust version.

## 📋 Project Overview

Zyncx is a privacy-preserving DeFi protocol on Solana that enables:
- **Shielded Deposits/Withdrawals** - Hide transaction amounts and participants
- **Private Swaps** - Execute trades without revealing strategy/slippage  
- **ZK Proofs** - Noir circuits for ownership verification
- **Confidential Computation** - Arcium MXE for encrypted trading logic (BLOCKED)

---

## 🚨 CRITICAL BLOCKERS

### Arcium SDK Dependency Issue

**Status:** ❌ BLOCKED - Cannot build with Arcium SDK

**Root Cause:**
```
The Arcium SDK (arcium-anchor = "0.6.3") depends on:
  → time-core v0.1.8
    → Requires Rust edition 2024
      → Requires Rust toolchain 1.85.0+

Current Rust version: 1.79.0 (installed with Anchor)
Required Rust version: 1.85.0+
```

**Error When Uncommenting Arcium:**
```
error: failed to parse manifest at ~/.cargo/registry/src/.../time-core-0.1.8/Cargo.toml
Caused by: failed to parse the `edition` key
  feature `edition2024` is required
```

**Resolution Options:**
1. **Wait for Arcium SDK update** - They need to pin `time-core` to `<0.1.8`
2. **Upgrade Rust to 1.85+** - May break Anchor/Solana BPF compatibility
3. **Fork Arcium SDK** - Patch the `time` dependency locally

**Impact:** Confidential swaps via MXE are unavailable. Basic deposits/withdrawals work fine.

---

## ✅ Completed Components

### Phase 1: Shielding & Unshielding
| Component | Status | Location |
|-----------|--------|----------|
| Noir ZK Circuit | ✅ Done | `mixer/src/main.nr` |
| Compiled Verifier | ✅ Done | `mixer/target/mixer.so` |
| Merkle Tree (keccak256) | ✅ Done | `contracts/solana/zyncx/src/state/merkle_tree.rs` |
| Vault State | ✅ Done | `contracts/solana/zyncx/src/state/vault.rs` |
| Nullifier PDAs | ✅ Done | `contracts/solana/zyncx/src/state/nullifier.rs` |
| Deposit Instructions | ✅ Done | `contracts/solana/zyncx/src/instructions/deposit.rs` |
| Withdraw Instructions | ✅ Done | `contracts/solana/zyncx/src/instructions/withdraw.rs` |
| Jupiter DEX Integration | ✅ Done | `contracts/solana/zyncx/src/dex/jupiter.rs` |

### Phase 2: Arcium MXE Integration (❌ BLOCKED - SDK Compatibility)
| Component | Status | Location |
|-----------|--------|---------|
| Arcis MPC Circuits | ✅ Done | `encrypted-ixs/src/lib.rs` |
| Encrypted State Accounts | ✅ Done | `contracts/solana/zyncx/src/state/arcium_mxe.rs` |
| Computation Def Initializers | ⏸️ Paused | Disabled until SDK stabilizes |
| Queue Encrypted Deposit | ⏸️ Paused | `arcium_mxe.rs` commented out |
| Queue Confidential Swap MXE | ⏸️ Paused | `arcium_mxe.rs` commented out |
| Anchor 0.32.1 | ✅ Done | `Cargo.toml` updated |
| Arcium 0.6.3 | ⚠️ Issues | Dependency conflicts, module disabled |

### Phase 3: Multi-Token Cross-Vault Swaps (NEW - COMPLETED)
| Component | Status | Location |
|-----------|--------|---------|
| Multi-token commitment format | ✅ Done | `mixer/src/main.nr` - `hash_4` with token_mint |
| Cross-token swap circuit | ✅ Done | `swap_circuit()` in `main.nr` |
| Dual-vault nullifier system | ✅ Done | Nullify in source, commit in destination |
| Cross-token swap instruction | ✅ Done | `cross_token_swap` in `lib.rs` |
| Slippage protection in circuit | ✅ Done | `assert(dst_amount >= min_dst_amount)` |
| 9 Noir tests passing | ✅ Done | `nargo test` |

### Arcium Integration Architecture
```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ARCIUM THREE-INSTRUCTION PATTERN                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. INIT COMP DEF (one-time per circuit)                               │
│     ┌──────────────────────────────────────────────────────────────┐   │
│     │ init_vault_comp_def / init_deposit_comp_def / init_swap_comp_def  │
│     │ - Registers circuit hash with MXE                            │   │
│     │ - Points to off-chain circuit URL (GitHub raw)               │   │
│     └──────────────────────────────────────────────────────────────┘   │
│                              │                                         │
│                              ▼                                         │
│  2. QUEUE COMPUTATION (per user operation)                             │
│     ┌──────────────────────────────────────────────────────────────┐   │
│     │ queue_encrypted_deposit / queue_confidential_swap_mxe        │   │
│     │ - ArgBuilder constructs encrypted payload:                   │   │
│     │   • .x25519_pubkey(user_pubkey)     Enc<Shared,T> input      │   │
│     │   • .plaintext_u128(nonce)          Client encryption nonce  │   │
│     │   • .encrypted_u64(ciphertext)      User's encrypted data    │   │
│     │   • .account(key, offset, size)     Enc<Mxe,T> on-chain state│   │
│     │ - Registers callback instruction                             │   │
│     └──────────────────────────────────────────────────────────────┘   │
│                              │                                         │
│                     ARX MPC NODES PROCESS                              │
│                              │                                         │
│                              ▼                                         │
│  3. CALLBACK (invoked by MXE after computation)                        │
│     ┌──────────────────────────────────────────────────────────────┐   │
│     │ deposit_callback / confidential_swap_callback_mxe            │   │
│     │ - Receives SignedComputationOutputs<T>                       │   │
│     │ - Verifies cluster signature                                 │   │
│     │ - Updates encrypted state accounts:                          │   │
│     │   • vault.vault_state = tuple.field_0.ciphertexts            │   │
│     │   • user_position.position_state = tuple.field_1.ciphertexts │   │
│     └──────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 🔧 Remaining Integration Tasks

### 🔴 HIGH PRIORITY (Required for MVP)

#### 1. Deploy Sunspot Verifier
**Status:** 🟡 Ready to deploy - verifier files exist in `mixer/target/`

You're using **Sunspot** to compile the Noir circuit into a Solana verifier program. This is the correct approach - no separate Groth16 implementation needed!

**What Sunspot does:**
- Compiles your Noir circuit (`mixer/src/main.nr`) 
- Generates a Solana program that can verify proofs on-chain
- Outputs: `mixer.vk` (verification key), `mixer.pk` (proving key), `mixer.so` (verifier program)

**Files already generated:**
```
mixer/target/
├── mixer.json      # Circuit ABI
├── mixer.ccs       # Compiled constraint system  
├── mixer.pk        # Proving key (used client-side)
├── mixer.vk        # Verification key (embedded in verifier)
└── mixer-keypair.json  # Program keypair for deployment
```

**Deployment Steps:**
```bash
# 1. Compile circuit (if not already done)
cd mixer && nargo compile

# 2. Generate Solana verifier with Sunspot
sunspot build

# 3. Deploy verifier to devnet
solana program deploy mixer/target/mixer.so \
  --program-id mixer/target/mixer-keypair.json \
  --url devnet

# 4. Note the deployed program ID and update Zyncx program
```

**CPI Integration:**
The Zyncx program already has CPI calls to the verifier. Just update `VERIFIER_PROGRAM_ID` with your deployed address.

#### 2. Frontend Crypto Implementation
**Status:** 🟡 Partially done - needs Poseidon WASM library

Need JavaScript/TypeScript Poseidon that matches Noir's BN254 curve:
```bash
cd app
yarn add poseidon-lite  # or circomlibjs
yarn add @noir-lang/noir_js @noir-lang/backend_barretenberg
```

**Files to create:**
- `app/lib/crypto.ts` - Commitment and nullifier computation
- `app/lib/prover.ts` - Noir proof generation in browser
- `app/lib/noteStorage.ts` - Encrypted local storage for notes

#### 3. Merkle Path Computation
**Status:** ❌ Not implemented

Need to compute sibling path for a given commitment:
```typescript
// app/lib/merkle.ts
export function computeMerklePath(
  leaves: Uint8Array[],
  commitment: Uint8Array
): { path: Uint8Array[], indices: number[] } {
  // 1. Find commitment index in leaves
  // 2. Walk up 20 levels, collecting sibling hashes
  // 3. Return path and left/right indicators
}
```

#### 4. Note Management UI
**Status:** ❌ Not implemented

Users must save their deposit notes. Need:
- Download note as JSON file
- Import note for withdrawal
- Optional: Encrypted browser storage
- Warning modals about losing funds

---

### 🟡 MEDIUM PRIORITY (Required for Production)

#### 5. Arcium SDK Fix
**Status:** ❌ BLOCKED by Rust 1.85+ requirement

**When Arcium releases a compatible version:**
```toml
# Uncomment in Cargo.toml:
[dependencies]
arcium-anchor = "0.6.x"  # When compatible version released
arcium-client = "0.6.x"
arcium-macros = "0.6.x"
```

```rust
// Re-enable in lib.rs:
mod instructions;
use instructions::arcium_mxe::*;

#[arcium_program]
#[program]
pub mod zyncx { ... }
```

#### 6. Devnet Deployment
```bash
# 1. Build programs
anchor build --no-idl

# 2. Deploy verifier
solana program deploy mixer/target/verifier.so --url devnet

# 3. Deploy main program  
anchor deploy --provider.cluster devnet

# 4. Initialize vaults
npx ts-node scripts/init-vaults.ts
```

#### 7. Pyth Oracle Integration
For swap slippage protection:
```rust
// Add real Pyth price feed addresses
const PYTH_SOL_USD: Pubkey = pubkey!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
const PYTH_USDC_USD: Pubkey = pubkey!("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD");
```

---

### 🟢 LOW PRIORITY (Future Features)

#### 8. Confidential Limit Orders
- Encrypt price threshold with Arcium
- Keeper network monitors and executes
- Uses same nullifier pattern

#### 9. Relayer Network
- Allow gasless withdrawals
- Relayer pays gas, deducts fee from withdrawal
- Prevents address linking

#### 10. Privacy Set Analytics
- Track anonymity set size
- Show privacy score to users
- Recommend optimal deposit sizes

---

## 📋 Quick Reference: Function Call Summary

### Contract Functions (Anchor)

| Function | Purpose | Key Parameters |
|----------|---------|----------------|
| `initialize_vault` | Create new token vault | `asset_mint`, `merkle_depth` |
| `deposit_native` | Deposit SOL | `amount`, `commitment` (32 bytes) |
| `deposit_spl` | Deposit SPL tokens | `amount`, `commitment`, `token_mint` |
| `withdraw_native` | Withdraw SOL | `amount`, `nullifier_hash`, `new_commitment`, `proof` |
| `withdraw_spl` | Withdraw SPL tokens | Same as native + `token_mint` |
| `cross_token_swap` | Swap between vaults | `src_proof`, `dst_proof`, `amounts` |

### Frontend Functions

| Function | Location | Purpose |
|----------|----------|---------|
| `computeCommitment()` | `lib/crypto.ts` | Poseidon(secret, nullifier, amount, token) |
| `computeNullifierHash()` | `lib/crypto.ts` | Poseidon(nullifier_secret) |
| `generateWithdrawProof()` | `lib/prover.ts` | Create Groth16 proof |
| `computeMerklePath()` | `lib/merkle.ts` | Get 20 sibling hashes |
| `depositNative()` | `hooks/useDeposit.ts` | Execute deposit transaction |
| `withdrawNative()` | `hooks/useWithdraw.ts` | Execute withdrawal transaction |

### Noir Circuit (Public Inputs)

```noir
fn main(
    // 6 PUBLIC INPUTS (passed to Solana verifier):
    pub root: Field,           // Current Merkle root
    pub nullifier_hash: Field, // Poseidon(nullifier_secret)  
    pub recipient: Field,      // 32-byte recipient address
    pub amount: Field,         // Withdrawal amount
    pub new_commitment: Field, // Change commitment (or 0)
    pub token_mint: Field,     // Token mint address
    
    // Private inputs (only prover knows):
    secret: Field,
    nullifier_secret: Field,
    // ... merkle path, etc
)
```

---

## 🖥️ Frontend Integration Guide

### Prerequisites
```bash
cd app
yarn add @coral-xyz/anchor @solana/web3.js @solana/wallet-adapter-react
yarn add @pythnetwork/client  # For price feeds
```

### 1. Program Setup

```typescript
// lib/program.ts
import { Program, AnchorProvider, Idl } from '@coral-xyz/anchor';
import { Connection, PublicKey } from '@solana/web3.js';
import idl from '../target/idl/zyncx.json';

export const PROGRAM_ID = new PublicKey('6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6');
export const VERIFIER_PROGRAM_ID = new PublicKey('YOUR_VERIFIER_PROGRAM_ID');

export function getProgram(provider: AnchorProvider): Program {
  return new Program(idl as Idl, PROGRAM_ID, provider);
}

// PDA derivations
export function getVaultPDA(assetMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), assetMint.toBuffer()],
    PROGRAM_ID
  );
}

export function getMerkleTreePDA(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('merkle_tree'), vault.toBuffer()],
    PROGRAM_ID
  );
}

export function getNullifierPDA(vault: PublicKey, nullifier: Uint8Array): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('nullifier'), vault.toBuffer(), nullifier],
    PROGRAM_ID
  );
}

export function getVaultTreasuryPDA(vault: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault_treasury'), vault.toBuffer()],
    PROGRAM_ID
  );
}
```

### 2. Cryptographic Utilities

```typescript
// lib/crypto.ts
import { poseidonHash } from 'poseidon-lite'; // or circomlibjs
import { randomBytes } from 'crypto';

// Generate deposit secrets
export function generateDepositSecrets() {
  const secret = randomBytes(32);
  const nullifierSecret = randomBytes(32);
  return { secret, nullifierSecret };
}

// ============================================================================
// IMPORTANT: Commitment Format (MUST match Noir circuit exactly!)
// ============================================================================
// commitment = Poseidon(secret, nullifier_secret, amount, token_mint)
//
// This 4-input hash is computed CLIENT-SIDE because:
// 1. On-chain Poseidon causes stack overflow (uses 11KB, Solana limit is 4KB)
// 2. The commitment binds the deposit to a specific token from the start
// ============================================================================

export function computeCommitment(
  secret: Uint8Array,        // 32 bytes - private, user keeps this
  nullifierSecret: Uint8Array, // 32 bytes - private, user keeps this  
  amount: bigint,            // deposit amount in lamports/token units
  tokenMint: Uint8Array      // 32 bytes - the token mint pubkey
): Uint8Array {
  // Convert to field elements (Noir uses BN254 field)
  const secretField = bytesToField(secret);
  const nullifierField = bytesToField(nullifierSecret);
  const amountField = amount;
  const tokenMintField = bytesToField(tokenMint);
  
  // Poseidon hash with 4 inputs (matches hash_4 in Noir)
  const hash = poseidonHash([
    secretField,
    nullifierField, 
    amountField,
    tokenMintField
  ]);
  
  return fieldToBytes32(hash);
}

// Compute nullifier hash = Poseidon(nullifier_secret)
// This is what gets stored on-chain to prevent double-spend
export function computeNullifierHash(nullifierSecret: Uint8Array): Uint8Array {
  const hash = poseidonHash([bytesToField(nullifierSecret)]);
  return fieldToBytes32(hash);
}

// Helper: Convert 32-byte array to BN254 field element
function bytesToField(bytes: Uint8Array): bigint {
  return BigInt('0x' + Buffer.from(bytes).toString('hex')) % BN254_FIELD_MODULUS;
}

// Helper: Convert field element to 32-byte array
function fieldToBytes32(n: bigint): Uint8Array {
  const hex = n.toString(16).padStart(64, '0');
  return Uint8Array.from(Buffer.from(hex, 'hex'));
}

const BN254_FIELD_MODULUS = BigInt('21888242871839275222246405745257275088548364400416034343698204186575808495617');

// ============================================================================
// NOTE: The Solana program uses keccak256 for on-chain Merkle tree hashing
// This is different from Poseidon used in the ZK circuit
// The two hash functions serve different purposes:
// - Poseidon: Used in ZK proofs (efficient in circuits)
// - keccak256: Used on-chain for Merkle tracking (native Solana support)
// ============================================================================
```

### 3. Noir Proof Generation

```typescript
// lib/prover.ts
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import circuit from '../../mixer/target/mixer.json';

let noir: Noir | null = null;
let backend: BarretenbergBackend | null = null;

export async function initProver() {
  if (!noir) {
    backend = new BarretenbergBackend(circuit);
    noir = new Noir(circuit, backend);
  }
  return { noir, backend };
}

// ============================================================================
// WITHDRAW PROOF INPUTS
// The Noir circuit expects these 6 PUBLIC inputs (in order):
// 1. root            - Current Merkle root
// 2. nullifier_hash  - Poseidon(nullifier_secret)
// 3. recipient       - 32-byte recipient address
// 4. amount          - Amount to withdraw
// 5. new_commitment  - Change commitment (or zeros for full withdrawal)
// 6. token_mint      - Token mint address (must match deposit)
// ============================================================================

export interface WithdrawProofInputs {
  // === PRIVATE INPUTS (known only to user) ===
  secret: Uint8Array;              // Original deposit secret
  nullifierSecret: Uint8Array;     // Original nullifier secret
  originalAmount: bigint;          // Full committed amount
  tokenMint: Uint8Array;           // Token mint (32 bytes)
  merklePath: Uint8Array[];        // 20 sibling hashes
  pathIndices: number[];           // 20 left/right indicators
  newSecret: Uint8Array;           // For partial withdrawal change
  newNullifierSecret: Uint8Array;  // For partial withdrawal change
  
  // === PUBLIC INPUTS (sent to contract + verifier) ===
  root: Uint8Array;                // Current Merkle root
  nullifierHash: Uint8Array;       // Poseidon(nullifier_secret)
  recipient: Uint8Array;           // Recipient address
  withdrawAmount: bigint;          // Amount being withdrawn
  newCommitment: Uint8Array;       // Change commitment (zeros if full)
  tokenMintPublic: Uint8Array;     // Must match private tokenMint
}

export async function generateWithdrawProof(inputs: WithdrawProofInputs): Promise<{
  proof: Uint8Array;
  publicInputs: Uint8Array[];  // 6 public inputs for verifier
}> {
  const { noir, backend } = await initProver();
  
  // Build witness inputs for Noir circuit
  const witnessInputs = {
    // Private inputs
    secret: Array.from(inputs.secret),
    nullifier_secret: Array.from(inputs.nullifierSecret),
    original_amount: inputs.originalAmount.toString(),
    token_mint_private: Array.from(inputs.tokenMint),
    merkle_path: inputs.merklePath.map(p => Array.from(p)),
    path_indices: inputs.pathIndices,
    new_secret: Array.from(inputs.newSecret),
    new_nullifier_secret: Array.from(inputs.newNullifierSecret),
    
    // Public inputs
    root: Array.from(inputs.root),
    nullifier_hash: Array.from(inputs.nullifierHash),
    recipient: Array.from(inputs.recipient),
    amount: inputs.withdrawAmount.toString(),
    new_commitment: Array.from(inputs.newCommitment),
    token_mint: Array.from(inputs.tokenMintPublic),
  };

  const { witness } = await noir!.execute(witnessInputs);
  const proofData = await backend!.generateProof(witness);
  
  // Return both proof and the 6 public inputs
  return {
    proof: proofData.proof,
    publicInputs: [
      inputs.root,
      inputs.nullifierHash,
      inputs.recipient,
      bigintToBytes32(inputs.withdrawAmount),
      inputs.newCommitment,
      inputs.tokenMintPublic,
    ],
  };
}

function bigintToBytes32(n: bigint): Uint8Array {
  const hex = n.toString(16).padStart(64, '0');
  return Uint8Array.from(Buffer.from(hex, 'hex'));
}
```

### 4. Deposit Flow

```typescript
// hooks/useDeposit.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Transaction, SystemProgram, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { getProgram, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA } from '../lib/program';
import { generateDepositSecrets, computeCommitment } from '../lib/crypto';

// ============================================================================
// DEPOSIT FLOW
// 
// 1. Generate random secrets client-side
// 2. Compute full commitment = Poseidon(secret, nullifier_secret, amount, token_mint)
// 3. Send commitment to contract (NOT precommitment - amount is already bound!)
// 4. Contract inserts commitment into Merkle tree
// 5. USER MUST SAVE: secret, nullifierSecret, amount, tokenMint (needed to withdraw)
// ============================================================================

export function useDeposit() {
  const { connection } = useConnection();
  const wallet = useWallet();

  async function depositNative(amountSol: number) {
    if (!wallet.publicKey) throw new Error('Wallet not connected');

    const program = getProgram(/* provider */);
    const amountLamports = BigInt(Math.floor(amountSol * LAMPORTS_PER_SOL));

    // Generate secrets (SAVE THESE - needed for withdrawal!)
    const { secret, nullifierSecret } = generateDepositSecrets();
    
    // Token mint for native SOL is all zeros
    const NATIVE_MINT = new Uint8Array(32);
    
    // *** IMPORTANT CHANGE ***
    // Compute FULL commitment client-side (includes amount and token_mint)
    // This is required because on-chain Poseidon causes stack overflow
    const commitment = computeCommitment(secret, nullifierSecret, amountLamports, NATIVE_MINT);

    // Derive PDAs
    const [vault] = getVaultPDA(new PublicKey(NATIVE_MINT));
    const [merkleTree] = getMerkleTreePDA(vault);
    const [vaultTreasury] = getVaultTreasuryPDA(vault);

    // *** API CHANGE: Now sends `commitment` not `precommitment` ***
    const tx = await program.methods
      .depositNative(
        new BN(amountLamports.toString()), 
        Array.from(commitment)  // Full commitment with amount+token bound
      )
      .accounts({
        depositor: wallet.publicKey,
        vault,
        merkleTree,
        vaultTreasury,
        systemProgram: SystemProgram.programId,
      })
      .transaction();

    // Send transaction
    const signature = await wallet.sendTransaction(tx, connection);
    await connection.confirmTransaction(signature);

    // ============================================================================
    // CRITICAL: Return note data - user MUST save this to withdraw later!
    // Without these values, funds are PERMANENTLY LOCKED in the vault
    // ============================================================================
    return {
      secret: Buffer.from(secret).toString('hex'),
      nullifierSecret: Buffer.from(nullifierSecret).toString('hex'),
      amount: amountLamports.toString(),
      tokenMint: Buffer.from(NATIVE_MINT).toString('hex'),  // Include token mint
      leafIndex: /* fetch from event or return value */,
      txSignature: signature,
    };
  }

  // For SPL tokens, use depositSpl with token mint address
  async function depositSpl(tokenMint: PublicKey, amount: bigint) {
    // Similar to depositNative but uses token accounts
    // commitment = computeCommitment(secret, nullifierSecret, amount, tokenMint.toBytes())
  }

  return { depositNative, depositSpl };
}
```
        merkleTree,
        vaultTreasury,
        systemProgram: SystemProgram.programId,
      })
      .transaction();

    // Send transaction
    const signature = await wallet.sendTransaction(tx, connection);
    await connection.confirmTransaction(signature);

    // Return note data (user must save this!)
    return {
      secret: Buffer.from(secret).toString('hex'),
      nullifierSecret: Buffer.from(nullifierSecret).toString('hex'),
      amount: amountLamports,
      txSignature: signature,
    };
  }

  return { depositNative };
}
```

### 5. Withdraw Flow

```typescript
// hooks/useWithdraw.ts
import { generateWithdrawProof, initProver } from '../lib/prover';
import { computeNullifierHash, computeCommitment } from '../lib/crypto';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ============================================================================
// WITHDRAW FLOW
//
// 1. Load saved note (secret, nullifierSecret, amount, tokenMint)
// 2. Compute commitment from note data
// 3. Fetch Merkle tree and compute path to commitment
// 4. Generate ZK proof with 6 public inputs
// 5. Send transaction with proof + public inputs
// 6. Contract verifies proof and transfers funds
// ============================================================================

export function useWithdraw() {
  const { connection } = useConnection();
  const wallet = useWallet();

  async function withdrawNative(
    // Note data (loaded from user's saved deposit receipt)
    secretHex: string,
    nullifierSecretHex: string,
    savedAmount: string,        // Original deposit amount
    savedTokenMintHex: string,  // Original token mint
    // Withdrawal parameters
    withdrawAmount: string,     // Amount to withdraw (can be less than savedAmount)
    recipient: PublicKey
  ) {
    if (!wallet.publicKey) throw new Error('Wallet not connected');

    await initProver();
    const program = getProgram(/* provider */);

    // Parse note data
    const secret = Uint8Array.from(Buffer.from(secretHex, 'hex'));
    const nullifierSecret = Uint8Array.from(Buffer.from(nullifierSecretHex, 'hex'));
    const originalAmount = BigInt(savedAmount);
    const tokenMint = Uint8Array.from(Buffer.from(savedTokenMintHex, 'hex'));
    const withdrawAmountBigInt = BigInt(withdrawAmount);

    // Compute commitment (must match what was deposited)
    const commitment = computeCommitment(secret, nullifierSecret, originalAmount, tokenMint);
    const nullifierHash = computeNullifierHash(nullifierSecret);

    // Fetch current Merkle tree state
    const [vault] = getVaultPDA(new PublicKey(tokenMint));
    const [merkleTree] = getMerkleTreePDA(vault);
    const merkleTreeAccount = await program.account.merkleTreeState.fetch(merkleTree);
    
    // Compute Merkle path (20 siblings + indices)
    const { path, indices } = computeMerklePath(merkleTreeAccount, commitment);

    // Handle partial withdrawal (change commitment)
    let newCommitment = new Uint8Array(32); // All zeros = full withdrawal
    let newSecret = new Uint8Array(32);
    let newNullifierSecret = new Uint8Array(32);
    
    if (withdrawAmountBigInt < originalAmount) {
      // Partial withdrawal: create new commitment for change
      const changeAmount = originalAmount - withdrawAmountBigInt;
      newSecret = crypto.getRandomValues(new Uint8Array(32));
      newNullifierSecret = crypto.getRandomValues(new Uint8Array(32));
      newCommitment = computeCommitment(newSecret, newNullifierSecret, changeAmount, tokenMint);
      
      // *** IMPORTANT: Save new note for the change! ***
    }

    // Generate ZK proof
    const { proof, publicInputs } = await generateWithdrawProof({
      // Private inputs
      secret,
      nullifierSecret,
      originalAmount,
      tokenMint,
      merklePath: path,
      pathIndices: indices,
      newSecret,
      newNullifierSecret,
      // Public inputs
      root: Uint8Array.from(merkleTreeAccount.root),
      nullifierHash,
      recipient: recipient.toBytes(),
      withdrawAmount: withdrawAmountBigInt,
      newCommitment,
      tokenMintPublic: tokenMint,
    });

    // Derive PDAs
    const [nullifierPDA] = getNullifierPDA(vault, nullifierHash);
    const [vaultTreasury] = getVaultTreasuryPDA(vault);

    // ============================================================================
    // WITHDRAW TRANSACTION
    // The contract will:
    // 1. Verify the ZK proof against 6 public inputs
    // 2. Check nullifier hasn't been used (prevents double-spend)
    // 3. Create nullifier PDA (marks commitment as spent)
    // 4. Insert new_commitment to tree (if non-zero, for partial withdrawal)
    // 5. Transfer funds to recipient
    // ============================================================================
    const tx = await program.methods
      .withdrawNative(
        new BN(withdrawAmount),
        Array.from(nullifierHash),
        Array.from(newCommitment),
        Buffer.from(proof)
        // Note: token_mint is passed via remaining accounts or encoded in proof
      )
      .accounts({
        recipient,
        vault,
        merkleTree,
        vaultTreasury,
        nullifierAccount: nullifierPDA,
        verifierProgram: VERIFIER_PROGRAM_ID,
        payer: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .transaction();

    const signature = await wallet.sendTransaction(tx, connection);
    await connection.confirmTransaction(signature);

    // Return change note if partial withdrawal
    return { 
      txSignature: signature,
      changeNote: withdrawAmountBigInt < originalAmount ? {
        secret: Buffer.from(newSecret).toString('hex'),
        nullifierSecret: Buffer.from(newNullifierSecret).toString('hex'),
        amount: (originalAmount - withdrawAmountBigInt).toString(),
        tokenMint: savedTokenMintHex,
      } : null,
    };
  }

  return { withdrawNative };
}

// Helper: Compute Merkle path for a commitment
function computeMerklePath(
  treeAccount: { leaves: Uint8Array[], filledSubtrees: Uint8Array[] },
  commitment: Uint8Array
): { path: Uint8Array[], indices: number[] } {
  // Find leaf index
  const leafIndex = treeAccount.leaves.findIndex(
    leaf => Buffer.from(leaf).equals(Buffer.from(commitment))
  );
  if (leafIndex === -1) throw new Error('Commitment not found in tree');

  // Compute sibling path (20 levels)
  // This requires knowing the tree structure
  // Implementation depends on how leaves are stored
  
  return { path: [], indices: [] }; // TODO: Implement
}
```

### 6. Confidential Swap Flow

```typescript
// hooks/useConfidentialSwap.ts
import { encryptForArcium } from '../lib/arcium';

export function useConfidentialSwap() {
  async function queueConfidentialSwap(
    srcToken: PublicKey,
    dstToken: PublicKey,
    amount: number,
    minPrice: number,
    maxSlippage: number,
    secretHex: string,
    nullifierSecretHex: string
  ) {
    // 1. Create nullifier account first
    const nullifierHash = computeNullifierHash(nullifierSecret);
    await program.methods
      .createNullifier(Array.from(nullifierHash))
      .accounts({ /* ... */ })
      .rpc();

    // 2. Encrypt trading bounds for Arcium
    const encryptedBounds = await encryptForArcium({
      minPrice,
      maxSlippage,
      deadline: Date.now() + 300000, // 5 minutes
    });

    // 3. Generate ZK proof
    const proof = await generateWithdrawProof({ /* ... */ });

    // 4. Queue confidential swap
    const params = {
      srcToken,
      dstToken,
      amount: new BN(amount),
      encryptedBounds: Buffer.from(encryptedBounds),
      recipient: wallet.publicKey,
      nullifier: Array.from(nullifierHash),
      newCommitment: Array.from(newCommitment),
    };

    await program.methods
      .queueConfidentialSwap(params, Buffer.from(proof))
      .accounts({ /* ... */ })
      .rpc();

    // 5. Wait for Arcium callback (poll or websocket)
    // The swap executes automatically when Arcium calls back
  }

  return { queueConfidentialSwap };
}
```

---

## 🔄 Complete Workflow Diagrams

### Deposit Flow (UPDATED)
```
┌─────────────────────────────────────────────────────────────────┐
│                    DEPOSIT FLOW (Client-Side Commitment)        │
└─────────────────────────────────────────────────────────────────┘

User                    Frontend                  Solana Program
 │                         │                           │
 │ 1. Enter amount         │                           │
 │    + select token       │                           │
 ├────────────────────────>│                           │
 │                         │                           │
 │                         │ 2. Generate secrets       │
 │                         │    secret = random(32)    │
 │                         │    nullifier = random(32) │
 │                         │                           │
 │                         │ 3. Compute FULL commitment│
 │                         │    (CLIENT-SIDE!)         │
 │                         │    = Poseidon(            │
 │                         │        secret,            │
 │                         │        nullifier_secret,  │
 │                         │        amount,            │
 │                         │        token_mint         │
 │                         │      )                    │
 │                         │                           │
 │                         │ 4. deposit_native(        │
 │                         │      amount,              │
 │                         │      commitment  <-- FULL │
 │                         │    )                      │
 │                         ├──────────────────────────>│
 │                         │                           │
 │                         │                           │ 5. Transfer tokens
 │                         │                           │    to vault_treasury
 │                         │                           │
 │                         │                           │ 6. Insert commitment
 │                         │                           │    directly into
 │                         │                           │    Merkle tree
 │                         │                           │    (no hash on-chain!)
 │                         │                           │
 │                         │                           │ 7. Emit DepositEvent
 │                         │    8. Return leaf_index   │    {commitment,
 │                         │<──────────────────────────│     leaf_index,
 │                         │                           │     token_mint}
 │                         │                           │
 │ 9. SAVE NOTE LOCALLY!   │                           │
 │    {                    │                           │
 │      secret,            │                           │
 │      nullifierSecret,   │                           │
 │      amount,            │                           │
 │      tokenMint,         │                           │
 │      leafIndex          │                           │
 │    }                    │                           │
 │<────────────────────────│                           │
 │                         │                           │
 │ ⚠️ LOSE NOTE = LOSE FUNDS!                         │
```

### Withdraw Flow (UPDATED)
```
┌─────────────────────────────────────────────────────────────────┐
│                   WITHDRAW FLOW (6 Public Inputs)               │
└─────────────────────────────────────────────────────────────────┘

User                    Frontend                  Solana Program     Verifier
 │                         │                           │                │
 │ 1. Load saved note      │                           │                │
 │    {secret, nullifier,  │                           │                │
 │     amount, tokenMint}  │                           │                │
 ├────────────────────────>│                           │                │
 │                         │                           │                │
 │                         │ 2. Recompute commitment   │                │
 │                         │    from note data         │                │
 │                         │                           │                │
 │                         │ 3. Compute nullifier_hash │                │
 │                         │    = Poseidon(nullifier)  │                │
 │                         │                           │                │
 │                         │ 4. Fetch Merkle tree      │                │
 │                         ├──────────────────────────>│                │
 │                         │<──────────────────────────│                │
 │                         │                           │                │
 │                         │ 5. Compute Merkle path    │                │
 │                         │    (20 siblings)          │                │
 │                         │                           │                │
 │                         │ 6. Generate ZK proof      │                │
 │                         │    Public inputs:         │                │
 │                         │    [1] root               │                │
 │                         │    [2] nullifier_hash     │                │
 │                         │    [3] recipient          │                │
 │                         │    [4] amount             │                │
 │                         │    [5] new_commitment     │                │
 │                         │    [6] token_mint         │                │
 │                         │                           │                │
 │                         │ 7. withdraw_native(       │                │
 │                         │      amount,              │                │
 │                         │      nullifier_hash,      │                │
 │                         │      new_commitment,      │                │
 │                         │      proof                │                │
 │                         │    )                      │                │
 │                         ├──────────────────────────>│                │
 │                         │                           │                │
 │                         │                           │ 8. Build 6 public
 │                         │                           │    inputs array
 │                         │                           │                │
 │                         │                           │ 9. CPI: Verify │
 │                         │                           │    proof       │
 │                         │                           ├───────────────>│
 │                         │                           │                │
 │                         │                           │ 10. Valid?     │
 │                         │                           │<───────────────│
 │                         │                           │                │
 │                         │                           │ 11. Create nullifier
 │                         │                           │     PDA (prevents
 │                         │                           │     double-spend)
 │                         │                           │                │
 │                         │                           │ 12. If new_commitment
 │                         │                           │     != 0, insert to
 │                         │                           │     tree (change)
 │                         │                           │                │
 │                         │                           │ 13. Transfer tokens
 │                         │                           │     to recipient
 │                         │                           │                │
 │ 14. Funds received!     │                           │                │
 │<────────────────────────│                           │                │
```

### Confidential Swap Flow (❌ BLOCKED - Arcium SDK)
```
┌─────────────────────────────────────────────────────────────────┐
│        CONFIDENTIAL SWAP FLOW (Requires Arcium SDK)             │
└─────────────────────────────────────────────────────────────────┘

User          Frontend         Zyncx Program    Arcium MXE    Jupiter
 │               │                   │              │            │
 │ 1. Enter swap │                   │              │            │
 │    params     │                   │              │            │
 ├──────────────>│                   │              │            │
 │               │                   │              │            │
 │               │ 2. Encrypt bounds │              │            │
 │               │    (FHE cipher)   │              │            │
 │               │                   │              │            │
 │               │ 3. Generate proof │              │            │
 │               │                   │              │            │
 │               │ 4. create_nullifier()            │            │
 │               ├──────────────────>│              │            │
 │               │                   │              │            │
 │               │ 5. queue_confidential_swap()     │            │
 │               ├──────────────────>│              │            │
 │               │                   │              │            │
 │               │                   │ 6. Store request          │
 │               │                   │    Mark nullifier spent   │
 │               │                   │              │            │
 │               │                   │ 7. CPI: queue_computation │
 │               │                   ├─────────────>│            │
 │               │                   │              │            │
 │               │                   │              │ 8. MPC nodes
 │               │                   │              │    process
 │               │                   │              │    encrypted
 │               │                   │              │    strategy
 │               │                   │              │            │
 │               │                   │              │ 9. Compare
 │               │                   │              │    price vs
 │               │                   │              │    encrypted
 │               │                   │              │    bounds
 │               │                   │              │            │
 │               │                   │ 10. confidential_swap_callback()
 │               │                   │<─────────────│            │
 │               │                   │              │            │
 │               │                   │ 11. If approved:          │
 │               │                   │     CPI: Jupiter swap     │
 │               │                   ├─────────────────────────>│
 │               │                   │              │            │
 │               │                   │ 12. Insert new commitment │
 │               │                   │              │            │
 │ 13. Swap      │                   │              │            │
 │    complete!  │                   │              │            │
 │<──────────────│                   │              │            │
```

---

## 🚀 Deployment Checklist

### Devnet Deployment
```bash
# 1. Build programs
anchor build --no-idl

# 2. Deploy Noir verifier
solana program deploy mixer/target/mixer.so \
  --program-id mixer/target/mixer-keypair.json \
  --url devnet

# 3. Deploy Zyncx program
anchor deploy --provider.cluster devnet

# 4. Initialize vaults
# Run initialization script to create SOL vault, USDC vault, etc.
```

### Mainnet Deployment
- [ ] Complete security audit
- [ ] Deploy to mainnet-beta
- [ ] Initialize production vaults
- [ ] Set up monitoring and alerting
- [ ] Configure Arcium mainnet cluster

---

## 📊 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              ZYNCX ARCHITECTURE                         │
└─────────────────────────────────────────────────────────────────────────┘

                              ┌─────────────────┐
                              │   Next.js App   │
                              │   (Frontend)    │
                              └────────┬────────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
                    ▼                  ▼                  ▼
           ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
           │ Wallet Adapter│  │  Noir Prover  │  │Arcium Encrypt │
           │   (Phantom)   │  │  (WASM)       │  │   (FHE)       │
           └───────┬───────┘  └───────┬───────┘  └───────┬───────┘
                   │                  │                  │
                   └──────────────────┼──────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            SOLANA BLOCKCHAIN                            │
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │  Zyncx Program  │  │ Verifier Program│  │  Jupiter DEX    │         │
│  │                 │  │   (mixer.so)    │  │                 │         │
│  │  ┌───────────┐  │  │                 │  │                 │         │
│  │  │   Vaults  │  │  │  Groth16 BN254  │  │  Swap Router    │         │
│  │  │  ┌─────┐  │  │  │  Verification   │  │                 │         │
│  │  │  │ SOL │  │  │  │                 │  │                 │         │
│  │  │  ├─────┤  │  │  └─────────────────┘  └─────────────────┘         │
│  │  │  │USDC │  │  │                                                   │
│  │  │  └─────┘  │  │           ┌─────────────────┐                     │
│  │  └───────────┘  │           │   Pyth Oracle   │                     │
│  │                 │           │  (Price Feeds)  │                     │
│  │  ┌───────────┐  │           └─────────────────┘                     │
│  │  │  Merkle   │  │                                                   │
│  │  │   Trees   │  │                                                   │
│  │  └───────────┘  │                                                   │
│  │                 │                                                   │
│  │  ┌───────────┐  │                                                   │
│  │  │ Nullifier │  │                                                   │
│  │  │   PDAs    │  │                                                   │
│  │  └───────────┘  │                                                   │
│  └────────┬────────┘                                                   │
│           │                                                            │
└───────────┼────────────────────────────────────────────────────────────┘
            │
            │ CPI (Confidential Swaps)
            ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           ARCIUM MXE CLUSTER                            │
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │    Arx Node 1   │  │    Arx Node 2   │  │    Arx Node 3   │         │
│  │                 │  │                 │  │                 │         │
│  │   MPC + FHE     │  │   MPC + FHE     │  │   MPC + FHE     │         │
│  │   Computation   │  │   Computation   │  │   Computation   │         │
│  │                 │  │                 │  │                 │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                         │
│  • Receives encrypted trading bounds                                    │
│  • Fetches Pyth price (public)                                         │
│  • Computes: encrypted_bound vs public_price                           │
│  • Returns: approve/reject (threshold signed)                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 📝 Environment Variables

```bash
# .env.local (Frontend)
NEXT_PUBLIC_SOLANA_RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_PROGRAM_ID=6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6
NEXT_PUBLIC_VERIFIER_PROGRAM_ID=<VERIFIER_PROGRAM_ID>
NEXT_PUBLIC_ARCIUM_CLUSTER=<ARCIUM_MXE_ADDRESS>

# Anchor.toml
[programs.devnet]
zyncx = "6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6"
```

---

## � Troubleshooting Guide

### Build Errors

#### Arcium SDK "edition2024" Error
```
error: failed to parse manifest at time-core-0.1.8/Cargo.toml
feature `edition2024` is required
```
**Solution:** Arcium SDK 0.6.3 requires Rust 1.85+. Keep Arcium modules commented out until:
1. Arcium releases a compatible SDK version, OR
2. You upgrade to Rust 1.85+ (may break Anchor compatibility)

#### Poseidon Stack Overflow
```
Program failed to complete: exceeded CUs meter at BPF instruction
```
**Solution:** Poseidon hashing uses ~11KB stack, Solana limit is 4KB. Compute Poseidon commitments CLIENT-SIDE, not on-chain.

#### blake3 Compilation Error
```
error: no rules expected the token `=`
```
**Solution:** Pin blake3 version in `Cargo.toml`:
```toml
[patch.crates-io]
blake3 = { git = "https://github.com/oconnor663/blake3", tag = "1.6.1" }
```

### Runtime Errors

#### "Commitment not found in tree"
**Cause:** The commitment format doesn't match between deposit and withdraw.
**Solution:** Ensure both use `Poseidon(secret, nullifier_secret, amount, token_mint)` with the exact same values.

#### "Nullifier already spent"
**Cause:** Attempting to withdraw the same note twice.
**Solution:** Each note can only be withdrawn once. This is by design (prevents double-spend).

#### "Invalid proof"
**Cause:** Mismatch between Noir circuit and Solana verifier inputs.
**Solution:** Ensure verifier receives exactly 6 public inputs in order:
1. root, 2. nullifier_hash, 3. recipient, 4. amount, 5. new_commitment, 6. token_mint

### Testing

```bash
# Run Noir circuit tests
cd mixer && nargo test

# Run Anchor tests
anchor test

# Run single test
anchor test -- --test-threads=1

# Check for build errors
anchor build 2>&1 | grep -i error
```

---

## 📊 Current Project Status Summary

| Component | Status | Notes |
|-----------|--------|-------|
| Noir ZK Circuit | ✅ Working | 9 tests passing, multi-token support |
| Solana Program | ✅ Building | Anchor build succeeds |
| Merkle Tree | ✅ Working | keccak256 hashing via solana-keccak-hasher |
| Deposit Flow | ✅ Ready | Accepts full commitment from client |
| Withdraw Flow | ✅ Ready | Passes 6 public inputs to verifier |
| Sunspot Verifier | 🟡 Ready to deploy | Files in `mixer/target/`, needs deployment |
| Arcium Integration | ❌ Blocked | Needs Rust 1.85+ for time-core 0.1.8 |
| Frontend | 🟡 Partial | UI exists, crypto utilities needed |
| Devnet Deploy | ⬜ Not started | Ready to deploy when verifier done |

---

## 📚 Resources

- [Noir Documentation](https://noir-lang.org/docs)
- [Arcium Developer Docs](https://docs.arcium.com)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Pyth Network](https://pyth.network/developers)
- [Jupiter Aggregator](https://station.jup.ag/docs)
- [Light Protocol (ZK Compression)](https://www.lightprotocol.com/)
- [Sunspot](https://github.com/noir-lang/sunspot) - Noir to Solana verifier compiler
- [poseidon-lite](https://www.npmjs.com/package/poseidon-lite) - JavaScript Poseidon hash
- [solana-keccak-hasher](https://docs.rs/solana-keccak-hasher/latest/) - On-chain keccak256

---

## 📅 Session History

**Latest Session Summary:**
1. ✅ Fixed multi-token Noir circuit (9 tests passing)
2. ✅ Aligned commitment format between circuit and contract
3. ✅ Updated deposit to accept full commitment from client
4. ✅ Updated withdraw to pass 6 public inputs to verifier
5. ✅ Switched on-chain Merkle tree to keccak256 (solana-keccak-hasher)
6. ❌ Arcium blocked by Rust 1.85+ requirement
7. ✅ Anchor build succeeds with all changes
