# Bitcoin CLI with HTLC Support

Bitcoin development CLI with wallet management and Hash Time Locked Contracts supporting both secret revelation and timeout refund paths.

## Quick Start

```bash
just start                    # Start Bitcoin regtest
just balance-admin           # Check balance (starts with 10 BTC)
just send-admin-to-maker 2.5 # Send Bitcoin

# HTLC atomic swap - Two possible outcomes:
# Path 1: Recipient claims with secret
just htlc-create-admin-to-maker 1.0 "secret123" 500
just htlc-claim-maker <contract_id> "secret123" 1.0 500 wallet/admin.toml

# Path 2: Sender reclaims after timeout (if unclaimed)
just htlc-refund-admin <contract_id> "secret123" 1.0 500 wallet/maker.toml

just stop                    # Stop
```

## HTLC Overview

Hash Time Locked Contracts (HTLCs) enable trustless atomic swaps with two spending paths:

1. **Secret Path**: Recipient can claim funds by revealing the secret preimage
2. **Timeout Path**: Sender can reclaim funds after a specified block height if unclaimed

This enables atomic swaps without requiring trust between parties.

## Commands

```bash
# Network Management
just start                    # Start regtest
just stop                     # Stop and cleanup

# Wallet Operations
just balance-admin           # Admin balance
just balance-maker           # Maker balance
just balance-taker           # Taker balance

# Basic Transactions
just send-admin-to-maker 5.0 # Send BTC
just send-admin-to-taker 3.0
just send-maker-to-taker 1.0

# HTLC Creation
just htlc-create-admin-to-maker 0.5 "secret" 500    # Admin → Maker
just htlc-create-admin-to-taker 1.0 "password" 600  # Admin → Taker
just htlc-create-maker-to-taker 0.25 "key" 450      # Maker → Taker

# HTLC Claims (with secret)
just htlc-claim-maker <contract_id> "secret" 0.5 500 wallet/admin.toml
just htlc-claim-taker <contract_id> "password" 1.0 600 wallet/admin.toml
just htlc-claim-admin <contract_id> "key" 0.25 450 wallet/maker.toml

# HTLC Refunds (after timeout, requires original secret)
just htlc-refund-admin <contract_id> "secret" 0.5 500 wallet/maker.toml
just htlc-refund-admin <contract_id> "password" 1.0 600 wallet/taker.toml
just htlc-refund-maker <contract_id> "key" 0.25 450 wallet/taker.toml

# Testing Examples
just htlc-test-create                                   # Create test HTLC
just htlc-test-claim <contract_id>                      # Claim with secret
just htlc-test-refund <contract_id>                     # Refund after timeout
```

## HTLC Workflow

### Successful Claim (Happy Path)
1. Alice creates HTLC: `just htlc-create-admin-to-maker 1.0 "mysecret" 500`
2. Bob claims with secret: `just htlc-claim-maker <contract_id> "mysecret" 1.0 500 wallet/admin.toml`
3. ✅ Bob receives 1.0 BTC, Alice's secret is revealed

### Timeout Refund (Fallback Path)
1. Alice creates HTLC: `just htlc-create-admin-to-maker 1.0 "mysecret" 500`
2. Bob doesn't claim (doesn't know secret or chooses not to)
3. After block 500: Alice refunds: `just htlc-refund-admin <contract_id> "mysecret" 1.0 500 wallet/maker.toml`
4. ✅ Alice gets her 1.0 BTC back

Built with Rust + BDK + Bitcoin Core regtest for development and testing.