#!/usr/bin/env bash
# Setup script for Prop AMM Challenge development environment

set -e

echo "=== Prop AMM Challenge Setup ==="

# 1. Add Solana CLI to PATH
SOLANA_BIN="$HOME/.local/share/solana/install/active_release/bin"
if [ -d "$SOLANA_BIN" ]; then
    export PATH="$SOLANA_BIN:$PATH"
    echo "[OK] Solana CLI: $(solana --version)"
else
    echo "[INSTALLING] Solana CLI..."
    sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
    export PATH="$SOLANA_BIN:$PATH"
    echo "[OK] Solana CLI: $(solana --version)"
fi

# 2. Install prop-amm CLI
echo "[INSTALLING] prop-amm CLI..."
cargo install --path crates/cli --quiet
echo "[OK] prop-amm CLI installed"

echo ""
echo "=== Setup complete ==="
echo ""
echo "To add Solana to your current shell session:"
echo "  export PATH=\"$SOLANA_BIN:\$PATH\""
echo ""
echo "Quick start:"
echo "  prop-amm validate my_amm.rs    # Check constraints"
echo "  prop-amm run my_amm.rs         # Run 1000 simulations"
