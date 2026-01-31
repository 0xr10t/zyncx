#!/bin/bash

# ZYNCX Devnet Deployment Script
# Run this to deploy the contracts to Solana devnet

set -e

echo "ZYNCX Devnet Deployment"
echo "=========================="

# Check if solana CLI is available
if ! command -v solana &> /dev/null; then
    echo "Solana CLI not found. Please install it first."
    exit 1
fi

# Check if anchor CLI is available
if ! command -v anchor &> /dev/null; then
    echo "Anchor CLI not found. Please install it first."
    exit 1
fi

# Set cluster to devnet
echo "Setting cluster to devnet..."
solana config set --url devnet

# Check wallet balance
BALANCE=$(solana balance 2>/dev/null || echo "0")
echo "Current wallet balance: $BALANCE"

# Request airdrop if balance is low
if [[ $(echo "$BALANCE" | cut -d' ' -f1 | cut -d'.' -f1) -lt 2 ]]; then
    echo "Requesting SOL airdrop..."
    solana airdrop 2 || echo "Airdrop may have failed (rate limited). Continue anyway."
    sleep 5
fi

# Build the program
echo "Building program..."
anchor build --no-idl

# Deploy the program
echo "Deploying to devnet..."
anchor deploy --provider.cluster devnet

echo ""
echo "Deployment complete!"
echo ""
echo "Program ID: 6Qm7RAmYr8bQxeg2YdxX3dtJwNkKcQ3b7zqFTeZYvTx6"
echo ""
echo "Next steps:"
echo "1. Initialize the SOL vault by running the init script"
echo "2. Start the frontend: cd app && npm run dev"
echo "3. Connect your wallet and test deposits/withdrawals"
