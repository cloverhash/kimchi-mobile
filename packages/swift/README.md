# Kimchi Mobile - Swift

Native iOS/macOS library for zero-knowledge proof generation using the Kimchi proof system.

## Installation

This package is not yet published to a package registry. Install directly from the repository:

### Swift Package Manager

```swift
dependencies: [
    // From local path
    .package(path: "path/to/kimchi-mobile/packages/swift")
    // Or from git (once pushed)
    // .package(url: "https://github.com/user/kimchi-mobile", branch: "main")
]
```

Build the XCFramework first:
```bash
cd kimchi-mobile
./scripts/build-ios.sh
```

### Xcode

1. File > Add Package Dependencies
2. Enter the local path or repository URL
3. Select "KimchiMobile" package

## Usage

```swift
import KimchiMobile

// Initialize (once at app startup)
try await KimchiMobile.initialize(srsLog2Size: 14)

// Generate proof that value < threshold
let proof = try await KimchiMobile.proveThreshold(value: 42, threshold: 100)
print("Proof generated in \(proof.generationTimeMs)ms")
print("Proof size: \(proof.proofSizeBytes) bytes")

// Verify locally
let isValid = try KimchiMobile.verifyProof(proofHandle: proof.proofHandle)
print("Valid: \(isValid)")

// Export for remote verification
let shareable = try proof.toShareable()
let json = try shareable.toJson()
print("Shareable JSON: \(json)")

// Free memory
try KimchiMobile.freeProof(proofHandle: proof.proofHandle)
```

## API Reference

### `KimchiMobile.initialize(srsLog2Size: UInt32?) async throws`
Initialize the prover. Call once at app startup.
- `srsLog2Size`: Log2 of SRS size (default: 14). Use 10-12 for testing.

### `KimchiMobile.proveThreshold(value: UInt64, threshold: UInt64) async throws -> ProofResult`
Generate a ZK proof that `value < threshold`.

### `KimchiMobile.verifyProof(proofHandle: Int64) throws -> Bool`
Verify a proof locally.

### `KimchiMobile.exportVerifierIndex(proofHandle: Int64) throws -> String`
Export the verifier index for remote verification.

### `KimchiMobile.freeProof(proofHandle: Int64) throws`
Release proof memory. Call when done with a proof.

### `ProofResult.toShareable() throws -> ShareableProof`
Create a shareable proof with verifier index included.

## Building from Source

```bash
cd kimchi-mobile
./scripts/build-ios.sh
```

## Requirements

- iOS 15.0+ or macOS 12.0+
- ARM64 device (or simulator on Apple Silicon)
