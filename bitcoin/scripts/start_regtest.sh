#!/bin/bash

# Bitcoin regtest startup script with automine
set -e

# Configuration
BLOCK_TIME=${BLOCK_TIME:-10}  # Default 10 seconds, configurable via environment variable
AUTOMINE=${AUTOMINE:-true}    # Enable automine by default

echo "🚀 Starting Bitcoin regtest node with automine..."
echo "⏰ Block time: ${BLOCK_TIME} seconds"

# Create data directory if it doesn't exist
mkdir -p "./data"

# Check if bitcoind is already running
if pgrep -f "bitcoind.*regtest" > /dev/null; then
    echo "⚠️  Bitcoin Core is already running in regtest mode"
    exit 0
fi

# Start Bitcoin daemon in regtest mode
bitcoind \
  -regtest \
  -daemon \
  -server \
  -datadir=./data \
  -rpcuser=bitcoin \
  -rpcpassword=bitcoin \
  -rpcport=18443 \
  -port=18444 \
  -fallbackfee=0.0002 \
  -rpcallowip=127.0.0.1 \
  -txindex=1

# Wait for the daemon to start
echo "⏳ Waiting for Bitcoin daemon to start..."
sleep 3

# Check if the daemon is running
echo "🧪 Checking Bitcoin daemon status..."
if bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 getblockchaininfo > /dev/null 2>&1; then
    echo "✅ Bitcoin regtest node is running!"
    echo "📍 RPC endpoint: http://127.0.0.1:18443"
    echo "🔑 Credentials: bitcoin/bitcoin"
    
    # Create BDK wallet
    echo "💰 Creating BDK wallet..."
    bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 createwallet "bdk_wallet" 2>/dev/null || echo "Wallet already exists"
    
    # Create admin wallet for mining
    echo "⛏️  Creating admin mining wallet..."
    bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 createwallet "admin" 2>/dev/null || {
        echo "Admin wallet already exists, loading it..."
        bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 loadwallet "admin" 2>/dev/null || true
    }
    
    # Generate mining address from admin wallet
    echo "🎯 Generating admin mining address..."
    MINING_ADDRESS=$(bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin getnewaddress "admin_mining" 2>/dev/null || bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin getnewaddress)
    echo "⛏️  Mining address: $MINING_ADDRESS"
    
    # Mine initial blocks for coinbase maturity to Admin Bitcoin Core wallet
    echo "🏗️  Mining initial 101 blocks for coinbase maturity..."
    bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 generatetoaddress 101 "$MINING_ADDRESS" > /dev/null
    
    # Now send some coins to the BDK admin wallet for testing
    echo "💸 Funding BDK admin wallet..."
    # Use the known BDK admin wallet address (derived from the standard mnemonic and path)
    # This address corresponds to the admin wallet with derivation path m/84h/1h/0h
    BDK_ADMIN_ADDRESS="bcrt1qmflavul2k53n45lz360278cfgr4nzahh2f2f43"
    echo "📍 BDK admin address: $BDK_ADMIN_ADDRESS"
    
    # Send 10 BTC to admin wallet for testing (this makes just balance-admin show incrementing balance)
    echo "💸 Sending initial funds to BDK admin wallet..."
    sleep 2  # Wait for wallet to be fully ready
    bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin sendtoaddress "$BDK_ADMIN_ADDRESS" 10 || {
        echo "⚠️  Failed to send initial funds, trying again..."
        sleep 2
        bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin sendtoaddress "$BDK_ADMIN_ADDRESS" 10 || echo "⚠️  Could not send initial funds to BDK admin"
    }
    
    # Mine one more block to confirm the transaction and provide coinbase maturity
    bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 generatetoaddress 1 "$MINING_ADDRESS" > /dev/null
    
    if [ "$AUTOMINE" = "true" ]; then
        echo "🔄 Starting automine with ${BLOCK_TIME}s intervals..."
        
        # Create automine script
        cat > /tmp/bitcoin_automine.sh << EOF
#!/bin/bash
MINING_ADDRESS="$MINING_ADDRESS"
BDK_ADMIN_ADDRESS="$BDK_ADMIN_ADDRESS"
BLOCK_COUNTER=0
while true; do
    sleep $BLOCK_TIME
    if pgrep -f "bitcoind.*regtest" > /dev/null; then
        bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 generatetoaddress 1 "\$MINING_ADDRESS" > /dev/null 2>&1
        BLOCK_COUNT=\$(bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 getblockcount 2>/dev/null)
        echo "\$(date): Mined block #\$BLOCK_COUNT"
        
        # Every 10 blocks, send some rewards to BDK wallets for testing
        BLOCK_COUNTER=\$((BLOCK_COUNTER + 1))
        if [ \$((BLOCK_COUNTER % 10)) -eq 0 ]; then
            # Send small amounts to BDK admin wallet to simulate rewards
            bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin sendmany "" '{"\$BDK_ADMIN_ADDRESS":0.1}' > /dev/null 2>&1 || true
            echo "\$(date): Sent rewards to BDK wallets"
        fi
    else
        echo "Bitcoin daemon stopped, exiting automine"
        break
    fi
done
EOF
        
        chmod +x /tmp/bitcoin_automine.sh
        
        # Start automine in background
        nohup /tmp/bitcoin_automine.sh > automine.log 2>&1 &
        AUTOMINE_PID=$!
        echo $AUTOMINE_PID > automine.pid
        
        echo "⚡ Automine started (PID: $AUTOMINE_PID)"
        echo "📋 Automine logs: automine.log"
        echo "🛑 To stop automine: scripts/stop_bitcoind.sh"
    fi
    
    echo ""
    echo "🎯 Bitcoin Network Ready!"
    echo "⛏️  Automine: $AUTOMINE (${BLOCK_TIME}s intervals)"
    echo "💰 Admin wallet balance: $(bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcwallet=admin getbalance) BTC"
    echo ""
   
else
    echo "❌ Failed to start Bitcoin daemon"
    exit 1
fi