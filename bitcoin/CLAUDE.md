# Bitcoin Wallet CLI - Claude Instructions

This project provides a Bitcoin wallet CLI tool that works with regtest network and automine functionality.

## Project Structure

```
bitcoin/
├── src/
│   ├── main.rs          # CLI entry point - returns wallet balance in satoshis
│   ├── args.rs          # CLI argument parsing (clap)
│   ├── wallet.rs        # Wallet management (BDK integration)
│   ├── contract.rs      # HTLC contract functionality
│   └── deployment.rs    # Contract deployment utilities
├── scripts/
│   ├── start_regtest.sh # Start Bitcoin regtest with automine
│   └── stop_bitcoind.sh # Stop Bitcoin regtest and cleanup
├── wallet/
│   ├── admin.toml       # Admin wallet config (miner wallet)
│   ├── maker.toml       # Maker wallet config  
│   └── taker.toml       # Taker wallet config
├── justfile             # Just command definitions
└── Cargo.toml          # Rust dependencies
```

## Key Technologies

- **Rust**: Main language with tokio async runtime
- **BDK (Bitcoin Dev Kit)**: Wallet functionality and Bitcoin Core integration
- **Bitcoin Core**: Regtest network via RPC (port 18443)
- **Clap**: CLI argument parsing
- **Just**: Task runner for common commands

## Core Functionality

### CLI Tool (`src/main.rs`)
- Takes `--wallet <path>` argument pointing to TOML config file
- Returns wallet balance in satoshis (not BTC)
- Only works with regtest network (hardcoded)
- Uses BDK to sync with Bitcoin Core RPC

### Automine System (`scripts/start_regtest.sh`)
- Automatically mines blocks at configurable intervals (default: 10s)
- Admin wallet receives mining rewards (acts as miner)
- Mines initial 101 blocks for coinbase maturity
- Sends periodic rewards (0.1 BTC every 10 blocks) to BDK admin wallet
- Creates background process with PID tracking and logging

### Wallet Configuration
TOML format with:
```toml
[wallet]
name = "admin"
type = "miner" 
network = "regtest"

[keys]
mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
derivation_path = "m/84h/1h/0h"

[config]
electrum_url = "tcp://localhost:50001"
block_height = 0
```

## Important Implementation Details

### Admin Wallet Balance Increments
- Bitcoin Core creates separate "admin" wallet for mining
- BDK admin wallet (from admin.toml) receives periodic funding transfers
- Uses hardcoded address `bcrt1qrh98qvlnec9k9au5auntfj3y2tmmw9w0emnpvh` (BDK admin wallet index 0)
- Balance grows over time as automine sends rewards

### Coinbase Maturity
- Bitcoin requires 100 blocks for coinbase transactions to be spendable
- Script mines 100 blocks initially, then continues with 1 block at configured intervals
- Admin wallet accumulates mature coinbase rewards over time

### Configuration Options
- `BLOCK_TIME`: Mining interval in seconds (default: 10)
- `AUTOMINE`: Enable/disable automatic mining (default: true)

## Common Just Commands

```bash
just start                    # Start regtest (10s blocks)
just start-fast 3            # Start with 3s blocks  
just start-no-mine           # Start without automine
just stop                    # Stop regtest and automine

just balance-admin           # Get admin wallet balance (increments over time)
just balance-maker           # Get maker wallet balance
just balance-taker           # Get taker wallet balance
just balance <wallet.toml>   # Get any wallet balance

just build                   # Build project
just test                    # Run tests
```

## Key Files to Edit

- **`src/main.rs`**: CLI logic, wallet balance retrieval
- **`scripts/start_regtest.sh`**: Automine configuration, reward distribution
- **`justfile`**: Add new commands
- **`wallet/*.toml`**: Wallet configurations

## Dependencies & Setup

Requires:
- Bitcoin Core installed with regtest support
- Rust toolchain
- Just task runner (optional but recommended)

The system automatically handles Bitcoin Core startup, wallet creation, and mining rewards distribution.

## Testing

- Use `just balance-admin` repeatedly to verify incrementing balance
- Check `automine.log` for mining activity
- All wallets return balance in satoshis (1 BTC = 100,000,000 satoshis)
- Admin wallet should show growing balance due to mining rewards