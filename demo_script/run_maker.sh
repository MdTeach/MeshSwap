#!/bin/bash

echo "🔨 Running Maker - Creating Cross-Chain Order..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Go to 1inch directory
cd ./1inch

echo "💼 Creating cross-chain swap order..."
echo "📝 This will generate an atomic swap order with hash locks..."

npx tsx src/maker.ts

echo ""
echo "✅ Cross-chain order creation completed! 🎉"
echo "📄 Order details saved to order.json"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"