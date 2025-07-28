# Bitcoin Wallet System with Regtest Integration

This project implements a Bitcoin wallet system with HTLC (Hash Time Locked Contract) functionality, integrated with Bitcoin's regtest network for development and testing.

## Prerequisites

1. **Bitcoin Core**: Install Bitcoin Core from https://bitcoin.org/en/download/
2. **Rust**: Make sure you have Rust installed from https://rustup.rs/

## Setup Instructions

### 1. Start Bitcoin Regtest Node

Run the provided script to start a Bitcoin regtest node:

```bash
./start_regtest.sh
```

Or manually start bitcoind:

```bash
bitcoind -regtest -daemon -server -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443 -rpcallowip=127.0.0.1 -fallbackfee=0.0002
```

### 2. Run the Application

```bash
cargo run
```

## What the Application Does

1. **Wallet Management**: Creates and manages three wallets:
   - **Admin**: Acts as the miner, receives block rewards
   - **Maker**: Initiates swaps
   - **Taker**: Responds to swaps

2. **Mining**: The admin wallet mines 101 blocks to generate spendable Bitcoin (coinbase maturity requirement)

3. **Balance Tracking**: Shows wallet balances before and after mining

4. **HTLC Deployment**: Deploys Hash Time Locked Contracts for atomic swaps

## Wallet Configuration

Wallets are configured via TOML files in the `wallet/` directory:

- `wallet/admin.toml` - Miner wallet configuration
- `wallet/maker.toml` - Swap initiator configuration  
- `wallet/taker.toml` - Swap responder configuration

Each wallet uses BIP39 mnemonic phrases for deterministic key generation.

## Network Configuration

The system connects to Bitcoin regtest at:
- **RPC URL**: http://127.0.0.1:18443
- **Credentials**: bitcoin/bitcoin
- **Network**: Regtest

## Key Features

- **BDK Integration**: Uses Bitcoin Development Kit for wallet management
- **Regtest Mining**: Admin wallet can mine blocks programmatically
- **Blockchain Sync**: Wallets sync with the regtest blockchain
- **Address Generation**: Each wallet can generate new addresses
- **Balance Queries**: Real-time balance checking with blockchain sync

## Stopping the Bitcoin Node

To stop the regtest node:

```bash
bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin stop
```

## Troubleshooting

If you encounter connection errors:

1. Ensure Bitcoin Core is installed and `bitcoind` is in your PATH
2. Check that the regtest node is running: `bitcoin-cli -regtest getblockchaininfo`
3. Verify the RPC credentials match the configuration
4. Make sure port 18443 is available and not blocked by firewall