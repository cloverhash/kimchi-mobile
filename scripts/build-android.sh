#!/bin/bash
# Build Kimchi mobile library for Android

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_DIR/packages/kotlin/src/main/jniLibs"

# Android NDK must be installed
if [ -z "$ANDROID_NDK_HOME" ]; then
    # Try common locations
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        ANDROID_NDK_HOME=$(ls -d "$HOME/Library/Android/sdk/ndk"/* 2>/dev/null | head -1)
    elif [ -d "$HOME/Android/Sdk/ndk" ]; then
        ANDROID_NDK_HOME=$(ls -d "$HOME/Android/Sdk/ndk"/* 2>/dev/null | head -1)
    fi

    if [ -z "$ANDROID_NDK_HOME" ]; then
        echo "Error: ANDROID_NDK_HOME not set and NDK not found in common locations"
        echo "Please install Android NDK and set ANDROID_NDK_HOME"
        exit 1
    fi
fi

echo "Using Android NDK: $ANDROID_NDK_HOME"

# Install Rust Android targets if needed
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Install cargo-ndk if needed
if ! command -v cargo-ndk &> /dev/null; then
    echo "Installing cargo-ndk..."
    cargo install cargo-ndk
fi

# Create output directories
mkdir -p "$OUTPUT_DIR/arm64-v8a"
mkdir -p "$OUTPUT_DIR/armeabi-v7a"
mkdir -p "$OUTPUT_DIR/x86_64"

cd "$PROJECT_DIR"

echo "Building for arm64-v8a..."
cargo ndk -t arm64-v8a -o "$OUTPUT_DIR" build --release -p kimchi-ffi

echo "Building for armeabi-v7a..."
cargo ndk -t armeabi-v7a -o "$OUTPUT_DIR" build --release -p kimchi-ffi

echo "Building for x86_64..."
cargo ndk -t x86_64 -o "$OUTPUT_DIR" build --release -p kimchi-ffi

# Generate Kotlin bindings
echo "Generating Kotlin bindings..."
cargo run -p kimchi-ffi --features uniffi/cli -- \
    generate \
    --library target/release/libkimchi_ffi.dylib \
    --language kotlin \
    --out-dir "$PROJECT_DIR/packages/kotlin/src/main/kotlin"

echo ""
echo "Build complete! Libraries are in:"
echo "  $OUTPUT_DIR/arm64-v8a/libkimchi_ffi.so"
echo "  $OUTPUT_DIR/armeabi-v7a/libkimchi_ffi.so"
echo "  $OUTPUT_DIR/x86_64/libkimchi_ffi.so"
echo ""
echo "Kotlin bindings are in:"
echo "  $PROJECT_DIR/packages/kotlin/src/main/kotlin/uniffi/kimchi_ffi/"
