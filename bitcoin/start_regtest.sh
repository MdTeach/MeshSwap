#!/bin/bash

# Bitcoin regtest startup script
echo "Starting Bitcoin regtest node..."

# Create data directory if it doesn't exist
mkdir -p ~/.bitcoin/regtest

# Start Bitcoin daemon in regtest mode
bitcoind \
  -regtest \
  -daemon \
  -server \
  -rpcuser=bitcoin \
  -rpcpassword=bitcoin \
  -rpcport=18443 \
  -port=18444 \
  -fallbackfee=0.0002 \
  -rpcallowip=127.0.0.1 \
  -datadir=~/.bitcoin/regtest \
  -txindex=1

# Wait a moment for the daemon to start
sleep 2

# Check if the daemon is running
echo "Checking Bitcoin daemon status..."
bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin getblockchaininfo

echo "Bitcoin regtest node is running!"
echo "RPC endpoint: http://127.0.0.1:18443"
echo "Credentials: bitcoin/bitcoin"