#!/bin/bash

if [ $# -eq 0 ]; then
    echo "❌ Error: Please provide the swap secret as an argument"
    echo "Usage: $0 <SWAP_SECRET>"
    echo "Example: $0 abc123def456..."
    exit 1
fi

SWAP_SECRET=$1

echo "💸 Withdrawing Maker Funds from Bitcoin..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Go to bitcoin directory
cd ./bitcoin

echo "🔓 Using swap secret to withdraw 1 BTC to maker..."
echo "🔑 Secret: ${SWAP_SECRET:0:8}************${SWAP_SECRET: -8}"

just withdraw-maker-from-admin 1 $SWAP_SECRET

echo ""
echo "✅ Maker withdrawal completed! 🎉"
echo "💰 Funds transferred from admin to maker wallet"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"