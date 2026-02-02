#!/bin/bash
# Build the kimchi-wasm package for web and Node.js deployment.
#
# Prerequisites:
#   - wasm-pack: cargo install wasm-pack
#   - wasm32-unknown-unknown target: rustup target add wasm32-unknown-unknown
#   - nightly toolchain: rustup install nightly
#
# Note: Nightly is required because upstream proof-systems uses unstable features
# for WASM targets (unsigned_is_multiple_of).
#
# Output:
#   - kimchi-wasm/pkg/      - Browser (ES module) target
#   - kimchi-wasm/pkg-node/ - Node.js target

set -e

cd "$(dirname "$0")/.."

echo "Building kimchi-wasm..."

# Check for nightly toolchain
if ! rustup run nightly rustc --version &> /dev/null; then
    echo "Installing nightly toolchain..."
    rustup install nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
fi

# Ensure wasm32 target is available on nightly
rustup target add wasm32-unknown-unknown --toolchain nightly 2>/dev/null || true

# Install wasm-pack if needed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

cd kimchi-wasm

# Build for web (browser) target
echo ""
echo "Building for web target..."
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target web --out-dir pkg

# Build for Node.js target
echo ""
echo "Building for Node.js target..."
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target nodejs --out-dir pkg-node

# Optimize the WASM binaries if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo ""
    echo "Optimizing WASM binaries..."
    wasm-opt -Oz pkg/kimchi_wasm_bg.wasm -o pkg/kimchi_wasm_bg.wasm
    wasm-opt -Oz pkg-node/kimchi_wasm_bg.wasm -o pkg-node/kimchi_wasm_bg.wasm
fi

echo ""
echo "=========================================="
echo "WASM packages built successfully!"
echo "=========================================="
echo ""
echo "Browser target: kimchi-wasm/pkg/"
ls -la pkg/
echo ""
echo "Node.js target: kimchi-wasm/pkg-node/"
ls -la pkg-node/
echo ""
echo "Usage (Browser):"
echo "  import init, { init_verifier, verify_kimchi_proof } from './pkg/kimchi_wasm.js';"
echo "  await init();"
echo "  await init_verifier(14);"
echo "  const isValid = verify_kimchi_proof(proofHex, verifierIndexHex, publicInputsHex);"
echo ""
echo "Usage (Node.js):"
echo "  const { init_verifier, verify_kimchi_proof } = require('./pkg-node/kimchi_wasm.js');"
echo "  init_verifier(14);"
echo "  const isValid = verify_kimchi_proof(proofHex, verifierIndexHex, publicInputsHex);"
