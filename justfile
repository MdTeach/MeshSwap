# MeshSwap Demo Script Commands

# Show available commands
help:
    @echo "🚀 MeshSwap Demo Script Commands:"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "  start    - Start both Bitcoin and Ethereum chains"
    @echo "  stop     - Stop both chains"
    @echo "  monitor  - Monitor blockchain status"
    @echo "  help     - Show this help message"
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