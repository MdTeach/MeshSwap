#!/bin/bash

echo "💰 MeshSwap Balance Checker"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🟠 BITCOIN REGTEST BALANCES"
echo "───────────────────────────"

# Get resolver (admin) wallet balance
cd bitcoin
echo "🤖 Resolver Balance:"
just balance-admin

echo ""
echo "🔨 Maker Balance:"
just balance-maker
echo ""
echo "⚡ ETHEREUM ANVIL BALANCES"
echo "─────────────────────────"

# Run the balance checker from 1inch directory
cd ../1inch
echo "🔍 Fetching WETH balances from smart contracts..."
npx tsx src/balance-checker.ts

echo ""
echo "✅ Balance check completed!"
echo "═══════════════════════════════════════════════════════════"