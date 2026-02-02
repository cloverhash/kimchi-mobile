#!/bin/bash
# Setup script for Kimchi Mobile development environment

set -e

echo "Setting up Kimchi Mobile development environment..."

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Rust version: $(rustc --version)"

# Install required Rust targets
echo "Installing Android targets..."
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# Install iOS targets (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Installing iOS targets..."
    rustup target add aarch64-apple-ios
    rustup target add aarch64-apple-ios-sim
    rustup target add x86_64-apple-ios
fi

# Install cargo-ndk for Android builds
echo "Installing cargo-ndk..."
cargo install cargo-ndk

# Install uniffi-bindgen
echo "Installing uniffi-bindgen..."
cargo install uniffi_bindgen

# Check for Android NDK
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo ""
    echo "WARNING: ANDROID_NDK_HOME not set"
    echo "Please install Android NDK via Android Studio SDK Manager"
    echo "Then set ANDROID_NDK_HOME in your shell profile:"
    echo ""
    echo "  export ANDROID_NDK_HOME=\$HOME/Library/Android/sdk/ndk/<version>"
    echo ""
fi

# Setup .cargo/config.toml if it doesn't exist
if [ ! -f "$SCRIPT_DIR/../.cargo/config.toml" ]; then
    if [ -f "$SCRIPT_DIR/../.cargo/config.toml.example" ]; then
        echo "Setting up .cargo/config.toml..."
        if [ -n "$ANDROID_NDK_HOME" ]; then
            # Detect host tag
            if [[ "$OSTYPE" == "darwin"* ]]; then
                HOST_TAG="darwin-x86_64"
            elif [[ "$OSTYPE" == "linux"* ]]; then
                HOST_TAG="linux-x86_64"
            else
                HOST_TAG="windows-x86_64"
            fi

            # Create config.toml from example with actual paths
            sed -e "s|\$NDK_HOME|$ANDROID_NDK_HOME|g" \
                -e "s|\$HOST_TAG|$HOST_TAG|g" \
                "$SCRIPT_DIR/../.cargo/config.toml.example" > "$SCRIPT_DIR/../.cargo/config.toml"
            echo "Created .cargo/config.toml with NDK path: $ANDROID_NDK_HOME"
        else
            echo "Skipping .cargo/config.toml setup - ANDROID_NDK_HOME not set"
        fi
    fi
fi

# Verify the setup
echo ""
echo "Verifying setup..."
echo "  Rust: $(rustc --version)"
echo "  Cargo: $(cargo --version)"
echo "  cargo-ndk: $(cargo ndk --version 2>/dev/null || echo 'not found')"

echo ""
echo "Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Ensure Android NDK is installed (via Android Studio SDK Manager)"
echo "  2. Set ANDROID_NDK_HOME environment variable"
echo "  3. Run: ./scripts/build-android.sh"
