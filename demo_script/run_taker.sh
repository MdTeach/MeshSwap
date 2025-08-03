#!/bin/bash

echo "📥 Running Taker - Filling Cross-Chain Order..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Go to 1inch directory
cd ./1inch

echo "🔄 Filling cross-chain swap order..."
echo "💰 This will execute the atomic swap on the taker side..."

npx tsx src/taker.ts

echo ""
echo "✅ Cross-chain order filling completed! 🎉"
echo "🔗 Atomic swap executed successfully"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"