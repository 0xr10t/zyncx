# 🎬 ZYNCX Demo Guide for Hackathon Judges

## Quick Start (5 minutes)

### 1. Deploy Contracts to Devnet

```bash
# Ensure you have SOL on devnet
solana config set --url devnet
solana airdrop 2

# Build and deploy
anchor build --no-idl
anchor deploy --provider.cluster devnet
```

### 2. Initialize the Vault

```bash
npx ts-node scripts/init-vault.ts
```

### 3. Start the Frontend

```bash
cd app
npm run dev
# Open http://localhost:3000
```

---

## Demo Flow for Video

### Scene 1: Introduction (30 seconds)
- Show the landing page with the 3D Spline background
- Highlight the tagline: "Privacy Protocol. Turbocharged."
- Mention the three pillars: ZK Proofs, Confidential Computation, DEX Integration

### Scene 2: Connect Wallet (15 seconds)
- Click "Select Wallet" → Connect Phantom (devnet)
- Show wallet connected state

### Scene 3: Shield Funds (Deposit) (1 minute)
- Scroll to Privacy Vault section
- Enter amount (e.g., 0.5 SOL)
- Click "Shield Funds"
- **IMPORTANT**: Show the secret note being generated
- Demonstrate copying/downloading the note
- Show transaction on Solana Explorer (devnet)

### Scene 4: Explain Privacy (30 seconds)
- Explain that the deposit creates a cryptographic commitment
- Funds go into a shared vault (anonymity set)
- No on-chain link between depositor and future withdrawer

### Scene 5: Unshield Funds (Withdraw) (1 minute)
- Switch to "Withdraw" tab
- Paste the secret note
- Click "Unshield Funds"
- Show ZK proof being verified
- Funds appear in wallet without any link to original deposit

### Scene 6: Architecture (30 seconds)
- Scroll to "How It Works" section
- Explain the three layers:
  1. **Layer 1**: ZK-SNARKs for transaction privacy
  2. **Layer 2**: Arcium MXE for confidential computation
  3. **Layer 3**: Jupiter for private swaps

### Scene 7: Code Highlights (Optional, 1 minute)
- Show Noir ZK circuit (`mixer/src/main.nr`)
- Show confidential swap implementation
- Show Jupiter integration

---

## Key Talking Points

### What Makes ZYNCX Unique?

1. **Three-Layer Privacy Stack**
   - Most privacy protocols only use ZK proofs
   - We add Arcium's FHE/MPC for confidential trading logic
   - Jupiter integration for best swap rates

2. **Solana-Native**
   - Built with Anchor framework
   - Uses Solana's alt_bn128 precompiles for efficient ZK verification
   - Sub-second finality

3. **Production-Ready Architecture**
   - Merkle tree with root history (prevents front-running)
   - Nullifier system (prevents double-spending)
   - Poseidon hashing (ZK-friendly)

### Technical Innovations

- **Confidential Swaps**: Users can swap tokens without revealing trading strategy
- **Call-and-Callback Pattern**: Arcium MXE processes encrypted bounds
- **Jupiter CPI**: Best swap routes across all Solana DEXs

---

## Test Accounts (Devnet)

Program ID: `6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6`

Get devnet SOL:
```bash
solana airdrop 2 <YOUR_ADDRESS> --url devnet
```

---

## Troubleshooting

### "Wallet not connected"
- Make sure Phantom is set to devnet
- Refresh the page after switching networks

### "Transaction failed"
- Check you have enough SOL for fees (~0.01 SOL)
- The vault may need to be initialized first

### "Invalid note"
- Make sure you copied the entire note (no spaces)
- Notes are base64 encoded JSON

---

## Project Statistics

- **Smart Contracts**: 16 Rust files, ~2,500 lines
- **ZK Circuit**: 315 lines of Noir
- **Frontend**: Next.js 16 with glassmorphic UI
- **Test Coverage**: 24/24 tests passing
- **Build Status**: All green

---

## Links

- **GitHub**: https://github.com/0xr10t/zyncx
- **Solana Explorer**: https://explorer.solana.com/address/6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6?cluster=devnet

---

