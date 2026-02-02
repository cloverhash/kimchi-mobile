#!/bin/bash
# Run all tests for Kimchi Mobile

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "========================================"
echo "Kimchi Mobile Test Suite"
echo "========================================"
echo ""

# Check Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo ""

# Run formatting check
echo "----------------------------------------"
echo "Checking code formatting..."
echo "----------------------------------------"
if cargo fmt --all -- --check; then
    echo "✓ Formatting OK"
else
    echo "✗ Formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi
echo ""

# Run clippy
echo "----------------------------------------"
echo "Running Clippy lints..."
echo "----------------------------------------"
cargo clippy --all-targets --all-features 2>&1 | grep -E "^(warning|error)" | head -20 || true
echo "✓ Clippy check complete"
echo ""

# Run tests
echo "----------------------------------------"
echo "Running unit tests..."
echo "----------------------------------------"
cargo test --workspace --all-features

echo ""
echo "========================================"
echo "All tests passed!"
echo "========================================"
