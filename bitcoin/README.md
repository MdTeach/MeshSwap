# Bitcoin CLI with HTLC Support

Bitcoin development CLI with wallet management and Hash Time Locked Contracts.

## Quick Start

```bash
just start                    # Start Bitcoin regtest
just balance-admin           # Check balance (starts with 10 BTC)
just send-admin-to-maker 2.5 # Send Bitcoin

# HTLC atomic swap
just htlc-create-admin-to-maker 1.0 "secret123" 500
just htlc-claim-maker <contract_id> "secret123" 1.0 500 wallet/admin.toml

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
just htlc-create-admin-to-maker 0.5 "secret" 500                    # Create HTLC
just htlc-claim-maker <contract_id> "secret" 0.5 500 wallet/admin.toml  # Claim HTLC
```

Built with Rust + BDK + Bitcoin Core regtest for development use.