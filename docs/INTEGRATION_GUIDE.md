# Zyncx Integration Guide

> **Complete step-by-step guide for deploying and integrating Zyncx Protocol**

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Building the Project](#building-the-project)
4. [Deploying Programs](#deploying-programs)
5. [Arcium MXE Setup](#arcium-mxe-setup)
6. [Frontend Integration](#frontend-integration)
7. [SDK Usage](#sdk-usage)
8. [Testing](#testing)
9. [Production Deployment](#production-deployment)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Tools

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Solana CLI | 2.0+ | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` |
| Anchor CLI | 0.32.1 | `cargo install --git https://github.com/coral-xyz/anchor avm --locked` then `avm install 0.32.1` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| Yarn | 1.22+ | `npm install -g yarn` |
| Nargo (Noir) | 0.31+ | `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install \| bash && noirup` |
| Arcium CLI | Latest | `npm i -g @aspect/arcis-cli` |

### Verify Installation

```bash
# Check all tools
solana --version      # solana-cli 2.x.x
anchor --version      # anchor-cli 0.32.1
nargo --version       # nargo version 0.31.x
arcium --version      # arcium-cli x.x.x
node --version        # v18.x.x or higher
yarn --version        # 1.22.x
```

---

## Environment Setup

### 1. Clone and Install Dependencies

```bash
# Clone repository
git clone https://github.com/your-org/zyncx.git
cd zyncx

# Install root dependencies
yarn install

# Install frontend dependencies
cd app && yarn install && cd ..
```

### 2. Configure Solana

```bash
# For local development
solana config set --url localhost

# For devnet
solana config set --url devnet

# Generate keypair (if needed)
solana-keygen new -o ~/.config/solana/id.json

# Airdrop for testing (devnet only)
solana airdrop 5
```

### 3. Configure Arcium

Create `Arcium.toml` in project root (if not exists):

```toml
[build]
circuit_path = "./encrypted-ixs"
output_path = "./target/encrypted-ixs"

[network]
cluster = "devnet"  # or "localnet" for testing
mxe_program_id = "..." # Arcium devnet program ID
```

---

## Building the Project

### Step 1: Build Solana Program

```bash
# From project root
anchor build

# Expected output:
# Compiling zyncx v0.1.0
# Finished release [optimized] target(s)
```

**Output files:**
- `target/deploy/zyncx.so` - Program binary
- `target/idl/zyncx.json` - IDL for clients
- `target/types/zyncx.ts` - TypeScript types

### Step 2: Build Arcium Circuits

```bash
# Build MPC circuits
arcium build

# Expected output:
# Building encrypted-ixs...
# Generated circuit artifacts
```

**Output files:**
- `target/encrypted-ixs/*.wasm` - Circuit WASM
- `target/encrypted-ixs/*.json` - Circuit metadata

### Step 3: Build Noir Circuit

```bash
cd mixer

# Compile the circuit
nargo compile

# Generate proving/verification keys
nargo prove
nargo verify

cd ..
```

**Output files:**
- `mixer/target/mixer.json` - Compiled circuit
- `mixer/target/mixer.pk` - Proving key
- `mixer/target/mixer.vk` - Verification key

### Step 4: Build Frontend

```bash
cd app
yarn build
cd ..
```

---

## Deploying Programs

### Local Deployment (Development)

#### 1. Start Local Validator

```bash
# Terminal 1: Start validator with Arcium
solana-test-validator --reset

# Or with specific programs pre-deployed:
solana-test-validator --reset \
  --bpf-program <ARCIUM_PROGRAM_ID> /path/to/arcium.so
```

#### 2. Deploy Zyncx Program

```bash
# Deploy to localnet
anchor deploy

# Get program ID
solana address -k target/deploy/zyncx-keypair.json
# Output: 7698BfsbJabinNT1jcmob9TxW7iD2gjtNCT4TbAkhyjH
```

#### 3. Update Program ID

If the program ID changed, update `Anchor.toml`:

```toml
[programs.localnet]
zyncx = "YOUR_NEW_PROGRAM_ID"
```

And `lib.rs`:

```rust
declare_id!("YOUR_NEW_PROGRAM_ID");
```

Then rebuild:

```bash
anchor build
anchor deploy
```

### Devnet Deployment

```bash
# Switch to devnet
solana config set --url devnet

# Ensure sufficient SOL
solana balance
solana airdrop 5  # If needed

# Deploy
anchor deploy --provider.cluster devnet

# Verify
solana program show <PROGRAM_ID>
```

### Mainnet Deployment

⚠️ **Production Checklist:**

- [ ] Security audit completed
- [ ] All tests passing
- [ ] Upgrade authority configured
- [ ] Multisig for authority
- [ ] Monitoring setup
- [ ] Incident response plan

```bash
# Switch to mainnet
solana config set --url mainnet-beta

# Deploy with upgrade authority
anchor deploy --provider.cluster mainnet \
  --program-keypair ./mainnet-keypair.json
```

---

## Arcium MXE Setup

### 1. Register Computation Definitions

After deploying the Solana program, register Arcium circuits:

```typescript
// scripts/setup-arcium.ts
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Zyncx } from "../target/types/zyncx";

async function setupArcium() {
  const provider = AnchorProvider.env();
  const program = anchor.workspace.Zyncx as Program<Zyncx>;
  
  // Initialize computation definitions
  console.log("Initializing init_vault computation...");
  await program.methods.initVaultCompDef()
    .accounts({
      payer: provider.wallet.publicKey,
      // ... arcium accounts
    })
    .rpc();

  console.log("Initializing process_deposit computation...");
  await program.methods.initProcessDepositCompDef()
    .accounts({
      payer: provider.wallet.publicKey,
      // ... arcium accounts
    })
    .rpc();

  console.log("Initializing confidential_swap computation...");
  await program.methods.initConfidentialSwapCompDef()
    .accounts({
      payer: provider.wallet.publicKey,
      // ... arcium accounts
    })
    .rpc();

  console.log("Arcium setup complete!");
}

setupArcium();
```

Run:

```bash
npx ts-node scripts/setup-arcium.ts
```

### 2. Create Encrypted Vault

```typescript
async function createEncryptedVault(tokenMint: PublicKey) {
  const computationOffset = Date.now(); // Unique offset
  const nonce = BigInt(Math.floor(Math.random() * 1e18));

  await program.methods.createEncryptedVault(
    new anchor.BN(computationOffset),
    nonce
  )
  .accounts({
    payer: provider.wallet.publicKey,
    tokenMint: tokenMint,
    vault: vaultPda,
    // ... arcium accounts
  })
  .rpc();
}
```

---

## Frontend Integration

### 1. Project Structure

```
app/
├── app/
│   ├── layout.tsx          # Root layout with providers
│   ├── page.tsx            # Landing page
│   └── globals.css         # Tailwind styles
├── components/
│   ├── WalletProvider.tsx  # Solana wallet adapter
│   ├── PrivacyVault.tsx    # Main vault UI
│   ├── Navbar.tsx          # Navigation
│   └── ...
├── lib/
│   ├── zyncx-client.ts     # SDK wrapper
│   ├── noir-prover.ts      # ZK proof generation
│   └── arcium-client.ts    # Arcium MXE client
└── hooks/
    ├── useVault.ts         # Vault state hook
    ├── useDeposit.ts       # Deposit hook
    └── useWithdraw.ts      # Withdrawal hook
```

### 2. Wallet Provider Setup

```tsx
// components/WalletProvider.tsx
"use client";

import { WalletAdapterNetwork } from "@solana/wallet-adapter-base";
import {
  ConnectionProvider,
  WalletProvider as SolanaWalletProvider,
} from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { clusterApiUrl } from "@solana/web3.js";

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const network = WalletAdapterNetwork.Devnet;
  const endpoint = clusterApiUrl(network);
  
  const wallets = [
    new PhantomWalletAdapter(),
    new SolflareWalletAdapter(),
  ];

  return (
    <ConnectionProvider endpoint={endpoint}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          {children}
        </WalletModalProvider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
}
```

### 3. Zyncx Client SDK

```typescript
// lib/zyncx-client.ts
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { Zyncx, IDL } from "../target/types/zyncx";
import * as crypto from "crypto";

const PROGRAM_ID = new PublicKey("7698BfsbJabinNT1jcmob9TxW7iD2gjtNCT4TbAkhyjH");

export class ZyncxClient {
  program: Program<Zyncx>;
  provider: AnchorProvider;

  constructor(connection: Connection, wallet: any) {
    this.provider = new AnchorProvider(connection, wallet, {});
    this.program = new Program(IDL, PROGRAM_ID, this.provider);
  }

  // Generate deposit secrets
  generateSecrets(): { secret: Buffer; nullifierSecret: Buffer } {
    return {
      secret: crypto.randomBytes(32),
      nullifierSecret: crypto.randomBytes(32),
    };
  }

  // Compute commitment
  computeCommitment(
    secret: Buffer,
    nullifierSecret: Buffer,
    amount: bigint
  ): Buffer {
    // Use keccak256 (matching on-chain)
    const data = Buffer.concat([
      secret,
      nullifierSecret,
      Buffer.from(amount.toString(16).padStart(16, '0'), 'hex'),
    ]);
    return crypto.createHash('sha3-256').update(data).digest();
  }

  // Initialize vault
  async initializeVault(assetMint: PublicKey): Promise<string> {
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), assetMint.toBuffer()],
      this.program.programId
    );

    const [merkleTreePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merkle_tree"), vaultPda.toBuffer()],
      this.program.programId
    );

    const tx = await this.program.methods
      .initializeVault(assetMint)
      .accounts({
        authority: this.provider.wallet.publicKey,
        vault: vaultPda,
        merkleTree: merkleTreePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    return tx;
  }

  // Deposit SOL
  async depositNative(
    amount: number,
    secrets: { secret: Buffer; nullifierSecret: Buffer }
  ): Promise<{ tx: string; commitment: Buffer; leafIndex: number }> {
    const amountLamports = BigInt(amount * 1e9);
    const commitment = this.computeCommitment(
      secrets.secret,
      secrets.nullifierSecret,
      amountLamports
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), PublicKey.default.toBuffer()], // Native SOL
      this.program.programId
    );

    const [merkleTreePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merkle_tree"), vaultPda.toBuffer()],
      this.program.programId
    );

    const [treasuryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_treasury"), vaultPda.toBuffer()],
      this.program.programId
    );

    const tx = await this.program.methods
      .depositNative(
        new BN(amountLamports.toString()),
        Array.from(commitment) as any
      )
      .accounts({
        user: this.provider.wallet.publicKey,
        vault: vaultPda,
        merkleTree: merkleTreePda,
        vaultTreasury: treasuryPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Get leaf index from event or state
    const merkleState = await this.program.account.merkleTreeState.fetch(merkleTreePda);
    const leafIndex = merkleState.nextIndex - 1;

    return { tx, commitment, leafIndex };
  }

  // Withdraw (requires ZK proof)
  async withdrawNative(
    amount: number,
    nullifier: Buffer,
    newCommitment: Buffer,
    proof: Buffer
  ): Promise<string> {
    // ... implementation
  }
}
```

### 4. ZK Proof Generation (Client-Side)

```typescript
// lib/noir-prover.ts
import { Noir } from "@noir-lang/noir_js";
import { UltraHonkBackend } from "@noir-lang/backend_barretenberg";
import circuit from "../mixer/target/mixer.json";

export class NoirProver {
  noir: Noir;
  backend: UltraHonkBackend;

  async initialize() {
    this.backend = new UltraHonkBackend(circuit.bytecode);
    this.noir = new Noir(circuit);
  }

  async generateWithdrawalProof(inputs: {
    // Private inputs
    secret: string;
    nullifierSecret: string;
    newSecret: string;
    newNullifierSecret: string;
    merklePath: string[];
    pathIndices: number[];
    totalAmount: string;
    // Public inputs
    root: string;
    nullifierHash: string;
    recipient: string;
    withdrawAmount: string;
    newCommitment: string;
  }): Promise<{ proof: Uint8Array; publicInputs: string[] }> {
    
    // Execute circuit
    const { witness } = await this.noir.execute(inputs);
    
    // Generate proof
    const proof = await this.backend.generateProof(witness);
    
    return {
      proof: proof.proof,
      publicInputs: proof.publicInputs,
    };
  }

  async verifyProof(
    proof: Uint8Array,
    publicInputs: string[]
  ): Promise<boolean> {
    return await this.backend.verifyProof({ proof, publicInputs });
  }
}
```

### 5. React Hooks

```typescript
// hooks/useDeposit.ts
import { useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { ZyncxClient } from "../lib/zyncx-client";

export function useDeposit() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const deposit = async (amount: number) => {
    if (!wallet.publicKey) {
      setError("Wallet not connected");
      return null;
    }

    setLoading(true);
    setError(null);

    try {
      const client = new ZyncxClient(connection, wallet);
      const secrets = client.generateSecrets();
      
      const result = await client.depositNative(amount, secrets);

      // CRITICAL: Store secrets securely!
      // In production, encrypt and store client-side
      const depositData = {
        commitment: result.commitment.toString('hex'),
        secret: secrets.secret.toString('hex'),
        nullifierSecret: secrets.nullifierSecret.toString('hex'),
        amount,
        leafIndex: result.leafIndex,
        tx: result.tx,
        timestamp: Date.now(),
      };

      // Store in localStorage (demo only - use secure storage in production)
      const deposits = JSON.parse(localStorage.getItem('zyncx_deposits') || '[]');
      deposits.push(depositData);
      localStorage.setItem('zyncx_deposits', JSON.stringify(deposits));

      return result;
    } catch (e: any) {
      setError(e.message);
      return null;
    } finally {
      setLoading(false);
    }
  };

  return { deposit, loading, error };
}
```

### 6. UI Components

```tsx
// components/PrivacyVault.tsx
"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useDeposit } from "../hooks/useDeposit";

export function PrivacyVault() {
  const { publicKey } = useWallet();
  const { deposit, loading, error } = useDeposit();
  const [amount, setAmount] = useState("");

  const handleDeposit = async () => {
    const result = await deposit(parseFloat(amount));
    if (result) {
      alert(`Deposit successful! TX: ${result.tx}`);
      // IMPORTANT: Show user their secrets to backup
    }
  };

  if (!publicKey) {
    return (
      <div className="flex flex-col items-center gap-4 p-8">
        <h2 className="text-2xl font-bold">Connect Wallet to Start</h2>
        <WalletMultiButton />
      </div>
    );
  }

  return (
    <div className="max-w-md mx-auto p-6 bg-gray-900 rounded-xl">
      <h2 className="text-xl font-bold mb-4">Private Vault</h2>
      
      {/* Deposit Section */}
      <div className="mb-6">
        <h3 className="font-semibold mb-2">Deposit SOL</h3>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="Amount in SOL"
          className="w-full p-2 rounded bg-gray-800 text-white mb-2"
        />
        <button
          onClick={handleDeposit}
          disabled={loading || !amount}
          className="w-full p-2 bg-green-600 hover:bg-green-700 rounded disabled:opacity-50"
        >
          {loading ? "Processing..." : "Deposit"}
        </button>
        {error && <p className="text-red-500 mt-2">{error}</p>}
      </div>

      {/* Warning */}
      <div className="p-4 bg-yellow-900/50 rounded text-yellow-200 text-sm">
        ⚠️ <strong>Important:</strong> After depositing, you will receive secrets 
        that MUST be saved. Without these secrets, your funds are unrecoverable!
      </div>
    </div>
  );
}
```

---

## SDK Usage

### Complete Deposit-Withdraw Flow

```typescript
// Example: Full privacy flow
import { ZyncxClient } from "./lib/zyncx-client";
import { NoirProver } from "./lib/noir-prover";

async function privacyDemo() {
  // 1. Initialize clients
  const zyncx = new ZyncxClient(connection, wallet);
  const prover = new NoirProver();
  await prover.initialize();

  // 2. Generate secrets for deposit
  const secrets = zyncx.generateSecrets();
  console.log("⚠️ SAVE THESE SECRETS:");
  console.log("Secret:", secrets.secret.toString('hex'));
  console.log("Nullifier Secret:", secrets.nullifierSecret.toString('hex'));

  // 3. Deposit 1 SOL
  const deposit = await zyncx.depositNative(1.0, secrets);
  console.log("Deposited! TX:", deposit.tx);
  console.log("Leaf Index:", deposit.leafIndex);

  // 4. Wait some time for privacy...
  await sleep(60000); // 1 minute minimum recommended

  // 5. Prepare withdrawal
  const newSecrets = zyncx.generateSecrets(); // For change
  
  // Get merkle proof (from on-chain state)
  const merkleProof = await zyncx.getMerkleProof(deposit.leafIndex);

  // 6. Generate ZK proof
  const proof = await prover.generateWithdrawalProof({
    secret: secrets.secret.toString(),
    nullifierSecret: secrets.nullifierSecret.toString(),
    newSecret: newSecrets.secret.toString(),
    newNullifierSecret: newSecrets.nullifierSecret.toString(),
    merklePath: merkleProof.path,
    pathIndices: merkleProof.indices,
    totalAmount: (1e9).toString(), // 1 SOL in lamports
    root: merkleProof.root,
    nullifierHash: computeNullifierHash(secrets.nullifierSecret),
    recipient: wallet.publicKey.toString(),
    withdrawAmount: (0.5e9).toString(), // Withdraw 0.5 SOL
    newCommitment: zyncx.computeCommitment(
      newSecrets.secret,
      newSecrets.nullifierSecret,
      BigInt(0.5e9) // 0.5 SOL remaining
    ).toString(),
  });

  // 7. Submit withdrawal
  const withdrawTx = await zyncx.withdrawNative(
    0.5,
    computeNullifierHash(secrets.nullifierSecret),
    proof.newCommitment,
    Buffer.from(proof.proof)
  );
  console.log("Withdrawn! TX:", withdrawTx);

  // 8. Save new secrets for remaining balance
  console.log("⚠️ SAVE NEW SECRETS for remaining 0.5 SOL:");
  console.log("New Secret:", newSecrets.secret.toString('hex'));
  console.log("New Nullifier Secret:", newSecrets.nullifierSecret.toString('hex'));
}
```

---

## Testing

### Run Unit Tests

```bash
# Rust unit tests
cd contracts/solana/zyncx
cargo test

# Anchor integration tests
cd ../../..
anchor test

# Noir circuit tests
cd mixer
nargo test
```

### Run Local Integration Test

```bash
# Terminal 1: Start validator
solana-test-validator --reset

# Terminal 2: Deploy and test
anchor deploy
anchor run test
```

### Test Script Example

```typescript
// tests/zyncx.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zyncx } from "../target/types/zyncx";
import { expect } from "chai";

describe("zyncx", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Zyncx as Program<Zyncx>;

  it("initializes vault", async () => {
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), PublicKey.default.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeVault(PublicKey.default)
      .accounts({
        authority: provider.wallet.publicKey,
        vault: vaultPda,
        // ...
      })
      .rpc();

    const vault = await program.account.vaultState.fetch(vaultPda);
    expect(vault.totalDeposited.toNumber()).to.equal(0);
  });

  it("deposits SOL", async () => {
    // ... test deposit
  });

  it("withdraws with valid proof", async () => {
    // ... test withdrawal with ZK proof
  });
});
```

---

## Production Deployment

### Pre-Deployment Checklist

- [ ] **Security audit** - Smart contract and circuits audited
- [ ] **Test coverage** - >90% code coverage
- [ ] **Load testing** - Tested under expected load
- [ ] **Key management** - Upgrade authority secured (multisig)
- [ ] **Monitoring** - Alerts for anomalies
- [ ] **Documentation** - User guides complete
- [ ] **Legal review** - Compliance verified

### Mainnet Deployment Steps

```bash
# 1. Build release version
anchor build --verifiable

# 2. Verify build (reproducible)
anchor verify <PROGRAM_ID>

# 3. Deploy with multisig authority
anchor deploy --provider.cluster mainnet \
  --program-keypair ./mainnet-program-keypair.json

# 4. Initialize Arcium computations
npx ts-node scripts/setup-arcium.ts --network mainnet

# 5. Create initial vaults
npx ts-node scripts/create-vaults.ts --network mainnet

# 6. Verify deployment
solana program show <PROGRAM_ID>
```

### Environment Variables (Production)

```bash
# .env.production
NEXT_PUBLIC_SOLANA_RPC=https://api.mainnet-beta.solana.com
NEXT_PUBLIC_PROGRAM_ID=<MAINNET_PROGRAM_ID>
NEXT_PUBLIC_ARCIUM_CLUSTER=mainnet
```

---

## Troubleshooting

### Common Build Issues

| Error | Cause | Solution |
|-------|-------|----------|
| `Stack offset exceeded` | Large variables in arcium-client | Ignore warning (from dependency) |
| `unresolved import keccak` | Missing solana-program | Add to Cargo.toml |
| `anchor version mismatch` | CLI vs package mismatch | `yarn upgrade @coral-xyz/anchor@0.32.1` |
| `circuit compilation failed` | Noir syntax error | Check `mixer/src/main.nr` |

### Runtime Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `NullifierAlreadySpent` | Double-spend attempt | Use fresh nullifier |
| `InvalidMerkleRoot` | Old proof | Re-fetch current root |
| `InvalidProof` | Bad ZK proof | Verify inputs match circuit |
| `ComputationTimeout` | Arcium slow | Increase timeout or retry |

### Debug Commands

```bash
# Check program logs
solana logs <PROGRAM_ID>

# Inspect account
solana account <ACCOUNT_ADDRESS>

# View transaction
solana confirm -v <TX_SIGNATURE>

# Check Arcium status
arcium status --program <PROGRAM_ID>
```

---

## Next Steps

### Planned Enhancements

1. **User Positions** - Track individual encrypted positions
2. **DCA Support** - Dollar-cost averaging circuits
3. **Limit Orders** - Encrypted limit order book
4. **Multi-Token Vaults** - Single vault for multiple assets
5. **Relayer Network** - Gas-free withdrawals

### Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

---

*Integration Guide v0.3.0 - February 2026*
