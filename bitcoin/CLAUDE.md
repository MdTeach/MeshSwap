# Bitcoin Wallet CLI - Claude Instructions

This project provides a Bitcoin wallet CLI tool that works with regtest network and automine functionality.

## Project Structure

```
bitcoin/
├── src/
│   ├── main.rs          # CLI entry point with balance, address, send commands
│   └── args.rs          # CLI argument parsing (clap)
├── scripts/
│   ├── start_regtest.sh # Start Bitcoin regtest with automine
│   └── stop_bitcoind.sh # Stop Bitcoin regtest and cleanup
├── wallet/
│   ├── admin.toml       # Admin wallet config (miner wallet)
│   ├── maker.toml       # Maker wallet config  
│   └── taker.toml       # Taker wallet config
├── data/                # Bitcoin regtest data directory (auto-created)
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
- **Commands**:
  - `balance` (default): Returns wallet balance in BTC and sats with clean formatting
  - `address`: Returns wallet's receiving address
  - `send --to <wallet.toml> --amount <btc>`: Send BTC between wallets
- Only works with regtest network (hardcoded)
- Uses BDK with proper BIP32 derivation paths for wallet isolation
- Clean BTC formatting (removes trailing zeros, e.g., "2.5" instead of "2.50000000")

### Automine System (`scripts/start_regtest.sh`)
- **Data Storage**: Uses `./data` directory for Bitcoin regtest data (not ~/.bitcoin)
- Automatically mines blocks at configurable intervals (default: 10s)
- Admin wallet receives mining rewards (acts as miner)
- Mines initial 101 blocks for coinbase maturity
- Sends initial 10 BTC to BDK admin wallet address
- Sends periodic rewards (0.1 BTC every 10 blocks) to BDK admin wallet
- Creates background process with PID tracking and logging

### Wallet Configuration
TOML format with **different derivation paths for wallet isolation**:
```toml
[wallet]
name = "admin"
type = "miner" 
network = "regtest"

[keys]
mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
derivation_path = "m/84h/1h/0h"  # Admin: m/84h/1h/0h, Maker: m/84h/1h/1h, Taker: m/84h/1h/2h

[config]
electrum_url = "tcp://localhost:50001"
block_height = 0
```

## Important Implementation Details

### Admin Wallet Balance System
- Bitcoin Core creates separate "admin" wallet for mining (Bitcoin Core wallet)
- BDK admin wallet (from admin.toml) receives funding from Bitcoin Core admin wallet
- Uses actual BDK admin address `bcrt1qmflavul2k53n45lz360278cfgr4nzahh2f2f43` (derived from m/84h/1h/0h)
- **Initial funding**: 10 BTC sent during startup
- Balance grows over time as automine sends periodic rewards (0.1 BTC every 10 blocks)

### Coinbase Maturity
- Bitcoin requires 100 blocks for coinbase transactions to be spendable
- Script mines 101 blocks initially (1 extra to make first block spendable)
- Admin wallet accumulates mature coinbase rewards over time

### Configuration Options
- `BLOCK_TIME`: Mining interval in seconds (default: 10)
- `AUTOMINE`: Enable/disable automatic mining (default: true)

## Common Just Commands

```bash
# Network Management
just start                    # Start regtest (10s blocks) - data stored in ./data
just start-fast 3            # Start with 3s blocks  
just start-no-mine           # Start without automine
just stop                    # Stop regtest and automine
just clean                   # Stop regtest and clear ./data directory

# Balance Commands
just balance-admin           # Get admin wallet balance (starts at 10 BTC, increments over time)
just balance-maker           # Get maker wallet balance
just balance-taker           # Get taker wallet balance
just balance <wallet.toml>   # Get any wallet balance

# Address Commands
just address-admin           # Get admin wallet address
just address-maker           # Get maker wallet address
just address-taker           # Get taker wallet address
just address <wallet.toml>   # Get any wallet address

# Send Commands
just send-admin-to-maker 5   # Send 5 BTC from admin to maker
just send-admin-to-taker 3   # Send 3 BTC from admin to taker
just send-maker-to-taker 1   # Send 1 BTC from maker to taker
just send <from> <to> <amt>  # Send between any wallets

# Build & Test
just build                   # Build project
just test                    # Run tests
```

## Key Files to Edit

- **`src/main.rs`**: CLI logic, wallet balance/address/send functionality
- **`src/args.rs`**: CLI argument parsing and subcommands
- **`scripts/start_regtest.sh`**: Automine configuration, reward distribution, data storage
- **`justfile`**: Task runner commands
- **`wallet/*.toml`**: Wallet configurations with derivation paths

## Dependencies & Setup

Requires:
- Bitcoin Core installed with regtest support
- Rust toolchain
- Just task runner (optional but recommended)

The system automatically handles Bitcoin Core startup, wallet creation, and mining rewards distribution.

## Just Commands Available to Claude

Claude has access to all just commands and can run them directly without asking permission:

```bash
# Network Management
just start                    # Start regtest (10s blocks)
just start-fast <seconds>     # Start with custom block time
just start-no-mine           # Start without automine
just stop                    # Stop regtest and automine
just clean                   # Stop regtest and clear data

# Balance Commands
just balance-admin           # Get admin wallet balance
just balance-maker           # Get maker wallet balance  
just balance-taker           # Get taker wallet balance
just balance <wallet.toml>   # Get any wallet balance

# Address Commands
just address-admin           # Get admin wallet address
just address-maker           # Get maker wallet address
just address-taker           # Get taker wallet address
just address <wallet.toml>   # Get any wallet address

# Send Commands
just send-admin-to-maker <amount>    # Send BTC from admin to maker
just send-admin-to-taker <amount>    # Send BTC from admin to taker
just send-maker-to-taker <amount>    # Send BTC from maker to taker
just send-taker-to-maker <amount>    # Send BTC from taker to maker
just send <from> <to> <amount>       # Send between any wallets

# Build & Test
just build                   # Build project
just test                    # Run tests
```

**Permission**: Claude can execute any of these just commands directly without asking for permission.

## Testing

- Use `just balance-admin` repeatedly to verify incrementing balance (starts at 10 BTC)
- Check `automine.log` for mining activity and reward distribution
- Test send functionality: `just send-admin-to-maker 1` then `just balance-maker`
- All wallets return balance in both BTC and sats with clean formatting
- Admin wallet should show growing balance due to initial funding + periodic rewards
- Each wallet has unique addresses due to different derivation paths

## Data Management

- Bitcoin regtest data stored in `./data` directory (not ~/.bitcoin)
- Use `just clean` to completely reset the blockchain state
- Automine process logs to `automine.log` in project root
- PID tracking allows proper process cleanup