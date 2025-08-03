# MeshSwap Demo Script Commands

# Show available commands
help:
    @echo "🚀 MeshSwap Demo Script Commands:"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "  start        - Start both Bitcoin and Ethereum chains"
    @echo "  stop         - Stop both chains"
    @echo "  monitor      - Monitor blockchain status"
    @echo "  balance      - Check wallet and contract balances"
    @echo "  deploy-btc   - Deploy Bitcoin contracts and execute swap"
    @echo "  prove        - Generate zero-knowledge proof"
    @echo "  verify       - Verify zero-knowledge proof"
    @echo "  maker        - Create cross-chain order (maker side)"
    @echo "  taker        - Fill cross-chain order (taker side)"
    @echo "  withdraw SECRET - Withdraw maker funds using swap secret"
    @echo "  help         - Show this help message"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start both chains and deploy contracts
start:
    @./demo_script/spin_chain.sh

# Stop both chains
stop:
    @./demo_script/stop_chain.sh

# Monitor blockchain status
monitor:
    @./demo_script/network_monitor.sh

# Check wallet and contract balances
balance:
    @./demo_script/balance_checker.sh

# Deploy Bitcoin contracts and execute swap
deploy-btc:
    @./demo_script/deploy_bitcoin_contract.sh

# Generate zero-knowledge proof
prove:
    @./demo_script/generate_proof.sh

# Verify zero-knowledge proof
verify:
    @./demo_script/verify_proof.sh

# Create cross-chain order (maker side)
maker:
    @./demo_script/run_maker.sh

# Fill cross-chain order (taker side)
taker:
    @./demo_script/run_taker.sh

# Withdraw maker funds using swap secret
withdraw SECRET:
    @./demo_script/withdraw_maker.sh {{SECRET}}