# Arcium SDK Integration Guide

## Project Structure

```
zyncx/
├── Cargo.toml                          # Workspace with proc-macro2 patch
├── rust-toolchain                      # Rust 1.88.0 required
├── Arcium.toml                         # Arcium configuration
├── contracts/solana/zyncx/
│   ├── Cargo.toml                      # arcium-anchor, arcium-client, arcium-macros
│   └── src/
│       ├── lib.rs                      # #[arcium_program] instead of #[program]
│       └── instructions/
│           └── confidential.rs         # Arcium callback handlers
└── encrypted-ixs/
    ├── Cargo.toml                      # arcis-imports = "0.3.0"
    └── src/
        └── lib.rs                      # #[encrypted] circuits with #[instruction]
```

## Dependencies Added

### Workspace (Cargo.toml)
```toml
[patch.crates-io]
blake3 = { git = "https://github.com/BLAKE3-team/BLAKE3", tag = "1.5.4" }
proc-macro2 = { git = "https://github.com/arcium-hq/proc-macro2.git" }  # Required for Arcium v0.3.0
```

### Solana Program (contracts/solana/zyncx/Cargo.toml)
```toml
[dependencies]
anchor-lang = { version = "0.30.1", features = ["init-if-needed"] }
arcium-anchor = "0.3.0"
arcium-client = "0.3.0"
arcium-macros = "0.3.0"
```

### Encrypted Instructions (encrypted-ixs/Cargo.toml)
```toml
[dependencies]
arcis-imports = "0.3.0"
```

## Rust Toolchain

Created `rust-toolchain` file with:
```
1.88.0
```

## Circuit Structure (encrypted-ixs/src/lib.rs)

Following the official Arcium pattern:

```rust
use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    pub struct InputValues {
        v1: u8,
        v2: u8,
    }

    #[instruction]
    pub fn add_together(input_ctxt: Enc<Shared, InputValues>) -> Enc<Shared, u16> {
        let input = input_ctxt.to_arcis();
        let sum = input.v1 as u16 + input.v2 as u16;
        input_ctxt.owner.from_arcis(sum)
    }
}
```

## Solana Program Integration

### Main Program (lib.rs)
```rust
use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

#[arcium_program]  // Instead of #[program]
pub mod zyncx {
    use super::*;
    
    // Regular Anchor instructions...
    
    // Arcium computation definition initialization
    pub fn init_comp_def(ctx: Context<InitCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }
    
    // Queue confidential computation
    pub fn queue_swap(
        ctx: Context<QueueSwap>,
        computation_offset: u64,
        ciphertext_min_price: [u8; 32],
        ciphertext_max_slippage: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let args = vec![
            Argument::ArcisPubkey(pub_key),
            Argument::PlaintextU128(nonce),
            Argument::EncryptedU64(ciphertext_min_price),
            Argument::EncryptedU16(ciphertext_max_slippage),
        ];
        
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![SwapCallback::callback_ix(&[])],
        )?;
        Ok(())
    }
    
    // Callback handler
    #[arcium_callback(encrypted_ix = "validate_confidential_swap")]
    pub fn swap_callback(
        ctx: Context<SwapCallback>,
        output: ComputationOutputs<ValidateConfidentialSwapOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(ValidateConfidentialSwapOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        // Execute swap with result
        emit!(SwapExecuted {
            should_execute: result.ciphertexts[0],
            min_output: result.ciphertexts[1],
        });
        Ok(())
    }
}
```

## TypeScript Client Usage

### Install Dependencies
```bash
npm install @arcium-hq/client
```

### Encryption/Decryption
```typescript
import { RescueCipher, x25519, getArciumEnv } from "@arcium-hq/client";
import { randomBytes } from "crypto";

// Generate keys
const privateKey = x25519.utils.randomSecretKey();
const publicKey = x25519.getPublicKey(privateKey);

// Get MXE public key
const mxePublicKey = await getMXEPublicKeyWithRetry(
  provider as anchor.AnchorProvider,
  program.programId
);

// Create shared secret
const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
const cipher = new RescueCipher(sharedSecret);

// Encrypt data
const plaintext = [BigInt(1000), BigInt(50)]; // min_price, max_slippage
const nonce = randomBytes(16);
const ciphertext = cipher.encrypt(plaintext, nonce);

// Queue computation
await program.methods
  .queueSwap(
    computationOffset,
    Array.from(ciphertext[0]),
    Array.from(ciphertext[1]),
    Array.from(publicKey),
    new anchor.BN(deserializeLE(nonce).toString())
  )
  .accounts({
    computationAccount: getComputationAccAddress(program.programId, computationOffset),
    clusterAccount: arciumEnv.arciumClusterPubkey,
    // ... other accounts
  })
  .rpc();

// Wait for finalization
await awaitComputationFinalization(
  provider,
  computationOffset,
  program.programId,
  "confirmed"
);

// Decrypt result
const decrypted = cipher.decrypt([result.ciphertext], result.nonce);
```

## Local Testing

### 1. Start Arcium Local Network
```bash
# Arcium CLI automatically manages local MPC nodes via Docker
arcium test
```

### 2. Configuration for Localnet
```typescript
const arciumEnv = getArciumEnv(); // Auto-detects local environment
const clusterAccount = arciumEnv.arciumClusterPubkey;
```

### 3. Run Tests
```bash
arcium test  # Runs both Anchor tests and Arcium MPC tests
```

## Devnet Deployment

### 1. Get Cluster Offset
Register on Arcium devnet to get your cluster offset (e.g., `1078779259`)

### 2. Update Configuration
```typescript
const clusterAccount = getClusterAccAddress(1078779259); // Your cluster offset
```

### 3. Deploy
```bash
arcium deploy --cluster-offset 1078779259 \
  --keypair-path ~/.config/solana/id.json \
  -u d  # devnet
```

## Build Commands

```bash
# Build everything (Anchor program + Arcium circuits)
arcium build

# Test everything
arcium test

# Deploy to devnet
arcium deploy --cluster-offset <your-offset> -u d
```

## Key Differences from Standard Anchor

1. **Program Macro**: Use `#[arcium_program]` instead of `#[program]`
2. **Additional Imports**: `use arcium_anchor::prelude::*;`
3. **Callback Macro**: `#[arcium_callback(encrypted_ix = "function_name")]`
4. **Encrypted Circuits**: Separate `encrypted-ixs` directory with `#[encrypted]` module
5. **Dependencies**: Additional Arcium crates in Cargo.toml
6. **Rust Toolchain**: Fixed to 1.88.0
7. **Proc-macro2 Patch**: Required for compilation

## References

- **Arcium Docs**: https://docs.arcium.com/developers
- **Hello World Tutorial**: https://docs.arcium.com/developers/hello-world
- **TypeScript API**: https://ts.arcium.com/api
- **Examples**: https://github.com/arcium-hq/examples
