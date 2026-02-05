#!/bin/bash
# Build Kimchi Expo package (builds native dependencies first)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
EXPO_DIR="$PROJECT_DIR/packages/expo"

echo "Building Kimchi Expo package..."
echo ""

# Build native dependencies first
echo "=== Building Android native libraries ==="
"$SCRIPT_DIR/build-android.sh"

echo ""
echo "=== Building iOS native libraries ==="
"$SCRIPT_DIR/build-ios.sh"

echo ""
echo "=== Building Expo TypeScript module ==="
cd "$EXPO_DIR"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

# Build TypeScript
echo "Building TypeScript..."
npm run build

echo ""
echo "Build complete!"
echo ""
echo "The Expo package is ready at: $EXPO_DIR"
echo ""
echo "To use in your Expo app:"
echo "  1. npm install file:../path/to/kimchi-mobile/packages/expo"
echo "  2. Or publish to npm and install: npm install @kimchi/expo"
echo ""
echo "Note: This module requires 'expo prebuild' - it cannot run in Expo Go"
echo "due to the native binary dependencies."
