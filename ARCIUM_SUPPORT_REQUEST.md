# Message to Arcium Team - MXE Initialization Support Request

---

## For Discord (#support channel) or Email (support@arcium.com)

---

**Subject:** MXE Initialization Failing on Devnet - ConstraintExecutable Error 2007

---

Hi Arcium Team,

I'm building **ZYNCX**, a privacy-preserving DeFi protocol on Solana that combines Noir ZK-SNARKs with Arcium MXE for confidential swap verification and MEV protection. I've successfully integrated Arcium into my Solana program and compiled my circuits, but I'm encountering a blocking error during MXE initialization on devnet.

---

## 🔴 Problem Summary

**Issue:** MXE computation definition initialization fails with `ConstraintExecutable` error

**Error Code:** 2007

**Impact:** Cannot initialize Arcium MXE cluster, blocking confidential swap functionality

---

## 📋 Technical Details

### **My Deployed Program:**
```
Program ID: 6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC
Network: Devnet
IDL Account: 748qBPYRKJEgTWSPzkRnQetmWcKSumfnuwvGESmwWzUE
Size: 845,168 bytes
Status: Deployed and executable
```

**Explorer:** https://explorer.solana.com/address/6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC?cluster=devnet

### **Arcium Configuration:**
```toml
[dependencies]
arcium-client = "0.6.3"
arcium-macros = "0.6.3"
arcium-anchor = "0.6.3"

[features]
idl-build = ["anchor-lang/idl-build", "arcium-macros/idl-build"]
```

**Cluster Configuration:**
```
cluster-offset: 456 (recommended for v0.6.3)
recovery-set-size: 4
```

### **Circuits Compiled Successfully:**
```
✅ init_vault: 464,029,697 ACUs
✅ process_deposit: 141,557,761 ACUs  
✅ confidential_swap: 194,084,865 ACUs

All circuits built without errors using arcium-cli
```

---

## ❌ Exact Error

When running `arcium deploy --skip-deploy` or attempting manual MXE initialization:

```
Error: SolanaClientError(
  RpcError(
    RpcResponseError {
      code: -32002,
      message: "Transaction simulation failed: Error processing Instruction 0: 
               custom program error: 0x7d7",
      data: SendTransactionPreflightFailure(
        RpcSimulateTransactionResult {
          err: Some(InstructionError(0, Custom(2007))),
          logs: Some([
            "Program log: AnchorError caused by account: mxe_program. 
             Error Code: ConstraintExecutable. 
             Error Number: 2007. 
             Error Message: An executable constraint was violated."
          ])
        }
      )
    }
  )
)
```

**Specific Error:**
- Account: `mxe_program`
- Error: `ConstraintExecutable` (#2007)
- Message: "An executable constraint was violated"

---

## 🔧 What I've Tried

### 1. **Verified Program Deployment** ✅
```bash
$ solana program show 6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC --url devnet

Program Id: 6Pqr8ipXuXdmXRaSgLZpV8ukoHJEwZtBxybBSwuJwNPC
Owner: BPFLoaderUpgradeab1e11111111111111111111111
Authority: 45hQzq6zkohyRkqgNhiYk2rrTDLLAySAcTrRfarFN81R
```
✅ Program is properly deployed and marked as executable

### 2. **Tried Different Cluster Offsets**
- ❌ 123 (failed)
- ❌ 456 (recommended, failed)
- ❌ 789 (failed)

### 3. **Command Variations**
```bash
# Standard deployment
arcium deploy

# Skip program deployment (program already deployed)
arcium deploy --skip-deploy

# Manual initialization via TypeScript
npx ts-node scripts/init-mxe.ts
```
All result in the same `ConstraintExecutable` error

### 4. **Checked Account Addresses**
```rust
// My program code
#[account(
    executable,
    address = crate::ARCIUM_PROGRAM_ID
)]
pub mxe_program: AccountInfo<'info>,
```

Using: `Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ` (from Arcium docs)

---

## 🤔 Possible Causes

Based on my investigation:

1. **MXE Program Address Issue?**
   - Is the devnet MXE program at `Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ`?
   - Is this address outdated for v0.6.3?

2. **Cluster Access Permissions?**
   - Do I need credentials/allowlist for devnet cluster access?
   - Is there a registration process I missed?

3. **Cluster Availability?**
   - Is the devnet MXE cluster currently operational?
   - Are there known issues with cluster-offset 456?

4. **SDK Version Mismatch?**
   - Should I use a different version than 0.6.3?
   - Any known breaking changes I should be aware of?

---

## ✅ What IS Working

**My Integration:**
- ✅ Program compiles with `arcium-anchor` macros
- ✅ IDL builds successfully with `idl-build` feature
- ✅ `#[encrypted]` module compiles without errors
- ✅ All 3 circuits compile successfully
- ✅ Program deployed and executable on devnet
- ✅ Computation definition structs defined correctly
- ✅ No stack overflow issues (warning only)

**Core Functionality:**
- ✅ Deposits work (without MXE)
- ✅ Withdrawals work (with Noir ZK verification)
- ✅ Swaps work (without confidential MXE layer)

**Only blocker:** MXE initialization for confidential computation

---

## 🎯 What I Need Help With

### 1. **Correct MXE Program Address for Devnet**
```
Question: What is the correct Arcium MXE program ID for devnet on v0.6.3?
Current: Arcj82pX7HxYKLR92qvgZUAd7vGS1k4hQvAFcPATFdEQ
Is this correct?
```

### 2. **Cluster Access**
```
Question: Do I need special credentials or allowlist approval for devnet?
Process: How do I register for cluster access?
```

### 3. **Recommended Cluster Configuration**
```
Question: What cluster-offset and recovery-set-size should I use for devnet?
Current: offset=456, size=4
Are these correct?
```

### 4. **Debug Steps**
```
Question: Are there additional diagnostic commands I can run?
Example: How to verify cluster health/availability?
```

### 5. **Alternative Approach**
```
Question: Is there a simpler way to test MXE on devnet?
Example: Minimal working example or test program?
```

---

## 📁 Additional Context

### **My Use Case:**
ZYNCX uses Arcium MXE for three confidential computations:

1. **`init_vault`**: Initialize encrypted vault state
2. **`process_deposit`**: Track confidential deposits
3. **`confidential_swap`**: Execute swaps with MEV protection

**Privacy Architecture:**
- Noir ZK-SNARKs: Unlinkable deposits/withdrawals ✅ (working)
- Arcium MXE: Confidential swap verification ⏳ (blocked)
- Jupiter Integration: DEX routing ✅ (working)

### **Circuit Code Sample:**
```rust
#[encrypted]
mod circuits {
    use arcis::*;

    #[derive(Clone, ArcisSerialize, ArcisDeserialize)]
    pub struct ConfidentialSwapInput {
        pub amount_in: u64,
        pub min_amount_out: u64,
        pub slippage_tolerance: u16,
    }

    #[instruction]
    pub fn confidential_swap(
        input: Enc<Shared, ConfidentialSwapInput>
    ) -> Enc<Shared, SwapResult> {
        // Circuit logic
        // ...
    }
}
```

### **Documentation I've Followed:**
- ✅ https://docs.arcium.com/developers/quick-start
- ✅ https://docs.arcium.com/developers/deployment
- ✅ https://docs.arcium.com/developers/computation-definition
- ✅ GitHub examples: arcium-io/arcium-examples

---

## 💻 My Environment

```
OS: macOS
Solana CLI: 1.18.x
Anchor: 0.32.1
Arcium CLI: Latest (from cargo install)
Rust: 1.75+
Node: 23.11.1
```

**Wallet:**
```
Address: 45hQzq6zkohyRkqgNhiYk2rrTDLLAySAcTrRfarFN81R
Balance: 6.25 SOL (devnet)
```

---

## 🚀 Project Status

**Timeline:** Working toward demo in 1-2 weeks

**Progress:**
- ✅ 95% complete (all core features working)
- ⏳ 5% blocked by MXE initialization

**Fallback Plan:**
- Can demo without MXE (using regular swaps)
- Would prefer to show full confidential computation

---

## 📞 How to Reach Me

**GitHub:** [your-github-handle]
**Discord:** [your-discord-handle]
**Email:** [your-email]

**Preferred Contact:** Discord for real-time troubleshooting

**Availability:** 
- Timezone: UTC+5:30 (IST)
- Available: Most evenings/weekends

---

## 🙏 Request

Could you please help me:

1. **Verify** the correct MXE program address for devnet
2. **Provide** cluster access if needed (credentials/allowlist)
3. **Suggest** the correct cluster configuration for my setup
4. **Share** any known issues with devnet cluster
5. **Point me** to additional debugging resources

I'm happy to:
- Provide more logs/traces if needed
- Test patches/fixes
- Share my code for review
- Join a call for real-time debugging

---

## 📎 Attachments

**Full Error Logs:** [Available upon request]

**Program Code:** 
- Repo: [link to repo if public]
- Or can share relevant files privately

**Build Artifacts:**
```
✅ zyncx.so (845 KB)
✅ zyncx.json (IDL)
✅ init_vault.arcis
✅ process_deposit.arcis
✅ confidential_swap.arcis
```

---

**Thank you for your time and support!** 

I'm excited about Arcium's capabilities and eager to showcase ZYNCX as a real-world use case for privacy-preserving DeFi on Solana. Looking forward to getting this resolved so I can complete my integration.

Best regards,
[Your Name]

---

**PS:** If there's a better time to reach out (e.g., office hours, specific support queue), please let me know and I'll follow that process instead.

**PPS:** Happy to provide any additional technical details, run diagnostic commands, or test experimental features if that helps with troubleshooting!
