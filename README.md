# ZYNCX

**Privacy-first DeFi protocol for anonymous transactions and swaps on Solana**

[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://explorer.solana.com/?cluster=devnet)
[![Noir](https://img.shields.io/badge/ZK-Noir-black)](https://noir-lang.org)
[![Arcium](https://img.shields.io/badge/MPC-Arcium-blue)](https://arcium.com)

---

## What is ZYNCX?

ZYNCX is a privacy-preserving DeFi protocol built on Solana that lets you deposit, withdraw, and swap tokens without revealing your transaction history. Think of it as a privacy layer for DeFi—once your funds enter the shielded pool, the link between your deposit and withdrawal is cryptographically broken.

### Why ZYNCX?

- **Break the Link**: Deposit and withdraw without anyone knowing which deposit belongs to you
- **MEV Protection**: Swap tokens with encrypted minimum output thresholds to prevent front-running
- **Fast**: ~400ms transaction times on Solana (vs. 15+ seconds on Ethereum)
- **Partial Withdrawals**: Withdraw any amount while keeping the rest private
- **DeFi Integration**: Swap through Jupiter DEX while maintaining privacy

---

## How It Works

ZYNCX combines three powerful technologies:

### 1. Zero-Knowledge Proofs (Noir)

When you deposit, you receive a secret note. When you withdraw, you prove you own *some* deposit in the pool without revealing which one. It's like proving you have a winning lottery ticket without showing the ticket number.

```
Deposit:  "I'm putting 10 SOL into the pool"
          → Get secret note (keep this safe!)

Withdraw: "I own some deposit with 10 SOL"
          → ZK proof verifies this WITHOUT revealing which deposit
          → No one can link your withdrawal to your deposit
```

### 2. MEV Protection (Arcium MXE)

When swapping tokens, your minimum acceptable output is encrypted. MEV bots can see the current price but not your threshold, preventing sandwich attacks and front-running.

```
You want to swap SOL → USDC with minimum 100 USDC
  ↓
Your minimum (100) is ENCRYPTED
  ↓
MEV bots see: current output = 102 USDC, swap executed = true
MEV bots DON'T see: your minimum threshold
  ↓
No front-running possible!
```

### 3. Merkle Tree Commitments

Your deposit creates a cryptographic commitment stored in an on-chain Merkle tree. You can prove membership without revealing your position.

---

## Quick Start

### For Users

1. **Connect your Solana wallet** (Phantom, Solflare, etc.)
2. **Deposit SOL** into the privacy vault
3. **Save your secret note** (you'll need this to withdraw!)
4. **Wait a bit** (optional, for better privacy)
5. **Withdraw** to any address using your secret note

### For Developers

```bash
# Clone the repo
git clone https://github.com/yourusername/zyncx.git
cd zyncx

# Install dependencies
yarn install

# Build the Solana program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run the frontend
cd app
npm install
npm run dev
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    ZYNCX Protocol                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  User → Frontend → Solana Program → Arcium MXE         │
│           ↓            ↓                ↓               │
│      Noir Circuit  Merkle Tree   Encrypted State       │
│      (ZK Proofs)   (On-Chain)    (MEV Protection)      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Tech Stack

| Component | Technology | Purpose |
|-----------|------------|----------|
| **Smart Contracts** | Anchor (Rust) | On-chain logic |
| **ZK Proofs** | Noir | Privacy-preserving withdrawals |
| **MEV Protection** | Arcium MXE | Encrypted swap parameters |
| **Cryptography** | Poseidon Hash | ZK-friendly commitments |
| **DEX Integration** | Jupiter | Token swaps |
| **Frontend** | Next.js + TypeScript | User interface |

---

## Features

### What ZYNCX Hides

- **Withdrawal-to-Deposit Link**: No one can tell which deposit a withdrawal came from
- **Swap Minimum Output**: Your slippage tolerance is encrypted
- **Internal Balance Changes**: Vault state updates happen in encrypted memory
- **Partial Withdrawal Patterns**: Each withdrawal creates a fresh commitment

### What ZYNCX Doesn't Hide (Blockchain Limitations)

- **Deposit Amounts**: Visible when you deposit (use standard amounts like 1, 10, 100 SOL)
- **Withdrawal Amounts**: Visible when you withdraw (use multiple partial withdrawals)
- **Timestamps**: Block times are public (wait random periods before withdrawing)
- **Gas Patterns**: Transaction metadata is public (use relayers for better privacy)

**Privacy Tip**: The more people use ZYNCX, the larger your anonymity set becomes!

---

## Project Structure

```
zyncx/
├── contracts/solana/zyncx/    # Solana smart contract (Anchor)
│   ├── src/
│   │   ├── lib.rs            # Program entry point
│   │   ├── instructions/     # Deposit, withdraw, swap logic
│   │   ├── state/            # Vault, Merkle tree, nullifiers
│   │   └── dex/              # Jupiter integration
│   └── Cargo.toml
│
├── encrypted-ixs/             # Arcium MPC circuits
│   └── src/lib.rs            # init_vault, process_deposit, confidential_swap
│
├── mixer/                     # Noir ZK circuits
│   └── src/main.nr           # Withdrawal proof circuit
│
├── app/                       # Next.js frontend
│   ├── components/           # React components
│   ├── lib/                  # SDK, crypto utilities
│   └── pages/                # UI pages
│
└── scripts/                   # Deployment & testing scripts
```

---

## Usage Examples

### Deposit SOL

```typescript
import { useZyncx } from '@/lib';

const { depositSol } = useZyncx();

// Deposit 1 SOL and get a secret note
const note = await depositSol(1.0);

// IMPORTANT: Save this note! You need it to withdraw
console.log('Secret Note:', note);
// Store it securely (encrypted backup, password manager, etc.)
```

### Withdraw SOL

```typescript
import { useZyncx, decodeNote } from '@/lib';

const { withdrawSol } = useZyncx();

// Paste your secret note
const note = decodeNote('eyJzZWNyZXQiOi...');

// Withdraw 0.5 SOL (partial withdrawal)
await withdrawSol(note, 0.5);

// Or withdraw everything
await withdrawSol(note);
```

### Private Swap

```typescript
// Coming soon: Swap SOL → USDC with MEV protection
const { swapTokens } = useZyncx();

await swapTokens({
  note: mySecretNote,
  fromToken: 'SOL',
  toToken: 'USDC',
  amount: 1.0,
  minOutput: 100, // Encrypted! MEV bots can't see this
});
```

---

## Security

### Trust Assumptions

| Component | Trust Model |
|-----------|-------------|
| **Noir ZK Proofs** | Cryptographic soundness (audited) |
| **Arcium MXE** | Honest threshold of MPC nodes |
| **Solana Runtime** | Validator consensus |
| **Poseidon Hash** | Collision resistance |

### User Security Best Practices

1. **NEVER lose your secret note** - Without it, your funds are unrecoverable
2. **Store notes offline** - Use encrypted backups, not browser storage
3. **Use fresh addresses** - Withdraw to new addresses to prevent correlation
4. **Wait before withdrawing** - Immediate withdrawal can link you to your deposit
5. **Use standard amounts** - Deposit 1, 10, or 100 SOL instead of odd amounts like 7.3284 SOL

### Known Limitations

- **Merkle tree capacity**: ~1M commitments per vault
- **Root history**: Last 30 roots stored (old roots expire)
- **Anonymity set**: Privacy improves with more users

---

## Comparison with Other Privacy Protocols

| Feature | ZYNCX | Tornado Cash | Aztec |
|---------|-------|--------------|-------|
| **Blockchain** | Solana | Ethereum | Ethereum L2 |
| **Speed** | ~400ms | ~15s | ~minutes |
| **Partial Withdrawals** | Yes | No | Yes |
| **DeFi Integration** | Jupiter | No | Limited |
| **MEV Protection** | Encrypted | N/A | Partial |
| **Proof System** | Noir (PLONK) | Groth16 | PLONK |

---

## Development

### Building from Source

```bash
# Build Solana program
anchor build

# Build Noir circuit
cd mixer
nargo compile

# Build Arcium circuits
cd encrypted-ixs
cargo build-sbf

# Build frontend
cd app
npm run build
```

### Testing

```bash
# Run Solana program tests
anchor test

# Test Noir circuit
cd mixer
nargo test

# Run frontend tests
cd app
npm test
```

### Deployment

```bash
# Deploy to Solana devnet
anchor deploy --provider.cluster devnet

# Initialize a vault
ts-node scripts/init-vault.ts

# Deploy frontend
cd app
vercel deploy
```

---

## Documentation

- **[PROTOCOL.md](./PROTOCOL.md)** - Deep dive into privacy guarantees and technical architecture
- **[ROADMAP.md](./ROADMAP.md)** - Development roadmap and future features
- **[docs/](./docs/)** - Integration guides and API reference

---

## Roadmap

### Completed (v0.3.0)
- Private deposits and withdrawals
- Partial withdrawal support
- MEV-protected swaps via Arcium
- Jupiter DEX integration
- Next.js frontend

### In Progress
- Multi-asset support (USDC, USDT, etc.)
- Relayer network for gas abstraction
- Mobile wallet support

### Future
- Cross-chain privacy bridges
- DCA (Dollar Cost Averaging) with encrypted parameters
- Limit orders with hidden thresholds
- Governance token and DAO

---

## Contributing

We welcome contributions! Here's how you can help:

1. **Report bugs** - Open an issue with reproduction steps
2. **Suggest features** - Share your ideas in discussions
3. **Submit PRs** - Fix bugs or add features (check ROADMAP.md first)
4. **Improve docs** - Help make our documentation clearer

### Development Setup

```bash
# Fork and clone the repo
git clone https://github.com/yourusername/zyncx.git

# Create a feature branch
git checkout -b feature/amazing-feature

# Make your changes and test
anchor test

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature

# Open a Pull Request
```

---

## License

MIT License - see [LICENSE](./LICENSE) for details

---

## Acknowledgments

- **[Arcium](https://arcium.com)** - Multi-party computation infrastructure
- **[Noir](https://noir-lang.org)** - Zero-knowledge proof framework
- **[Jupiter](https://jup.ag)** - Best-in-class DEX aggregator on Solana
- **[Tornado Cash](https://tornado.cash)** - Pioneer in privacy protocols

---

## Contact & Community

- **Twitter**: [@zyncx_protocol](https://twitter.com/zyncx_protocol)
- **Discord**: [Join our community](https://discord.gg/zyncx)
- **Docs**: [docs.zyncx.io](https://docs.zyncx.io)
- **Email**: team@zyncx.io

---

<div align="center">

**Built for privacy on Solana**

*Version 0.3.0 | February 2026*

</div>
