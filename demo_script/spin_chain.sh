#!/bin/bash

echo "🚀 Starting MeshSwap chain deployment..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo "🧽 Cleaning and starting Bitcoin chain..."
cd bitcoin
just clean-start

echo ""
echo "⚡ Starting Ethereum chain in new terminal..."
osascript -e 'tell app "Terminal" to do script "cd '"$(pwd)"'/../ethereum && just start"'

echo ""
echo "⏳ Waiting for Ethereum chain to be ready..."
cd ../ethereum

# Wait for Ethereum to be ready by checking if we can connect
while ! curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://localhost:8545 > /dev/null 2>&1; do
    echo "⏸️  Ethereum not ready yet, waiting 2 seconds..."
    sleep 2
done

echo "🎯 Ethereum is ready! Deploying contracts..."
just deploy

echo ""
echo "🔧 Initializing the contract in 1inch..."
cd ../1inch && npx tsx src/init_contract.ts && cd ..

echo ""
echo "✅ Both chains are running and contracts deployed! 🎉"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"