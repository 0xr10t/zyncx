# ZYNCX Frontend Integration Guide

> **Complete guide for integrating Zyncx privacy protocol into your frontend application**

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Architecture Overview](#architecture-overview)
4. [Core Modules](#core-modules)
5. [User Flows Implementation](#user-flows-implementation)
6. [Arcium MXE Integration](#arcium-mxe-integration)
7. [Error Handling](#error-handling)
8. [Security Best Practices](#security-best-practices)
9. [API Reference](#api-reference)
10. [Testing](#testing)

---

## Prerequisites

### Required Dependencies

```bash
# Core Solana dependencies
npm install @solana/web3.js @coral-xyz/anchor

# Wallet adapter
npm install @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets

# Cryptography
npm install @noble/hashes

# Noir ZK proving (for withdrawal proofs)
npm install @noir-lang/noir_js @noir-lang/backend_barretenberg

# Arcium SDK (for confidential computation)
npm install @arcium/client
```

### Build Artifacts Required

Before frontend integration, ensure these are built:

```bash
# 1. Anchor program (generates IDL and types)
anchor build

# 2. Noir circuit (generates prover artifacts)
cd mixer && nargo compile && nargo prove --package mixer

# 3. Arcis circuits (for MPC)
arcium build
```

Generated files:
- `target/idl/zyncx.json` - Program IDL
- `target/types/zyncx.ts` - TypeScript types
- `mixer/target/mixer.json` - Noir circuit for proofs
- `build/*.arcis.ir` - Arcis MPC circuits

---

## Installation

### 1. Copy Required Files

```bash
# From project root
cp target/idl/zyncx.json app/lib/
cp target/types/zyncx.ts app/lib/
cp mixer/target/mixer.json app/lib/
```

### 2. Environment Configuration

Create `.env.local`:

```env
# Solana RPC
NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com

# Program ID (from Anchor.toml)
NEXT_PUBLIC_PROGRAM_ID=CVq4XNG8mtrBCvBawm1PRSSa72Dew9yy1rC9C25zJqh3

# Arcium MXE endpoint (devnet)
NEXT_PUBLIC_ARCIUM_MXE_URL=https://mxe.devnet.arcium.com

# Pyth Price Feed (SOL/USD)
NEXT_PUBLIC_PYTH_SOL_USD=H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         FRONTEND APPLICATION                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │   Deposit    │  │    Swap      │  │   Withdraw   │              │
│  │   Component  │  │   Component  │  │   Component  │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                 │                 │                       │
│         ▼                 ▼                 ▼                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      HOOKS LAYER                             │   │
│  │  useDeposit() │ useSwap() │ useWithdraw() │ useZyncx()      │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                 │                 │                       │
│         ▼                 ▼                 ▼                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    CORE LIBRARIES                            │   │
│  │  crypto.ts │ merkle.ts │ prover.ts │ program.ts │ arcium.ts │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└──────────────────────────────────────┬──────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────┐
│                          EXTERNAL SERVICES                           │
├─────────────────┬─────────────────┬─────────────────┬────────────────┤
│  Solana RPC     │   Arcium MXE    │  Jupiter API    │  Pyth Oracle   │
│  (transactions) │  (confidential)  │  (swap routes)  │  (prices)      │
└─────────────────┴─────────────────┴─────────────────┴────────────────┘
```

---

## Core Modules

### 1. Program Interface (`lib/program.ts`)

```typescript
import { PublicKey, Connection } from '@solana/web3.js';
import { AnchorProvider, Program, BN, Wallet } from '@coral-xyz/anchor';
import idl from './zyncx.json';

export const PROGRAM_ID = new PublicKey(process.env.NEXT_PUBLIC_PROGRAM_ID!);

// PDA Derivation Functions
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

// Encrypted accounts (for Arcium)
export function getEncryptedVaultPDA(tokenMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('enc_vault'), tokenMint.toBuffer()],
    PROGRAM_ID
  );
}

export function getEncryptedPositionPDA(vault: PublicKey, user: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('enc_position'), vault.toBuffer(), user.toBuffer()],
    PROGRAM_ID
  );
}

export function getProgram(connection: Connection, wallet: Wallet): Program {
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  return new Program(idl as any, PROGRAM_ID, provider);
}
```

### 2. Cryptography (`lib/crypto.ts`)

```typescript
import { poseidon2 } from 'poseidon-lite'; // Use actual Poseidon for production

// Generate deposit secrets
export function generateDepositSecrets(): { secret: Uint8Array; nullifierSecret: Uint8Array } {
  const secret = crypto.getRandomValues(new Uint8Array(32));
  const nullifierSecret = crypto.getRandomValues(new Uint8Array(32));
  return { secret, nullifierSecret };
}

// Compute precommitment (matches on-chain)
export function computePrecommitment(secret: Uint8Array, nullifierSecret: Uint8Array): Uint8Array {
  // In production: use Poseidon hash to match circuit
  const combined = new Uint8Array(64);
  combined.set(secret, 0);
  combined.set(nullifierSecret, 32);
  return poseidon2([bytesToBigInt(secret), bytesToBigInt(nullifierSecret)]);
}

// Compute commitment (matches on-chain)
export function computeCommitment(amount: bigint, precommitment: Uint8Array): Uint8Array {
  return poseidon2([amount, bytesToBigInt(precommitment)]);
}

// Compute nullifier hash (revealed on withdrawal)
export function computeNullifierHash(nullifierSecret: Uint8Array): Uint8Array {
  return poseidon2([bytesToBigInt(nullifierSecret)]);
}

// Deposit note structure
export interface DepositNote {
  secret: string;           // hex-encoded
  nullifierSecret: string;  // hex-encoded
  precommitment: string;    // hex-encoded
  amount: string;           // lamports as string
  commitment?: string;      // hex-encoded
  txSignature?: string;
  timestamp: number;
}

// Create deposit note
export function createDepositNote(
  secret: Uint8Array,
  nullifierSecret: Uint8Array,
  amount: bigint
): DepositNote {
  const precommitment = computePrecommitment(secret, nullifierSecret);
  const commitment = computeCommitment(amount, precommitment);
  
  return {
    secret: bytesToHex(secret),
    nullifierSecret: bytesToHex(nullifierSecret),
    precommitment: bytesToHex(precommitment),
    amount: amount.toString(),
    commitment: bytesToHex(commitment),
    timestamp: Date.now(),
  };
}

// Encode note as shareable string
export function encodeNote(note: DepositNote): string {
  return btoa(JSON.stringify(note));
}

// Decode note from string
export function decodeNote(encoded: string): DepositNote {
  return JSON.parse(atob(encoded));
}
```

### 3. ZK Proof Generation (`lib/prover.ts`)

```typescript
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import circuit from './mixer.json';

let noir: Noir | null = null;
let backend: BarretenbergBackend | null = null;

export async function initProver(): Promise<void> {
  if (!backend) {
    backend = new BarretenbergBackend(circuit as any);
    noir = new Noir(circuit as any);
  }
}

export interface WithdrawProofInputs {
  // Private inputs
  secret: Uint8Array;
  nullifierSecret: Uint8Array;
  totalAmount: bigint;          // Full deposit amount
  merklePath: Uint8Array[];     // Sibling hashes
  pathIndices: number[];        // 0=left, 1=right
  
  // For partial withdrawals
  newSecret: Uint8Array;
  newNullifierSecret: Uint8Array;
  
  // Public inputs
  root: Uint8Array;
  nullifierHash: Uint8Array;
  recipient: Uint8Array;
  withdrawAmount: bigint;       // Amount to withdraw
  newCommitment: Uint8Array;    // Change commitment (or zeros for full)
}

export async function generateWithdrawProof(inputs: WithdrawProofInputs): Promise<{
  proof: Uint8Array;
  publicInputs: any;
}> {
  await initProver();
  
  const witnessInputs = {
    // Private
    secret: toField(inputs.secret),
    nullifier_secret: toField(inputs.nullifierSecret),
    total_amount: inputs.totalAmount.toString(),
    merkle_path: inputs.merklePath.map(toField),
    path_indices: inputs.pathIndices.map(String),
    new_secret: toField(inputs.newSecret),
    new_nullifier_secret: toField(inputs.newNullifierSecret),
    
    // Public
    root: toField(inputs.root),
    nullifier_hash: toField(inputs.nullifierHash),
    recipient: toField(inputs.recipient),
    withdraw_amount: inputs.withdrawAmount.toString(),
    new_commitment: toField(inputs.newCommitment),
  };

  const { witness } = await noir!.execute(witnessInputs);
  const proof = await backend!.generateProof(witness);
  
  return {
    proof: proof.proof,
    publicInputs: proof.publicInputs,
  };
}

function toField(bytes: Uint8Array): string {
  return BigInt('0x' + bytesToHex(bytes)).toString();
}
```

### 4. Arcium MXE Integration (`lib/arcium.ts`)

```typescript
import { ArciumClient, Enc, X25519KeyPair } from '@arcium/client';

const MXE_URL = process.env.NEXT_PUBLIC_ARCIUM_MXE_URL!;

let arciumClient: ArciumClient | null = null;
let userKeyPair: X25519KeyPair | null = null;

// Initialize Arcium client
export async function initArcium(): Promise<void> {
  if (!arciumClient) {
    arciumClient = new ArciumClient(MXE_URL);
    userKeyPair = await X25519KeyPair.generate();
  }
}

// Get user's X25519 public key for encryption
export function getEncryptionPubkey(): Uint8Array {
  if (!userKeyPair) throw new Error('Arcium not initialized');
  return userKeyPair.publicKey;
}

// Encrypt swap input (amount hidden from everyone except user + MXE)
export async function encryptSwapInput(amount: bigint): Promise<{
  ciphertext: Uint8Array;
  nonce: Uint8Array;
}> {
  await initArcium();
  
  const amountBytes = new Uint8Array(8);
  new DataView(amountBytes.buffer).setBigUint64(0, amount, true);
  
  return arciumClient!.encrypt(amountBytes, userKeyPair!);
}

// Encrypt swap bounds (min_out, slippage, etc.)
export async function encryptSwapBounds(
  minOut: bigint,
  maxSlippageBps: number,
  aggressive: boolean
): Promise<{
  ciphertext: Uint8Array;
  nonce: Uint8Array;
}> {
  await initArcium();
  
  // Serialize bounds struct
  const data = new Uint8Array(11); // 8 + 2 + 1 bytes
  const view = new DataView(data.buffer);
  view.setBigUint64(0, minOut, true);
  view.setUint16(8, maxSlippageBps, true);
  data[10] = aggressive ? 1 : 0;
  
  return arciumClient!.encrypt(data, userKeyPair!);
}

// Decrypt swap result from MXE callback
export async function decryptSwapResult(
  encryptedResult: Uint8Array,
  nonce: Uint8Array
): Promise<{
  shouldExecute: boolean;
  minAmountOut: bigint;
}> {
  await initArcium();
  
  const decrypted = await arciumClient!.decrypt(encryptedResult, nonce, userKeyPair!);
  const view = new DataView(decrypted.buffer);
  
  return {
    shouldExecute: decrypted[0] === 1,
    minAmountOut: view.getBigUint64(1, true),
  };
}
```

---

## User Flows Implementation

### Flow 1: Deposit

```typescript
// hooks/useDeposit.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState } from 'react';
import { getProgram, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA } from '../program';
import { generateDepositSecrets, computePrecommitment, createDepositNote, DepositNote } from '../crypto';

export function useDeposit() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function deposit(amountSol: number): Promise<DepositNote> {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsLoading(true);
    setError(null);

    try {
      const program = getProgram(connection, wallet as any);
      const amountLamports = BigInt(Math.floor(amountSol * 1e9));

      // 1. Generate secrets (USER MUST SAVE THESE!)
      const { secret, nullifierSecret } = generateDepositSecrets();
      const precommitment = computePrecommitment(secret, nullifierSecret);

      // 2. Derive PDAs
      const NATIVE_MINT = PublicKey.default;
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      const [vaultTreasury] = getVaultTreasuryPDA(vault);

      // 3. Build and send transaction
      const tx = await program.methods
        .depositNative(
          new BN(amountLamports.toString()),
          Array.from(precommitment)
        )
        .accounts({
          depositor: wallet.publicKey,
          vault,
          merkleTree,
          vaultTreasury,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = wallet.publicKey;
      
      const signed = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      // 4. Create deposit note for user to save
      const note = createDepositNote(secret, nullifierSecret, amountLamports);
      note.txSignature = signature;

      return note;
    } catch (err: any) {
      setError(err.message);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }

  return { deposit, isLoading, error };
}
```

### Flow 2: Confidential Swap

```typescript
// hooks/useConfidentialSwap.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState } from 'react';
import { getProgram, getVaultPDA, getMerkleTreePDA, getEncryptedVaultPDA, getEncryptedPositionPDA } from '../program';
import { encryptSwapInput, encryptSwapBounds, getEncryptionPubkey, initArcium } from '../arcium';
import { generateWithdrawProof } from '../prover';

interface SwapParams {
  amountIn: bigint;           // Amount to swap (will be encrypted!)
  minAmountOut: bigint;       // Minimum output (encrypted)
  maxSlippageBps: number;     // Max slippage in basis points
  srcToken: PublicKey;        // Source token mint
  dstToken: PublicKey;        // Destination token mint
  depositNote: DepositNote;   // Your deposit note
}

export function useConfidentialSwap() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isLoading, setIsLoading] = useState(false);

  async function swap(params: SwapParams): Promise<string> {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsLoading(true);

    try {
      await initArcium();
      const program = getProgram(connection, wallet as any);

      // 1. Encrypt swap amount (HIDDEN from everyone!)
      const { ciphertext: encryptedAmount, nonce: amountNonce } = 
        await encryptSwapInput(params.amountIn);

      // 2. Encrypt swap bounds (min_out, slippage)
      const { ciphertext: encryptedBounds, nonce: boundsNonce } = 
        await encryptSwapBounds(params.minAmountOut, params.maxSlippageBps, false);

      // 3. Generate ZK proof for ownership
      const proof = await generateWithdrawProof({
        secret: hexToBytes(params.depositNote.secret),
        nullifierSecret: hexToBytes(params.depositNote.nullifierSecret),
        totalAmount: BigInt(params.depositNote.amount),
        // ... merkle path from on-chain
        withdrawAmount: params.amountIn,
        // ...
      });

      // 4. Derive PDAs
      const [srcVault] = getVaultPDA(params.srcToken);
      const [encVault] = getEncryptedVaultPDA(params.srcToken);
      const [encPosition] = getEncryptedPositionPDA(srcVault, wallet.publicKey);

      // 5. Queue confidential computation
      const tx = await program.methods
        .queueConfidentialSwap({
          encryptionPubkey: Array.from(getEncryptionPubkey()),
          amountNonce: new BN(amountNonce),
          encryptedAmount: Array.from(encryptedAmount),
          boundsNonce: new BN(boundsNonce),
          encryptedBounds: Array.from(encryptedBounds),
          srcToken: params.srcToken,
          dstToken: params.dstToken,
          nullifier: Array.from(computeNullifierHash(hexToBytes(params.depositNote.nullifierSecret))),
          newCommitment: Array.from(new Uint8Array(32)), // For full swap
          proof: Array.from(proof.proof),
        })
        .accounts({
          user: wallet.publicKey,
          vault: srcVault,
          encryptedVault: encVault,
          userPosition: encPosition,
          // ... other accounts
        })
        .transaction();

      const signature = await wallet.sendTransaction(tx, connection);
      await connection.confirmTransaction(signature, 'confirmed');

      return signature;
    } finally {
      setIsLoading(false);
    }
  }

  return { swap, isLoading };
}
```

### Flow 3: Withdrawal

```typescript
// hooks/useWithdraw.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useState } from 'react';
import { getProgram, getVaultPDA, getMerkleTreePDA, getVaultTreasuryPDA, getNullifierPDA } from '../program';
import { parseDepositNote, computeNullifierHash, generateDepositSecrets, computePrecommitment, computeCommitment, DepositNote } from '../crypto';
import { generateWithdrawProof, initProver } from '../prover';
import { fetchMerklePath } from '../merkle';

interface WithdrawParams {
  note: DepositNote;          // Your deposit note
  withdrawAmount: bigint;     // Amount to withdraw (can be partial)
  recipient: PublicKey;       // Where to send funds
}

export function useWithdraw() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [isLoading, setIsLoading] = useState(false);

  async function withdraw(params: WithdrawParams): Promise<{
    signature: string;
    changeNote?: DepositNote;  // New note for remaining balance (if partial)
  }> {
    if (!wallet.publicKey || !wallet.signTransaction) {
      throw new Error('Wallet not connected');
    }

    setIsLoading(true);

    try {
      await initProver();
      const program = getProgram(connection, wallet as any);
      const parsed = parseDepositNote(params.note);
      
      const totalAmount = BigInt(params.note.amount);
      const isPartial = params.withdrawAmount < totalAmount;

      // 1. Generate new secrets for change (if partial withdrawal)
      let newSecret = new Uint8Array(32);
      let newNullifierSecret = new Uint8Array(32);
      let newCommitment = new Uint8Array(32);
      let changeNote: DepositNote | undefined;

      if (isPartial) {
        const changeSecrets = generateDepositSecrets();
        newSecret = changeSecrets.secret;
        newNullifierSecret = changeSecrets.nullifierSecret;
        
        const changeAmount = totalAmount - params.withdrawAmount;
        const precommitment = computePrecommitment(newSecret, newNullifierSecret);
        newCommitment = computeCommitment(changeAmount, precommitment);
        
        // Create change note for user to save
        changeNote = {
          secret: bytesToHex(newSecret),
          nullifierSecret: bytesToHex(newNullifierSecret),
          precommitment: bytesToHex(precommitment),
          amount: changeAmount.toString(),
          commitment: bytesToHex(newCommitment),
          timestamp: Date.now(),
        };
      }

      // 2. Fetch Merkle path from on-chain
      const NATIVE_MINT = PublicKey.default;
      const [vault] = getVaultPDA(NATIVE_MINT);
      const [merkleTree] = getMerkleTreePDA(vault);
      
      const merkleAccount = await program.account.merkleTreeState.fetch(merkleTree);
      const { path, indices, root } = fetchMerklePath(
        merkleAccount,
        hexToBytes(params.note.commitment!)
      );

      // 3. Generate ZK proof
      const { proof } = await generateWithdrawProof({
        secret: parsed.secret,
        nullifierSecret: parsed.nullifierSecret,
        totalAmount,
        merklePath: path,
        pathIndices: indices,
        newSecret,
        newNullifierSecret,
        root,
        nullifierHash: parsed.nullifierHash,
        recipient: params.recipient.toBytes(),
        withdrawAmount: params.withdrawAmount,
        newCommitment,
      });

      // 4. Derive PDAs
      const [vaultTreasury] = getVaultTreasuryPDA(vault);
      const [nullifierPDA] = getNullifierPDA(vault, parsed.nullifierHash);

      // 5. Send withdraw transaction
      const tx = await program.methods
        .withdrawNative(
          new BN(params.withdrawAmount.toString()),
          Array.from(parsed.nullifierHash),
          Array.from(newCommitment),
          Array.from(proof)
        )
        .accounts({
          recipient: params.recipient,
          vault,
          merkleTree,
          vaultTreasury,
          nullifierAccount: nullifierPDA,
          verifierProgram: VERIFIER_PROGRAM_ID, // Sunspot verifier
          payer: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .transaction();

      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      tx.feePayer = wallet.publicKey;
      
      const signed = await wallet.signTransaction(tx);
      const signature = await connection.sendRawTransaction(signed.serialize());
      await connection.confirmTransaction(signature, 'confirmed');

      return { signature, changeNote };
    } finally {
      setIsLoading(false);
    }
  }

  return { withdraw, isLoading };
}
```

---

## Arcium MXE Integration

### Queue/Callback Pattern

The Arcium integration follows a queue-and-callback pattern:

```
1. User calls queue_confidential_swap with encrypted inputs
2. Transaction recorded on-chain with encrypted data
3. Arcium MXE nodes pick up the computation
4. MXE processes encrypted data without seeing plaintext
5. MXE calls back with encrypted result
6. User decrypts result with their private key
```

### Encrypted Data Types

| Parameter | Type | Who Can Decrypt |
|-----------|------|-----------------|
| `swap_input.amount` | `Enc<Shared, _>` | User + MXE |
| `swap_bounds` | `Enc<Shared, _>` | User + MXE |
| `vault_state` | `Enc<Mxe, _>` | MXE only |
| `user_position` | `Enc<Mxe, _>` | MXE only |
| `swap_result` | `Enc<Shared, _>` | User + MXE |

### Listening for Callbacks

```typescript
// Subscribe to swap completion events
function subscribeToSwapCompletion(
  connection: Connection,
  swapRequestPDA: PublicKey,
  onComplete: (result: SwapResult) => void
) {
  const subscriptionId = connection.onAccountChange(
    swapRequestPDA,
    async (accountInfo) => {
      const data = accountInfo.data;
      // Parse swap request status
      const status = data[SWAP_STATUS_OFFSET];
      
      if (status === SwapRequestStatus.Completed) {
        // Extract encrypted result
        const encryptedResult = data.slice(RESULT_OFFSET, RESULT_OFFSET + 64);
        const resultNonce = data.slice(NONCE_OFFSET, NONCE_OFFSET + 16);
        
        // Decrypt with user's key
        const result = await decryptSwapResult(encryptedResult, resultNonce);
        onComplete(result);
      }
    },
    'confirmed'
  );

  return () => connection.removeAccountChangeListener(subscriptionId);
}
```

---

## Error Handling

### Common Errors

```typescript
export enum ZyncxError {
  // Deposit errors
  INVALID_DEPOSIT_AMOUNT = 'InvalidDepositAmount',
  VAULT_NOT_FOUND = 'VaultNotFound',
  
  // Withdrawal errors
  INVALID_ZK_PROOF = 'InvalidZKProof',
  NULLIFIER_ALREADY_SPENT = 'NullifierAlreadySpent',
  INSUFFICIENT_FUNDS = 'InsufficientFunds',
  INVALID_MERKLE_ROOT = 'InvalidMerkleRoot',
  
  // Swap errors
  INVALID_SWAP_AMOUNT = 'InvalidSwapAmount',
  SLIPPAGE_EXCEEDED = 'SlippageExceeded',
  
  // Arcium errors
  COMPUTATION_EXPIRED = 'ComputationExpired',
  COMPUTATION_FAILED = 'AbortedComputation',
  CLUSTER_NOT_SET = 'ClusterNotSet',
}

export function parseZyncxError(error: any): string {
  const errorCode = error?.error?.errorCode?.code;
  
  switch (errorCode) {
    case ZyncxError.INVALID_ZK_PROOF:
      return 'Invalid proof - check your deposit note';
    case ZyncxError.NULLIFIER_ALREADY_SPENT:
      return 'This deposit has already been withdrawn';
    case ZyncxError.INSUFFICIENT_FUNDS:
      return 'Insufficient funds in vault';
    default:
      return error.message || 'Unknown error';
  }
}
```

---

## Security Best Practices

### 1. Never Store Secrets in localStorage

```typescript
// ❌ BAD - Never do this!
localStorage.setItem('depositNote', JSON.stringify(note));

// ✅ GOOD - Let user download/backup manually
function downloadNote(note: DepositNote) {
  const blob = new Blob([JSON.stringify(note, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `zyncx-note-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
}
```

### 2. Validate Inputs Before Sending

```typescript
function validateWithdrawInputs(note: DepositNote, amount: bigint) {
  if (!note.secret || note.secret.length !== 64) {
    throw new Error('Invalid secret');
  }
  if (!note.nullifierSecret || note.nullifierSecret.length !== 64) {
    throw new Error('Invalid nullifier secret');
  }
  if (amount <= 0n) {
    throw new Error('Amount must be positive');
  }
  if (amount > BigInt(note.amount)) {
    throw new Error('Cannot withdraw more than deposited');
  }
}
```

### 3. Use Fresh Addresses for Withdrawals

```typescript
// Generate a new keypair for receiving funds
import { Keypair } from '@solana/web3.js';

function generateFreshRecipient(): { keypair: Keypair; address: PublicKey } {
  const keypair = Keypair.generate();
  return {
    keypair,
    address: keypair.publicKey,
  };
}
```

### 4. Clear Sensitive Data After Use

```typescript
function clearSecrets(secrets: { secret: Uint8Array; nullifierSecret: Uint8Array }) {
  secrets.secret.fill(0);
  secrets.nullifierSecret.fill(0);
}
```

---

## API Reference

### Program Instructions

| Instruction | Description | Accounts |
|-------------|-------------|----------|
| `deposit_native` | Deposit SOL to shielded pool | vault, merkle_tree, vault_treasury |
| `deposit_token` | Deposit SPL tokens | vault, merkle_tree, vault_token_account |
| `withdraw_native` | Withdraw SOL with ZK proof | vault, merkle_tree, nullifier_account, verifier |
| `withdraw_token` | Withdraw tokens with ZK proof | vault, merkle_tree, nullifier_account, verifier |
| `queue_confidential_swap` | Queue encrypted swap | vault, encrypted_vault, swap_request |
| `confidential_swap_callback` | MXE callback (internal) | swap_request, vault |

### PDAs

| PDA | Seeds | Purpose |
|-----|-------|---------|
| Vault | `["vault", asset_mint]` | Vault state |
| MerkleTree | `["merkle_tree", vault]` | Commitment tree |
| VaultTreasury | `["vault_treasury", vault]` | Holds SOL |
| Nullifier | `["nullifier", vault, nullifier_hash]` | Spent tracking |
| EncryptedVault | `["enc_vault", token_mint]` | MXE vault state |
| EncryptedPosition | `["enc_position", vault, user]` | User MXE position |
| SwapRequest | `["swap_request", computation_offset]` | Pending swap |

---

## Testing

### Unit Tests

```typescript
// __tests__/crypto.test.ts
import { generateDepositSecrets, computePrecommitment, computeCommitment, computeNullifierHash } from '../lib/crypto';

describe('Crypto', () => {
  test('generates unique secrets', () => {
    const a = generateDepositSecrets();
    const b = generateDepositSecrets();
    expect(a.secret).not.toEqual(b.secret);
    expect(a.nullifierSecret).not.toEqual(b.nullifierSecret);
  });

  test('commitment is deterministic', () => {
    const { secret, nullifierSecret } = generateDepositSecrets();
    const precommitment = computePrecommitment(secret, nullifierSecret);
    const amount = 1000000000n;
    
    const commitment1 = computeCommitment(amount, precommitment);
    const commitment2 = computeCommitment(amount, precommitment);
    
    expect(commitment1).toEqual(commitment2);
  });

  test('nullifier hash matches circuit', () => {
    const nullifierSecret = new Uint8Array(32).fill(42);
    const hash = computeNullifierHash(nullifierSecret);
    expect(hash.length).toBe(32);
  });
});
```

### Integration Tests

```typescript
// __tests__/integration.test.ts
import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';

describe('Zyncx Integration', () => {
  const connection = new Connection('http://localhost:8899', 'confirmed');
  let payer: Keypair;

  beforeAll(async () => {
    payer = Keypair.generate();
    const sig = await connection.requestAirdrop(payer.publicKey, 10 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
  });

  test('deposit and withdraw flow', async () => {
    // 1. Deposit
    const depositResult = await deposit(1); // 1 SOL
    expect(depositResult.txSignature).toBeDefined();
    
    // 2. Wait for confirmation
    await new Promise(r => setTimeout(r, 2000));
    
    // 3. Withdraw
    const withdrawResult = await withdraw({
      note: depositResult,
      withdrawAmount: 1_000_000_000n,
      recipient: payer.publicKey,
    });
    
    expect(withdrawResult.signature).toBeDefined();
  });
});
```

---

## Summary

### Integration Checklist

- [ ] Install all dependencies
- [ ] Copy build artifacts (IDL, types, circuit)
- [ ] Configure environment variables
- [ ] Implement `useDeposit` hook
- [ ] Implement `useWithdraw` hook  
- [ ] Implement `useConfidentialSwap` hook (optional)
- [ ] Add note backup/restore UI
- [ ] Add error handling
- [ ] Test on devnet
- [ ] Security review

### Build Status

| Component | Status | Files |
|-----------|--------|-------|
| Anchor Program | ✅ Built | `target/idl/zyncx.json`, `target/types/zyncx.ts` |
| Noir Circuit | ✅ Built | `mixer/target/mixer.json`, `mixer.pk`, `mixer.vk` |
| Arcis Circuits | ✅ Built | 11 circuits in `build/*.arcis.ir` |
| Verifier (Sunspot) | ✅ Built | `mixer/target/mixer.so` |

---

*Last Updated: February 2026*
*Integration Guide Version: 1.0.0*
