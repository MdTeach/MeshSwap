#!/bin/bash

echo "🛑 Stopping MeshSwap chains..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "🟠 Stopping Bitcoin chain..."
cd bitcoin
just stop

echo ""
echo "⚡ Stopping Ethereum (Anvil) chain..."
# Kill all anvil processes
pkill -f "anvil"

# Also kill any processes running on port 8545
lsof -ti:8545 | xargs kill -9 2>/dev/null || true

echo ""
echo "✅ Both chains have been stopped! 🔴"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"