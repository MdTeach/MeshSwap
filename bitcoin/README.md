# Bitcoin Wallet CLI

A simple CLI tool to get Bitcoin wallet balance from TOML configuration files.

## Features

- **Automine**: Automatically mines blocks at configurable intervals (default: 10s)
- **Regtest Network**: Local Bitcoin network for development
- **Multiple Wallets**: Pre-configured admin, maker, and taker wallets
- **Real-time Balance**: Get wallet balance in real-time as blocks are mined

## Quick Start with Just

```bash
# Start Bitcoin regtest with automine (mines blocks every 10s)
just start

# Get wallet balance (updates as blocks are mined)
just balance-maker

# Stop Bitcoin regtest and automine
just stop
```

## Just Commands

```bash
just start                    # Start regtest with automine (10s blocks)
just start-fast 3            # Start with faster mining (3s blocks)
just start-no-mine           # Start without automine (manual mining only)
just stop                    # Stop regtest and automine

just balance-maker           # Get maker wallet balance
just balance-admin           # Get admin wallet balance  
just balance-taker           # Get taker wallet balance
just balance wallet/my.toml  # Get custom wallet balance

just build                   # Build project
just test                    # Run tests
```

**Automine**: Continuously mines blocks in the background. Check `automine.log` for mining activity.

**Block Time**: Configurable mining interval (default 10s). Faster times = more frequent transactions.

Returns wallet balance in satoshis (1 BTC = 100,000,000 satoshis).