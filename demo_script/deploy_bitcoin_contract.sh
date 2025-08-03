#!/bin/bash

echo "🟠 Deploying Bitcoin contracts and initializing swap..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Go to bitcoin directory
echo "💰 Executing swap from admin to maker (1 BTC)..."
cd bitcoin
just swap-admin-to-maker 1
echo ""
echo "✅ Bitcoin contract deployment completed! 🎉"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"