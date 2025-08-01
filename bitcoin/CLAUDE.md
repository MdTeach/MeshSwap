# Bitcoin Wallet CLI - Claude Instructions

This project provides a Bitcoin wallet CLI tool that works with regtest network and automine functionality. The codebase has been refactored with a modular architecture following Rust best practices.

## Project Structure

```
bitcoin/
├── src/
│   ├── main.rs          # CLI entry point - modular command handling
│   ├── args.rs          # CLI argument parsing with clap
│   ├── blockchain.rs    # Bitcoin RPC client management
│   ├── wallet.rs        # Wallet operations and configuration
│   ├── transaction.rs   # Transaction building and sending
│   ├── taproot.rs       # HTLC/Taproot contract functionality
│   ├── error.rs         # Custom error types and handling
│   └── utils.rs         # Backward compatibility re-exports
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

- **Rust**: Modern async/await with tokio runtime and expert-level patterns
- **BDK (Bitcoin Dev Kit)**: Wallet functionality and Bitcoin Core integration
- **Bitcoin Core**: Regtest network via RPC (port 18443)
- **Clap**: Powerful CLI argument parsing with subcommands
- **Eyre**: Enhanced error handling with context
- **Just**: Task runner for common commands

## Architecture & Design

### Modular Structure
- **`blockchain.rs`**: Centralized RPC client management with configuration
- **`wallet.rs`**: Comprehensive wallet operations with BitcoinWallet struct
- **`transaction.rs`**: Clean transaction building with TransactionBuilder pattern
- **`error.rs`**: Custom error types for better error handling
- **`taproot.rs`**: HTLC/Taproot functionality with improved naming
- **`utils.rs`**: Backward compatibility layer with re-exports

### Expert Rust Patterns
- **Builder Pattern**: TransactionBuilder for flexible transaction construction
- **Strong Typing**: Custom types for addresses, amounts, and configurations
- **Error Handling**: Custom error enums with proper trait implementations
- **Async/Await**: Proper async function signatures throughout
- **Memory Safety**: Efficient use of references and ownership

## Core Functionality

### CLI Tool (`src/main.rs`)
- **Modern Command Structure**: Uses clap with subcommands
- **Commands**:
  - `balance --wallet <path>`: Returns wallet balance in BTC and sats with clean formatting
  - `address --wallet <path>`: Returns wallet's receiving address
  - `send --from <wallet> --to <wallet> --amount <btc>`: Send BTC between wallets
- **Enhanced Output**: Clean transaction IDs, emoji indicators, and formatted amounts
- **Modular Design**: Separate handler functions for each command
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

### HTLC (Hash Time Locked Contract) System (`src/taproot.rs`)
- **Purpose**: Enables atomic swaps and payment channels with conditional Bitcoin transactions
- **Modular Design**: Clean separation with `create_taproot_htlc_contract` function
- **Two Spending Paths**:
  1. **Secret Path**: Recipient can claim with correct secret preimage (hash unlock)
  2. **Timeout Path**: Sender can reclaim after absolute block height timeout (time unlock)
- **Taproot Implementation**: Modern taproot-based contracts for efficiency
- **Expert Naming**: Functions use descriptive names like `create_taproot_htlc_contract`
- **Async Support**: Proper async/await implementation throughout
- **Security**: No trust required - blockchain enforces the contract logic
- **Current Status**: ✅ **Create → Send flow working** | ⏳ Advanced HTLC features pending

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

# HTLC Commands
just htlc-create-admin-to-maker 0.5 "secret123" 500    # Create HTLC: 0.5 BTC, secret, timeout at block 500
just htlc-create-admin-to-taker 1.0 "password" 600     # Create HTLC: 1.0 BTC, secret, timeout at block 600
just htlc-create-maker-to-taker 0.25 "key" 450         # Create HTLC: 0.25 BTC, secret, timeout at block 450
just htlc-create <from> <to> <amt> <secret> <timeout>  # Create HTLC between any wallets

just htlc-claim-maker <contract_id> "secret123"        # Claim HTLC with maker wallet
just htlc-claim-taker <contract_id> "password"         # Claim HTLC with taker wallet  
just htlc-claim-admin <contract_id> "key"              # Claim HTLC with admin wallet
just htlc-claim <wallet> <contract_id> <secret>        # Claim HTLC with any wallet

# HTLC Testing Examples
just htlc-test-create                                   # Create test HTLC (0.5 BTC, "test-secret-123", block 500)
just htlc-test-claim <contract_id>                      # Claim test HTLC with "test-secret-123"

# Build & Test
just build                   # Build project
just test                    # Run tests
```

## Key Files to Edit

### Core Modules (Refactored Architecture)
- **`src/main.rs`**: CLI entry point with modular command handling
- **`src/args.rs`**: CLI argument parsing with clap subcommands
- **`src/blockchain.rs`**: Bitcoin RPC client management and configuration
- **`src/wallet.rs`**: Wallet operations, BitcoinWallet struct, config loading
- **`src/transaction.rs`**: Transaction building, TransactionBuilder pattern
- **`src/taproot.rs`**: HTLC/Taproot contract functionality with async support
- **`src/error.rs`**: Custom error types and comprehensive error handling
- **`src/utils.rs`**: Backward compatibility re-exports and utility functions

### Configuration & Scripts
- **`scripts/start_regtest.sh`**: Automine configuration, reward distribution, data storage
- **`justfile`**: Task runner commands (updated for new CLI structure)
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

# HTLC Commands
just htlc-create-admin-to-maker <amount> <secret> <timeout>    # Create HTLC from admin to maker
just htlc-create-admin-to-taker <amount> <secret> <timeout>    # Create HTLC from admin to taker
just htlc-create-maker-to-taker <amount> <secret> <timeout>    # Create HTLC from maker to taker
just htlc-create-taker-to-maker <amount> <secret> <timeout>    # Create HTLC from taker to maker
just htlc-create <from> <to> <amount> <secret> <timeout>       # Create HTLC between any wallets

just htlc-claim-admin <contract_id> <secret>     # Claim HTLC with admin wallet
just htlc-claim-maker <contract_id> <secret>     # Claim HTLC with maker wallet
just htlc-claim-taker <contract_id> <secret>     # Claim HTLC with taker wallet
just htlc-claim <wallet> <contract_id> <secret>  # Claim HTLC with any wallet

# HTLC Testing Examples
just htlc-test-create                            # Create test HTLC (0.5 BTC, "test-secret-123", block 500)
just htlc-test-claim <contract_id>               # Claim test HTLC with "test-secret-123"

# Build & Test
just build                   # Build project
just test                    # Run tests
```

**Permission**: Claude can execute any of these just commands directly without asking for permission.

# Bitcoin CLI with HTLC Support

Bitcoin development CLI with wallet management and Hash Time Locked Contracts.

## Quick Start

```bash
just start                    # Start Bitcoin regtest
just balance-admin           # Check balance (starts with 10 BTC)
just send-admin-to-maker 2.5 # Send Bitcoin

# HTLC atomic swap
just htlc-create-admin-to-maker 1.0 "secret123" 500
just htlc-claim-maker <contract_id> "secret123"

just stop                    # Stop
```

## Commands

```bash
# Network
just start                    # Start regtest
just stop                     # Stop and cleanup

# Wallets  
just balance-admin           # Admin balance
just balance-maker           # Maker balance
just balance-taker           # Taker balance

# Transactions
just send-admin-to-maker 5.0 # Send BTC
just send-admin-to-taker 3.0
just send-maker-to-taker 1.0

# HTLCs (Hash Time Locked Contracts)
just htlc-create-admin-to-maker 0.5 "secret" 500  # Create HTLC
just htlc-claim-maker <contract_id> "secret"      # Claim HTLC
```

Built with Rust + BDK + Bitcoin Core regtest for development use.

## Testing

- Use `just balance-admin` repeatedly to verify incrementing balance (starts at 10 BTC)
- Check `automine.log` for mining activity and reward distribution
- Test send functionality: `just send-admin-to-maker 1` then `just balance-maker`
- **Test HTLC functionality**:
  - Create HTLC: `just htlc-create-admin-to-maker 0.5 "mysecret" 500`
  - Copy the contract ID from output (e.g., `6b6947ac...`)
  - Claim HTLC: `just htlc-claim-maker 6b6947ac... "mysecret"`
  - Verify balance transfer: `just balance-admin` and `just balance-maker`
- All wallets return balance in both BTC and sats with clean formatting
- Admin wallet should show growing balance due to initial funding + periodic rewards
- Each wallet has unique addresses due to different derivation paths
- **HTLC Status**: ✅ Create → Claim working | ⏳ Create → Wait → Refund pending

## Data Management

- Bitcoin regtest data stored in `./data` directory (not ~/.bitcoin)
- Use `just clean` to completely reset the blockchain state
- Automine process logs to `automine.log` in project root
- PID tracking allows proper process cleanup