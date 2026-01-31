# Zyncx Privacy Protocol - Roadmap & Integration Guide

## 📋 Project Overview

Zyncx is a privacy-preserving DeFi protocol on Solana that enables:
- **Shielded Deposits/Withdrawals** - Hide transaction amounts and participants
- **Private Swaps** - Execute trades without revealing strategy/slippage
- **ZK Proofs** - Noir circuits for ownership verification
- **Confidential Computation** - Arcium MXE for encrypted trading logic

---

## ✅ Completed Components

### Phase 1: Shielding & Unshielding
| Component | Status | Location |
|-----------|--------|----------|
| Noir ZK Circuit | ✅ Done | `mixer/src/main.nr` |
| Compiled Verifier | ✅ Done | `mixer/target/mixer.so` |
| Merkle Tree (Poseidon) | ✅ Done | `contracts/solana/zyncx/src/state/merkle_tree.rs` |
| Vault State | ✅ Done | `contracts/solana/zyncx/src/state/vault.rs` |
| Nullifier PDAs | ✅ Done | `contracts/solana/zyncx/src/state/nullifier.rs` |
| Deposit Instructions | ✅ Done | `contracts/solana/zyncx/src/instructions/deposit.rs` |
| Withdraw Instructions | ✅ Done | `contracts/solana/zyncx/src/instructions/withdraw.rs` |
| Jupiter DEX Integration | ✅ Done | `contracts/solana/zyncx/src/dex/jupiter.rs` |

### Phase 2: Arcium MXE Integration (PAUSED - SDK Compatibility Issues)
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

### High Priority

#### 1. Deploy Noir Verifier Program
```bash
# Compile the updated multi-token circuit
cd mixer && nargo compile && nargo codegen

# Deploy verifier to Solana
solana program deploy mixer/target/mixer.so --program-id mixer/target/mixer-keypair.json
```

#### 2. Update Frontend for Multi-Token
- Add token selector (SOL, USDC, etc.) to deposit/withdraw UI
- Update commitment generation to include `token_mint`
- Add cross-token swap UI with slippage settings
- Store token_mint in local note data

#### 3. Integrate groth16-solana Verifier
```bash
# The ZK proof verification is currently a placeholder
# Need to implement actual Groth16 verification on-chain
# Options: groth16-solana crate or Sunspot program
```

### Medium Priority

#### 4. Fix Arcium SDK Integration
The Arcium SDK (0.6.3) has compatibility issues with Anchor 0.32.1:
- `ArciumDeserialize` trait conflicts
- `comp_def_offset` macro import issues
- Callback output type serialization problems

**When Arcium SDK stabilizes, re-enable:**
```rust
// In lib.rs
use arcium_anchor::prelude::*;
#[arcium_program]

// In instructions/mod.rs
pub mod arcium_mxe;
pub use arcium_mxe::*;
```

#### 5. Deploy to Devnet
```bash
anchor deploy --provider.cluster devnet
```

#### 6. Pyth Oracle Integration
- [ ] Add real Pyth price feed account addresses
- [ ] Implement price staleness checks
- [ ] Use oracle prices for swap slippage validation

#### 6. Pyth Oracle Integration
- [ ] Add real Pyth price feed account addresses
- [ ] Implement price staleness checks

### Low Priority

#### 7. Advanced Features
- [ ] Confidential limit orders
- [ ] Confidential DCA
- [ ] Relayer network

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
import { poseidon } from 'circomlibjs';
import { randomBytes } from 'crypto';

// Generate deposit secrets
export function generateDepositSecrets() {
  const secret = randomBytes(32);
  const nullifierSecret = randomBytes(32);
  return { secret, nullifierSecret };
}

// Compute commitment = Poseidon(secret, nullifier_secret, amount, token_mint)
// NEW: Now includes token_mint for multi-token support
export function computeCommitment(
  secret: Uint8Array,
  nullifierSecret: Uint8Array,
  amount: bigint,
  tokenMint: Uint8Array  // NEW: 32-byte token mint address
): Uint8Array {
  const hash = poseidon([
    BigInt('0x' + Buffer.from(secret).toString('hex')),
    BigInt('0x' + Buffer.from(nullifierSecret).toString('hex')),
    amount,
    BigInt('0x' + Buffer.from(tokenMint).toString('hex'))  // NEW
  ]);
  return bigintToBytes32(hash);
}

// Compute nullifier hash = Poseidon(nullifier_secret)
export function computeNullifierHash(nullifierSecret: Uint8Array): Uint8Array {
  const hash = poseidon([
    BigInt('0x' + Buffer.from(nullifierSecret).toString('hex'))
  ]);
  return bigintToBytes32(hash);
}

// Compute precommitment for deposit (amount and token bound later)
export function computePrecommitment(
  secret: Uint8Array,
  nullifierSecret: Uint8Array
): Uint8Array {
  const hash = poseidon([
    BigInt('0x' + Buffer.from(secret).toString('hex')),
    BigInt('0x' + Buffer.from(nullifierSecret).toString('hex'))
  ]);
  return bigintToBytes32(hash);
}

function bigintToBytes32(n: bigint): Uint8Array {
  const hex = n.toString(16).padStart(64, '0');
  return Uint8Array.from(Buffer.from(hex, 'hex'));
}
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

export interface WithdrawProofInputs {
  // Private inputs
  secret: Uint8Array;
  nullifierSecret: Uint8Array;
  originalAmount: bigint;          // Full commitment amount
  tokenMint: Uint8Array;           // NEW: Token mint address
  merklePath: Uint8Array[];
  pathIndices: number[];
  newSecret: Uint8Array;           // For partial withdrawal
  newNullifierSecret: Uint8Array;  // For partial withdrawal
  // Public inputs
  root: Uint8Array;
  nullifierHash: Uint8Array;
  recipient: Uint8Array;
  withdrawAmount: bigint;
  newCommitment: Uint8Array;       // Change commitment (or zeros)
  tokenMintPublic: Uint8Array;     // NEW: Must match private tokenMint
}

export async function generateWithdrawProof(inputs: WithdrawProofInputs): Promise<Uint8Array> {
  const { noir, backend } = await initProver();
  
  const witnessInputs = {
    // Private
    secret: Array.from(inputs.secret),
    nullifier_secret: Array.from(inputs.nullifierSecret),
    merkle_path: inputs.merklePath.map(p => Array.from(p)),
    path_indices: inputs.pathIndices,
    // Public
    root: Array.from(inputs.root),
    nullifier_hash: Array.from(inputs.nullifierHash),
    recipient: Array.from(inputs.recipient),
    amount: inputs.amount.toString(),
  };

  const { witness } = await noir!.execute(witnessInputs);
  const proof = await backend!.generateProof(witness);
  
  return proof.proof;
}
```

### 4. Deposit Flow

```typescript
// hooks/useDeposit.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Transaction, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { getProgram, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA } from '../lib/program';
import { generateDepositSecrets, computePrecommitment } from '../lib/crypto';

export function useDeposit() {
  const { connection } = useConnection();
  const wallet = useWallet();

  async function depositNative(amountSol: number) {
    if (!wallet.publicKey) throw new Error('Wallet not connected');

    const program = getProgram(/* provider */);
    const amountLamports = amountSol * LAMPORTS_PER_SOL;

    // Generate secrets (SAVE THESE - needed for withdrawal!)
    const { secret, nullifierSecret } = generateDepositSecrets();
    const precommitment = computePrecommitment(secret, nullifierSecret);

    // Derive PDAs
    const NATIVE_MINT = new PublicKey(new Uint8Array(32)); // Zero pubkey for SOL
    const [vault] = getVaultPDA(NATIVE_MINT);
    const [merkleTree] = getMerkleTreePDA(vault);
    const [vaultTreasury] = getVaultTreasuryPDA(vault);

    // Build transaction
    const tx = await program.methods
      .depositNative(new BN(amountLamports), Array.from(precommitment))
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

export function useWithdraw() {
  const { connection } = useConnection();
  const wallet = useWallet();

  async function withdrawNative(
    secretHex: string,
    nullifierSecretHex: string,
    amount: number,
    recipient: PublicKey
  ) {
    if (!wallet.publicKey) throw new Error('Wallet not connected');

    await initProver();
    const program = getProgram(/* provider */);

    const secret = Uint8Array.from(Buffer.from(secretHex, 'hex'));
    const nullifierSecret = Uint8Array.from(Buffer.from(nullifierSecretHex, 'hex'));
    const amountBigInt = BigInt(amount);

    // Compute commitment and nullifier
    const commitment = computeCommitment(secret, nullifierSecret, amountBigInt);
    const nullifierHash = computeNullifierHash(nullifierSecret);

    // Fetch merkle tree and compute path
    const NATIVE_MINT = new PublicKey(new Uint8Array(32));
    const [vault] = getVaultPDA(NATIVE_MINT);
    const [merkleTree] = getMerkleTreePDA(vault);
    
    const merkleTreeAccount = await program.account.merkleTreeState.fetch(merkleTree);
    const { path, indices } = computeMerklePath(merkleTreeAccount, commitment);

    // Generate ZK proof
    const proof = await generateWithdrawProof({
      secret,
      nullifierSecret,
      merklePath: path,
      pathIndices: indices,
      root: Uint8Array.from(merkleTreeAccount.root),
      nullifierHash,
      recipient: recipient.toBytes(),
      amount: amountBigInt,
    });

    // Generate new commitment for change (if any)
    const newCommitment = new Uint8Array(32); // Or compute if splitting

    // Derive nullifier PDA
    const [nullifierPDA] = getNullifierPDA(vault, nullifierHash);
    const [vaultTreasury] = getVaultTreasuryPDA(vault);

    // Build transaction
    const tx = await program.methods
      .withdrawNative(
        new BN(amount),
        Array.from(nullifierHash),
        Array.from(newCommitment),
        Buffer.from(proof)
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

    return { txSignature: signature };
  }

  return { withdrawNative };
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

### Deposit Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                         DEPOSIT FLOW                            │
└─────────────────────────────────────────────────────────────────┘

User                    Frontend                  Solana Program
 │                         │                           │
 │ 1. Enter amount         │                           │
 ├────────────────────────>│                           │
 │                         │                           │
 │                         │ 2. Generate secrets       │
 │                         │    (secret, nullifier)    │
 │                         │                           │
 │                         │ 3. Compute precommitment  │
 │                         │    = Poseidon(secrets)    │
 │                         │                           │
 │                         │ 4. deposit_native()       │
 │                         ├──────────────────────────>│
 │                         │                           │
 │                         │                           │ 5. Transfer SOL
 │                         │                           │    to vault_treasury
 │                         │                           │
 │                         │                           │ 6. Compute commitment
 │                         │                           │    = Poseidon(amount, precommitment)
 │                         │                           │
 │                         │                           │ 7. Insert commitment
 │                         │                           │    into Merkle tree
 │                         │                           │
 │                         │      8. Return commitment │
 │                         │<──────────────────────────│
 │                         │                           │
 │ 9. Save "note" locally  │                           │
 │    (secrets + amount)   │                           │
 │<────────────────────────│                           │
 │                         │                           │
```

### Withdraw Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                        WITHDRAW FLOW                            │
└─────────────────────────────────────────────────────────────────┘

User                    Frontend                  Solana Program     Verifier
 │                         │                           │                │
 │ 1. Enter note           │                           │                │
 │    (secrets + amount)   │                           │                │
 ├────────────────────────>│                           │                │
 │                         │                           │                │
 │                         │ 2. Compute nullifier_hash │                │
 │                         │    = Poseidon(nullifier)  │                │
 │                         │                           │                │
 │                         │ 3. Fetch Merkle tree      │                │
 │                         ├──────────────────────────>│                │
 │                         │<──────────────────────────│                │
 │                         │                           │                │
 │                         │ 4. Compute Merkle path    │                │
 │                         │                           │                │
 │                         │ 5. Generate ZK proof      │                │
 │                         │    (Noir circuit)         │                │
 │                         │                           │                │
 │                         │ 6. withdraw_native()      │                │
 │                         ├──────────────────────────>│                │
 │                         │                           │                │
 │                         │                           │ 7. CPI: Verify │
 │                         │                           ├───────────────>│
 │                         │                           │                │
 │                         │                           │ 8. Proof valid │
 │                         │                           │<───────────────│
 │                         │                           │                │
 │                         │                           │ 9. Init nullifier PDA
 │                         │                           │    (prevents double-spend)
 │                         │                           │                │
 │                         │                           │ 10. Transfer SOL
 │                         │                           │     to recipient
 │                         │                           │                │
 │ 11. Funds received!     │                           │                │
 │<────────────────────────│                           │                │
```

### Confidential Swap Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                   CONFIDENTIAL SWAP FLOW                        │
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

## 📚 Resources

- [Noir Documentation](https://noir-lang.org/docs)
- [Arcium Developer Docs](https://docs.arcium.com)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Pyth Network](https://pyth.network/developers)
- [Jupiter Aggregator](https://station.jup.ag/docs)
- [Light Protocol (ZK Compression)](https://www.lightprotocol.com/)
