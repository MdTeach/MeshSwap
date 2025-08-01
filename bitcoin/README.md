# Bitcoin CLI with HTLC Support

Modern Bitcoin development CLI with wallet management and Hash Time Locked Contracts. Built with expert-level Rust patterns and modular architecture.

## Quick Start

```bash
just start                    # Start Bitcoin regtest
just balance-admin           # Check balance (starts with 10 BTC)
just send-admin-to-maker 2.5 # Send Bitcoin

# Modern CLI with improved UX
cargo run -- balance --wallet wallet/admin.toml
cargo run -- send --from wallet/admin.toml --to wallet/maker.toml --amount 1.5
cargo run -- address --wallet wallet/maker.toml

just stop                    # Stop
```

## Architecture

### 🏗️ Modular Design
- **`blockchain.rs`**: RPC client management
- **`wallet.rs`**: Wallet operations with BitcoinWallet struct  
- **`transaction.rs`**: Transaction building with Builder pattern
- **`error.rs`**: Custom error types with proper handling
- **`taproot.rs`**: HTLC/Taproot contract functionality
- **`utils.rs`**: Backward compatibility layer

## HTLC Overview

Hash Time Locked Contracts (HTLCs) enable trustless atomic swaps with two spending paths:

1. **Secret Path**: Recipient can claim funds by revealing the secret preimage
2. **Timeout Path**: Sender can reclaim funds after a specified block height if unclaimed

This enables atomic swaps without requiring trust between parties.

## Commands

### 🚀 Direct CLI Usage (Modern)
```bash
# Balance operations
cargo run -- balance --wallet wallet/admin.toml
cargo run -- balance --wallet wallet/maker.toml

# Address operations  
cargo run -- address --wallet wallet/admin.toml

# Send operations
cargo run -- send --from wallet/admin.toml --to wallet/maker.toml --amount 1.5
cargo run -- send --from wallet/maker.toml --to wallet/taker.toml --amount 0.5
```

### ⚡ Just Shortcuts (Convenience)
```bash
# Network Management
just start                    # Start regtest
just stop                     # Stop and cleanup
just clean                    # Reset blockchain state

# Balance Commands (with improved output)
just balance-admin           # Shows: "Balance: 6.9999436 BTC (699994360 sats)"
just balance-maker           # Clean formatting with emoji indicators
just balance-taker

# Send Commands (with transaction IDs)
just send-admin-to-maker 5.0 # ✅ Enhanced output with TXID
just send-admin-to-taker 3.0 # 📊 Amount confirmation
just send-maker-to-taker 1.0 # 🔗 Transaction ID display

# Address Commands
just address-admin           # Get receiving addresses
just address-maker
just address-taker
```

## Features

### ✨ Expert Rust Implementation
- **🏗️ Modular Architecture**: Clean separation of concerns
- **🔧 Builder Pattern**: TransactionBuilder for flexible transaction construction
- **⚡ Async/Await**: Modern async programming throughout
- **🎯 Strong Typing**: Custom types for addresses, amounts, configurations
- **🛡️ Error Handling**: Custom error enums with proper trait implementations
- **📦 Memory Safety**: Efficient use of references and ownership

### 🚀 Enhanced User Experience
- **Clean Output**: Formatted BTC amounts (e.g., "2.5" instead of "2.50000000")
- **Transaction IDs**: Full TXID display for verification
- **Emoji Indicators**: Visual feedback for successful operations
- **Incremental Balance**: Real-time balance updates with automine rewards
- **Backward Compatibility**: Existing Just commands continue to work

### 🔗 Bitcoin Integration
- **Regtest Network**: Safe development environment
- **BDK Integration**: Modern Bitcoin development kit
- **Automine System**: Automatic block generation with rewards
- **Wallet Isolation**: Different derivation paths for wallet separation
- **Fee Management**: Configurable fee rates (default: 20 sat/vByte)

## Testing

```bash
# Basic functionality test
just start
just balance-admin          # Should show ~10 BTC initially
just send-admin-to-maker 0.5
just balance-maker          # Should show 0.5 BTC received
just stop
```

Built with **modern Rust + BDK + Bitcoin Core** for professional Bitcoin development.