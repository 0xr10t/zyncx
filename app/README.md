# ZYNCX Frontend

High-tech, futuristic Web3 interface for the ZYNCX privacy protocol on Solana.

## Features

- **Cyberpunk Design**: Inspired by modern Web3 aesthetics with glowing effects, glass morphism, and animated gradients
- **Wallet Integration**: Phantom & Solflare wallet support via Solana Wallet Adapter
- **Responsive**: Mobile-first design that works on all devices
- **Animated**: Smooth animations using Framer Motion
- **Privacy-First UI**: Clean interface for deposits, withdrawals, and swaps

## Tech Stack

- **Next.js 16** - React framework with App Router
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **Framer Motion** - Animations
- **Solana Web3.js** - Blockchain interaction
- **Wallet Adapter** - Multi-wallet support

## Getting Started

Install dependencies:
```bash
npm install
```

Run the development server:
```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to view the app.

## Project Structure

```
app/
├── app/
│   ├── page.tsx          # Main landing page
│   ├── layout.tsx        # Root layout with wallet provider
│   └── globals.css       # Global styles with cyberpunk theme
├── components/
│   ├── Navbar.tsx        # Navigation with wallet button
│   ├── HeroSection.tsx   # Hero with animated background
│   ├── PrivacyVault.tsx  # Deposit/Withdraw/Swap interface
│   ├── HowItWorks.tsx    # Architecture explanation
│   ├── Footer.tsx        # Footer with links
│   └── WalletProvider.tsx # Solana wallet context
└── tailwind.config.ts    # Custom theme with cyber colors
```

## Design System

### Colors
- **Cyber Purple**: `#8B5CF6` - Primary brand color
- **Cyber Blue**: `#3B82F6` - Secondary accent
- **Cyber Cyan**: `#06B6D4` - Tertiary accent

### Effects
- **Glass Effect**: Frosted glass morphism with backdrop blur
- **Text Glow**: Neon text shadow effects
- **Border Glow**: Animated border gradients on hover
- **Floating Orbs**: Background ambient animations

## Integration with Smart Contracts

The frontend is ready to integrate with the Zyncx Solana program:
- Program ID configuration in wallet provider
- Account derivation for vaults and merkle trees
- Transaction building for deposits/withdrawals
- ZK proof generation (to be implemented with SDK)

## Deploy

Deploy to Vercel:
```bash
vercel deploy
```

Or build for production:
```bash
npm run build
npm start
```
