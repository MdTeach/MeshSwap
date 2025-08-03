#!/bin/bash

echo "🔮 Generating Zero-Knowledge Proof..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Go to prover directory
cd prover

echo "⚡ Starting proof generation..."
just generate-mock-proof
echo ""
echo "✅ Zero-knowledge proof generation completed! 🎉"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"