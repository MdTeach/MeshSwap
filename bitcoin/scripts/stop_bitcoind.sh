#!/bin/bash

# Stop Bitcoin Core regtest and automine
set -e

echo "🛑 Stopping Bitcoin Core and automine..."

# Stop automine process if running
if [ -f automine.pid ]; then
    AUTOMINE_PID=$(cat automine.pid)
    if kill -0 "$AUTOMINE_PID" 2>/dev/null; then
        kill "$AUTOMINE_PID"
        echo "⚡ Automine stopped (PID: $AUTOMINE_PID)"
    fi
    rm -f automine.pid
fi

# Stop bitcoind gracefully using RPC
if bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 stop 2>/dev/null; then
    echo "✅ Bitcoin Core stopped gracefully"
else
    echo "⚠️  RPC stop failed, trying to kill process..."
    
    # Find and kill bitcoind process
    BITCOIN_PID=$(pgrep -f "bitcoind.*regtest" || true)
    
    if [ -n "$BITCOIN_PID" ]; then
        kill "$BITCOIN_PID"
        echo "✅ Bitcoin Core process killed"
    else
        echo "ℹ️  Bitcoin Core was not running"
    fi
fi

# Clean up any remaining automine processes
pkill -f bitcoin_automine.sh 2>/dev/null || true

# Clean up temp files
rm -f /tmp/bitcoin_automine.sh automine.log

# Wait a moment for cleanup
sleep 2

# Verify it's stopped
if pgrep -f "bitcoind.*regtest" > /dev/null; then
    echo "❌ Bitcoin Core is still running!"
    exit 1
else
    echo "✅ Bitcoin Core and automine stopped successfully"
fi