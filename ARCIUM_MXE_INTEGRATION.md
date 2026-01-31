# Arcium MXE Integration Comparison

This document explains the difference between the current `confidential.rs` implementation and the required `arcium_mxe.rs` implementation that uses the proper Arcium SDK.

---

## Overview

The Zyncx protocol needs to integrate with Arcium's MXE (Multi-party eXecution Environment) to enable confidential DeFi operations. The integration follows a **three-instruction pattern**:

1. **Init Computation Definition** - One-time setup to register circuits on-chain
2. **Queue Computation** - User sends encrypted inputs to MXE
3. **Callback** - MXE returns encrypted results after MPC computation

---

## Current Implementation (`confidential.rs`)

### Location
```
contracts/solana/zyncx/src/instructions/confidential.rs
```

### Architecture
```
User → queue_confidential_swap → Event Emission → (Manual Callback)
```

### What It Does
- Uses a custom `ComputationRequest` state to track requests
- Emits events that must be picked up manually
- Uses hardcoded `ARCIUM_MXE_PROGRAM_ID` as a placeholder
- No integration with actual Arcium SDK accounts

### Key Limitations

| Issue | Description |
|-------|-------------|
| No SDK Integration | Doesn't use `arcium-anchor` macros |
| Manual Callbacks | Requires manual invocation of callbacks |
| No Account Derivation | Missing `derive_mxe_pda!()`, `derive_comp_pda!()` etc. |
| No `ArgBuilder` | Doesn't properly serialize encrypted arguments |
| No Signature Verification | Doesn't verify cluster signatures on outputs |

### Code Pattern (Current)
```rust
// Current: Manual event-based approach
pub fn handler_queue_confidential_swap(
    ctx: Context<QueueConfidentialSwap>,
    params: ConfidentialSwapParams,
    proof: Vec<u8>,
) -> Result<()> {
    // Store request in custom state
    computation_request.status = ComputationStatus::Pending;
    computation_request.encrypted_strategy = params.encrypted_bounds;
    
    // Emit event - requires external listener
    emit!(ComputationQueued {
        request_id,
        user: ctx.accounts.user.key(),
        // ...
    });
    
    Ok(())
}
```

---

## Required Implementation (`arcium_mxe.rs`)

### Location (from nested folder)
```
zyncx/zyncx/contracts/solana/zyncx/src/instructions/arcium_mxe.rs
```

### Architecture
```
1. init_*_comp_def  → Register circuit on-chain (one-time)
2. queue_*         → ArgBuilder.arg().account().build() → MXE
3. *_callback      → Automatic callback with verified results
```

### What It Does
- Uses `arcium-anchor` derive macros
- Properly derives all MXE accounts using SDK macros
- Uses `ArgBuilder` to serialize encrypted inputs correctly
- Verifies cluster signatures on callback outputs
- Updates on-chain encrypted state atomically

### Key Features

| Feature | Description |
|---------|-------------|
| `#[init_computation_definition_accounts]` | Auto-derives accounts for circuit registration |
| `#[queue_computation_accounts]` | Auto-derives accounts for queuing computations |
| `#[callback_accounts]` | Auto-derives accounts for receiving results |
| `#[arcium_callback]` | Macro for callback handlers with signature verification |
| `ArgBuilder` | Proper serialization of encrypted arguments |
| `SignedComputationOutputs` | Verified outputs from cluster |

### Code Pattern (Required)
```rust
// Required: SDK-integrated approach

// 1. Init computation definition (one-time)
#[init_computation_definition_accounts("confidential_swap", payer)]
#[derive(Accounts)]
pub struct InitSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    // ... SDK-managed accounts
}

// 2. Queue computation
#[queue_computation_accounts("confidential_swap", user)]
#[derive(Accounts)]
pub struct QueueConfidentialSwapMxe<'info> {
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ...))]
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_CONFIDENTIAL_SWAP))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    // ... more SDK accounts + custom accounts
}

pub fn handler_queue_confidential_swap_mxe(...) -> Result<()> {
    // Build arguments matching circuit signature exactly
    let args = ArgBuilder::new()
        .x25519_pubkey(params.encryption_pubkey)
        .plaintext_u128(params.bounds_nonce)
        .encrypted_u64(params.encrypted_min_out)
        // ... encrypted state from accounts
        .account(
            ctx.accounts.vault.key(),
            EncryptedVaultAccount::ENCRYPTED_STATE_OFFSET,
            EncryptedVaultAccount::ENCRYPTED_STATE_SIZE,
        )
        .build();

    // Queue to MXE with automatic callback registration
    queue_computation(ctx.accounts, computation_offset, args, ...)?;
    Ok(())
}

// 3. Callback with signature verification
#[callback_accounts("confidential_swap")]
#[derive(Accounts)]
pub struct ConfidentialSwapCallbackMxe<'info> {
    #[account(address = derive_cluster_pda!(mxe_account, ...))]
    pub cluster_account: Account<'info, Cluster>,
    // ... verification accounts
}

#[arcium_callback(encrypted_ix = "confidential_swap")]
pub fn confidential_swap_callback(
    ctx: Context<ConfidentialSwapCallbackMxe>,
    output: SignedComputationOutputs<SwapOutput>,
) -> Result<()> {
    // Verify cluster signature
    let tuple = output.verify_output(
        &ctx.accounts.cluster_account,
        &ctx.accounts.computation_account,
    )?;

    // Update encrypted state atomically
    ctx.accounts.vault.vault_state = tuple.field_1.ciphertexts;
    ctx.accounts.vault.nonce = tuple.field_1.nonce;
    Ok(())
}
```

---

## Computation Definition Offsets

The SDK uses deterministic offsets derived from circuit names:

```rust
const COMP_DEF_OFFSET_INIT_VAULT: u32 = comp_def_offset("init_vault");
const COMP_DEF_OFFSET_INIT_POSITION: u32 = comp_def_offset("init_position");
const COMP_DEF_OFFSET_PROCESS_DEPOSIT: u32 = comp_def_offset("process_deposit");
const COMP_DEF_OFFSET_CONFIDENTIAL_SWAP: u32 = comp_def_offset("confidential_swap");
const COMP_DEF_OFFSET_COMPUTE_WITHDRAWAL: u32 = comp_def_offset("compute_withdrawal");
const COMP_DEF_OFFSET_CLEAR_POSITION: u32 = comp_def_offset("clear_position");
```

These must match the Arcis circuit function names in `encrypted-ixs/src/lib.rs`.

---

## Account Derivation Comparison

| Account | Current | Required |
|---------|---------|----------|
| MXE Account | Hardcoded `ARCIUM_MXE_PROGRAM_ID` | `derive_mxe_pda!()` |
| Computation | Custom `ComputationRequest` PDA | `derive_comp_pda!(offset, mxe)` |
| Comp Def | Not used | `derive_comp_def_pda!(OFFSET)` |
| Cluster | Not used | `derive_cluster_pda!(mxe)` |
| Mempool | Not used | `derive_mempool_pda!(mxe)` |
| Exec Pool | Not used | `derive_execpool_pda!(mxe)` |
| Sign PDA | Not used | `derive_sign_pda!()` |

---

## Data Flow Comparison

### Current Flow
```
1. User calls queue_confidential_swap
2. ComputationRequest created with encrypted_strategy
3. Event emitted (ComputationQueued)
4. External system must:
   - Listen for event
   - Call Arcium manually
   - Call confidential_swap_callback manually
5. Callback updates state
```

### Required Flow
```
1. (One-time) Admin calls init_*_comp_def for each circuit
2. User calls queue_confidential_swap_mxe
3. ArgBuilder serializes encrypted inputs
4. queue_computation CPI to Arcium program
5. Arcium MXE nodes process encrypted data
6. Arcium automatically calls *_callback with signed results
7. Callback verifies signature and updates state
```

---

## Missing Cargo Dependencies

The current `Cargo.toml` needs these dependencies for full SDK integration:

```toml
[dependencies]
arcium-anchor = { version = "0.6", features = ["cpi"] }
arcium-client = "0.6"
arcium-macros = "0.6"
```

---

## Files to Add/Modify

| File | Action | Description |
|------|--------|-------------|
| `instructions/arcium_mxe.rs` | **Add** | Full SDK integration |
| `instructions/mod.rs` | Modify | Add `pub mod arcium_mxe;` |
| `lib.rs` | Modify | Add instruction handlers |
| `Cargo.toml` | Modify | Add arcium dependencies |
| `state/arcium_mxe.rs` | Keep | Already has encrypted account structures |

---

## Summary

| Aspect | Current (`confidential.rs`) | Required (`arcium_mxe.rs`) |
|--------|----------------------------|---------------------------|
| SDK Usage | None | Full `arcium-anchor` |
| Account Derivation | Manual PDAs | SDK macros |
| Argument Serialization | Manual | `ArgBuilder` |
| Callback Registration | Event-based | `queue_computation` with callbacks |
| Output Verification | None | `SignedComputationOutputs` |
| Circuit Registration | Not supported | `init_comp_def` |
| Integration Effort | Requires external orchestrator | Fully on-chain |

The `arcium_mxe.rs` implementation provides a **complete, production-ready** integration with Arcium's MXE that works entirely on-chain without external orchestration.

---

## Why Full Integration Was Not Possible

### The Toolchain Incompatibility

The Arcium SDK (`arcium-anchor`, `arcium-client`, `arcium-macros`) requires **Rust edition 2024**, which in turn requires **Cargo 1.85+**. However, Solana's build toolchain creates a blocking incompatibility:

```
error: failed to parse manifest at `.../arcium-anchor-0.6.6/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires Rust 1.85.0 (nightly), but the currently running version is Cargo 1.84.0
```

### Technical Details

| Component | Version | Limitation |
|-----------|---------|------------|
| System Rust | 1.89.0 | ✅ Supports edition2024 |
| System Cargo | 1.89.0 | ✅ Supports edition2024 |
| Solana CLI | 3.0.13 | Uses bundled cargo-build-sbf |
| `cargo-build-sbf` | Bundled | ❌ Uses **Cargo 1.84.0** internally |
| Arcium SDK | 0.6.6 | ❌ Requires **Cargo 1.85+** |

The `cargo-build-sbf` tool (used by `anchor build`) bundles its own version of Cargo (1.84.0) which **cannot be overridden**. This means even though the system has a newer Cargo, Solana programs are built with the older bundled version.

### Attempted Solutions

1. **Direct dependency addition** - Failed due to edition2024 requirement
2. **Platform SDK update** - Solana's SDK lags behind Rust ecosystem
3. **Feature flags** - Arcium SDK doesn't provide edition2021-compatible builds

### Current Workaround

We implemented a **mock version** of `arcium_mxe.rs` that:

- ✅ Maintains the same account structures and interfaces
- ✅ Uses the same three-instruction pattern (init_comp_def, queue, callback)
- ✅ Emits events for external MPC orchestration
- ✅ Compiles with current Solana toolchain
- ❌ Does not make actual CPI calls to Arcium MXE
- ❌ Requires external service to process queued computations

### Upgrade Path

When Solana's `cargo-build-sbf` is updated to use Cargo 1.85+:

1. Uncomment the Arcium dependencies in `Cargo.toml`:
   ```toml
   arcium-anchor = "0.6.6"
   arcium-client = "0.6.6"
   arcium-macros = "0.6.6"
   ```

2. Replace mock implementations with actual SDK macros:
   ```rust
   // Replace mock account derivation with:
   #[queue_computation_accounts(
       process_deposit,
       payer = user
   )]
   ```

3. Replace event emissions with actual CPI calls to Arcium MXE

The interface is designed to be **drop-in compatible** with the real SDK once the toolchain supports it.
