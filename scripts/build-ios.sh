#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/ios-output"

echo "Building Kimchi Mobile for iOS..."

# Check for required tools
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Install Rust first."
    exit 1
fi

if ! command -v xcodebuild &> /dev/null; then
    echo "Error: xcodebuild not found. Install Xcode first."
    exit 1
fi

# Install iOS targets if not present
echo "Checking iOS targets..."
rustup target add aarch64-apple-ios 2>/dev/null || true
rustup target add aarch64-apple-ios-sim 2>/dev/null || true

# Build for iOS device (arm64)
echo "Building for iOS device (aarch64-apple-ios)..."
cd "$PROJECT_ROOT"
cargo build --release --package kimchi-ffi --target aarch64-apple-ios

# Build for iOS simulator (arm64)
echo "Building for iOS simulator (aarch64-apple-ios-sim)..."
cargo build --release --package kimchi-ffi --target aarch64-apple-ios-sim

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Copy libraries
echo "Copying libraries..."
cp "$PROJECT_ROOT/target/aarch64-apple-ios/release/libkimchi_ffi.a" "$OUTPUT_DIR/libkimchi_ffi_ios.a"
cp "$PROJECT_ROOT/target/aarch64-apple-ios-sim/release/libkimchi_ffi.a" "$OUTPUT_DIR/libkimchi_ffi_sim.a"

# Generate Swift bindings using UniFFI
echo "Generating Swift bindings..."
cargo run --package kimchi-ffi --bin uniffi-bindgen generate \
    --library "$PROJECT_ROOT/target/aarch64-apple-ios/release/libkimchi_ffi.a" \
    --language swift \
    --out-dir "$OUTPUT_DIR/swift"

# Create XCFramework
echo "Creating XCFramework..."
rm -rf "$OUTPUT_DIR/KimchiFfi.xcframework"
xcodebuild -create-xcframework \
    -library "$OUTPUT_DIR/libkimchi_ffi_ios.a" \
    -headers "$OUTPUT_DIR/swift" \
    -library "$OUTPUT_DIR/libkimchi_ffi_sim.a" \
    -headers "$OUTPUT_DIR/swift" \
    -output "$OUTPUT_DIR/KimchiFfi.xcframework"

echo ""
echo "Build complete!"
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "To use in your iOS project:"
echo "1. Add KimchiFfi.xcframework to your Xcode project"
echo "2. Add the Swift files from $OUTPUT_DIR/swift"
echo "3. Add KimchiMobile Swift package from $PROJECT_ROOT/swift"
