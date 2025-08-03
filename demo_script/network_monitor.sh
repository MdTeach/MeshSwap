#!/bin/bash

echo "🌐 MeshSwap Network Monitor"
echo "╭─────────────────────────────────────────────────────────╮"
echo "│  🔍 Real-time monitoring of Bitcoin & Ethereum chains   │  "
echo "│  📡 Press Ctrl+C to stop monitoring                     │"
echo "╰─────────────────────────────────────────────────────────╯"
echo ""

while true; do
    clear
    echo "🌐 MeshSwap Network Monitor"
    echo "╭─────────────────────────────────────────────────────────╮"
    echo "│  🔍 Real-time monitoring of Bitcoin & Ethereum chains   │"
    echo "│  📡 Press Ctrl+C to stop monitoring                     │"
    echo "╰─────────────────────────────────────────────────────────╯"
    echo ""
    echo "🕐 $(date '+%Y-%m-%d %H:%M:%S') - Live Chain Status"
    echo ""
    
    # Get Bitcoin block info via RPC
    BTC_RESPONSE=$(curl -s -u bitcoin:bitcoin -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"1.0","method":"getblockcount","params":[],"id":1}' \
        http://127.0.0.1:18443 2>/dev/null)
    BTC_BLOCK=$(echo $BTC_RESPONSE | grep -o '"result":[0-9]*' | cut -d':' -f2 || echo "N/A")
    
    if [ "$BTC_BLOCK" != "N/A" ]; then
        BTC_HASH_RESPONSE=$(curl -s -u bitcoin:bitcoin -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"1.0","method":"getblockhash","params":['$BTC_BLOCK'],"id":1}' \
            http://127.0.0.1:18443 2>/dev/null)
        BTC_HASH=$(echo $BTC_HASH_RESPONSE | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | cut -c1-16 || echo "N/A")
        BTC_STATUS="🟢 ONLINE"
    else
        BTC_HASH="N/A"
        BTC_STATUS="🔴 OFFLINE"
    fi
    
    # Get Ethereum block info
    ETH_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8545 2>/dev/null)
    ETH_BLOCK_HEX=$(echo $ETH_RESPONSE | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    if [ -n "$ETH_BLOCK_HEX" ]; then
        ETH_BLOCK=$((16#${ETH_BLOCK_HEX#0x}))
        
        ETH_HASH_RESPONSE=$(curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}' \
            http://localhost:8545 2>/dev/null)
        ETH_HASH=$(echo $ETH_HASH_RESPONSE | grep -o '"hash":"[^"]*"' | cut -d'"' -f4 | cut -c1-18 || echo "N/A")
        ETH_STATUS="🟢 ONLINE"
    else
        ETH_BLOCK="N/A"
        ETH_HASH="N/A"
        ETH_STATUS="🔴 OFFLINE"
    fi
    
    echo "🟠 BITCOIN REGTEST - $BTC_STATUS"
    echo "   🧱 Block Height: $BTC_BLOCK"
    echo "   🔗 Block Hash:   ${BTC_HASH}..."
    echo "   🌐 RPC Endpoint: http://127.0.0.1:18443"
    echo ""
    echo "⚡ ETHEREUM ANVIL - $ETH_STATUS"
    echo "   🧱 Block Height: $ETH_BLOCK"
    echo "   🔗 Block Hash:   ${ETH_HASH}..."
    echo "   🌐 RPC Endpoint: http://localhost:8545"
    echo ""
    echo "🔄 Refreshing in 5 seconds..."
    
    sleep 5
done