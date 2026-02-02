# Kimchi Mobile

A mobile-friendly zero-knowledge proof toolkit using Kimchi (Mina's proof system) for Android and iOS.

**Generate Mina-compatible ZK proofs directly on mobile devices.**

## Overview

Kimchi Mobile provides a simple API for generating zero-knowledge proofs on mobile devices using Kimchi, the same proof system that powers the Mina Protocol. This enables:

- **Privacy-preserving mobile apps** - Prove facts without revealing private data
- **Mina blockchain integration** - Generate proofs compatible with Mina zkApps
- **Offline capability** - No server required for proof generation
- **Native performance** - Rust core with native bindings for Android/iOS

## Quick Start

### Android (Kotlin)

```kotlin
// Initialize once at app startup
KimchiMobile.initialize(context)

// Generate a threshold proof (prove value < threshold without revealing value)
val result = KimchiMobile.proveThreshold(value = 50, threshold = 100)
if (result.isSuccess) {
    val proof = result.getOrNull()!!
    println("Proof generated in ${proof.generationTimeMs}ms")

    // Verify locally
    val valid = KimchiMobile.verifyProof(proof.proofHandle)

    // Or share for remote verification
    val shareable = proof.toShareable(mapOf("app" to "MyApp"))
    sendToVerifier(shareable.toJson())
}
```

### iOS (Swift)

```swift
// Initialize once at app startup
try KimchiMobile.initialize()

// Generate a threshold proof
let proof = try await KimchiMobile.proveThreshold(value: 50, threshold: 100)
print("Proof generated in \(proof.generationTimeMs)ms")

// Verify locally
let valid = try KimchiMobile.verifyProof(proofHandle: proof.proofHandle)

// Or share for remote verification
let shareable = proof.toShareable(metadata: ["app": "MyApp"])
sendToVerifier(shareable.toJson())
```

### Rust (Direct Usage)

```rust
use kimchi_prover::{KimchiProver, ProverConfig, ThresholdCircuit};

// Create prover with configuration
let mut prover = KimchiProver::with_config(ProverConfig {
    srs_log2_size: 14,  // 2^14 = 16384 rows
    debug: false,
});

// Initialize SRS (one-time setup)
prover.init_srs()?;

// Create a threshold circuit
let circuit = ThresholdCircuit::new(100); // threshold = 100

// Setup the circuit
let (prover_index, verifier_index) = prover.setup(
    circuit.gates(),
    circuit.num_public_inputs()
)?;

// Generate witness for a private value
let (witness, public_inputs) = circuit.generate_witness(50)?; // value = 50

// Generate proof
let proof = prover.prove(&prover_index, witness)?;

// Verify
let valid = prover.verify(&verifier_index, &proof, &public_inputs)?;
assert!(valid);
```

## Architecture

```
kimchi-mobile/
├── kimchi-prover/          # Core Rust prover library
│   └── src/
│       ├── prover.rs       # Proof generation/verification
│       ├── types.rs        # Field elements
│       ├── circuits/       # Circuit implementations
│       │   └── threshold.rs # Threshold comparison circuit
│       └── gadgets/        # Constraint gadgets (SHA256, RSA)
│
├── kimchi-ffi/             # FFI bindings (UniFFI)
│   └── src/lib.rs          # FFI exports for Android/iOS
│
├── kimchi-wasm/            # WebAssembly verifier
│   └── src/lib.rs          # WASM exports for browser/Node.js
│
├── kotlin/                 # Android Kotlin wrapper
├── swift/                  # iOS Swift wrapper
│
└── scripts/
    ├── setup.sh            # Development setup
    ├── build-android.sh    # Android build script
    ├── build-ios.sh        # iOS build script
    └── build-wasm.sh       # WebAssembly build script
```

## Building

### Prerequisites

1. **Rust** (1.70+)
2. **Android NDK** (for Android builds)
3. **cargo-ndk** (for cross-compilation)

### Setup

```bash
# Run setup script (installs targets, cargo-ndk, configures NDK paths)
./scripts/setup.sh

# Or manually:
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk

# Copy and configure .cargo/config.toml for your NDK path
cp .cargo/config.toml.example .cargo/config.toml
# Edit .cargo/config.toml with your NDK path
```

### Build for Android

```bash
# Set NDK path
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/<version>

# Build native libraries and generate bindings
./scripts/build-android.sh
```

This produces:
- `kotlin/src/main/jniLibs/arm64-v8a/libkimchi_ffi.so`
- `kotlin/src/main/jniLibs/armeabi-v7a/libkimchi_ffi.so`
- `kotlin/src/main/jniLibs/x86_64/libkimchi_ffi.so`
- Kotlin bindings in `kotlin/src/main/kotlin/uniffi/kimchi_ffi/`

### Build for iOS

```bash
# Requires macOS with Xcode installed
./scripts/build-ios.sh
```

This produces:
- `ios-output/KimchiFfi.xcframework` - Universal framework for iOS
- Swift bindings in `ios-output/swift/`

### Build for WebAssembly

```bash
# Requires wasm-pack and nightly Rust
./scripts/build-wasm.sh
```

This produces:
- `kimchi-wasm/pkg/` - Browser WASM package
- `kimchi-wasm/pkg-node/` - Node.js WASM package

## Built-in Circuits

### ThresholdCircuit

Proves that a private value is less than a public threshold.

**Use cases:**
- "Prove I'm over 18" (age > 18)
- "Prove my balance is under $10,000" (for compliance)
- "Prove my score is below the limit"

```kotlin
// Android: Prove my secret value (50) is less than 100
val result = KimchiMobile.proveThreshold(value = 50, threshold = 100)
```

```swift
// iOS: Prove my secret value (50) is less than 100
let proof = try await KimchiMobile.proveThreshold(value: 50, threshold: 100)
```

## API Reference

### Kotlin API

| Function | Description |
|----------|-------------|
| `KimchiMobile.initialize(context)` | Initialize the prover (call once) |
| `KimchiMobile.isInitialized()` | Check if prover is initialized |
| `KimchiMobile.proveThreshold(value, threshold)` | Generate a threshold proof |
| `KimchiMobile.verifyProof(proofHandle)` | Verify a proof by handle |
| `KimchiMobile.exportVerifierIndex(proofHandle)` | Export verifier index for WASM |
| `KimchiMobile.freeProof(proofHandle)` | Free proof from memory |
| `KimchiMobile.getLibraryVersion()` | Get version string |

### Swift API

| Function | Description |
|----------|-------------|
| `KimchiMobile.initialize()` | Initialize the prover (call once) |
| `KimchiMobile.isInitialized()` | Check if prover is initialized |
| `KimchiMobile.proveThreshold(value:threshold:)` | Generate a threshold proof |
| `KimchiMobile.verifyProof(proofHandle:)` | Verify a proof by handle |
| `KimchiMobile.exportVerifierIndex(proofHandle:)` | Export verifier index for WASM |
| `KimchiMobile.freeProof(proofHandle:)` | Free proof from memory |
| `KimchiMobile.version()` | Get version string |

### Rust API

| Type | Description |
|------|-------------|
| `KimchiProver` | Main prover for generating/verifying proofs |
| `ProverConfig` | Configuration for SRS size and debug mode |
| `ThresholdCircuit` | Circuit for threshold comparison proofs |
| `FieldElement` | Field element for inputs/outputs |

### WASM Verifier API

| Function | Description |
|----------|-------------|
| `init_verifier(srs_log2_size)` | Initialize verifier with SRS size |
| `is_verifier_initialized()` | Check if verifier is ready |
| `verify_kimchi_proof(proof_hex, verifier_index_hex, public_inputs_hex)` | Verify a proof |
| `verify_kimchi_proof_detailed(...)` | Verify with detailed error info |

## Sharing Proofs

Both Kotlin and Swift wrappers include helper types for sharing proofs:

```kotlin
// Kotlin
val shareable = proofResult.toShareable(mapOf("app" to "MyApp"))
val json = shareable.toJson()
// Send `json` to your verifier service
```

```swift
// Swift
let shareable = proofResult.toShareable(metadata: ["app": "MyApp"])
let json = shareable.toJson()
// Send `json` to your verifier service
```

The `ShareableProof` includes:
- `proof` - Hex-encoded proof bytes
- `verifierIndex` - Hex-encoded verifier index (for WASM verification)
- `publicInputs` - Array of hex-encoded field elements
- `metadata` - Additional context (proof size, generation time, etc.)

## Performance

Expected performance on mobile devices:

| Operation | Time (est.) | Notes |
|-----------|-------------|-------|
| SRS Init | 2-5s | One-time setup |
| Circuit Setup | 0.5-1s | Per circuit type |
| Proof Generation | 1-10s | Depends on circuit complexity |
| Verification | 0.1-0.5s | Fast |

**Note:** Performance varies by device. High-end devices (8GB+ RAM) recommended for complex circuits.

## Mina Compatibility

Proofs generated by Kimchi Mobile use:
- **Kimchi proof system** (PLONK variant)
- **Pasta curves** (Pallas/Vesta)
- **Same SRS format** as Mina

This means proofs can be verified by Mina nodes and used in zkApps.

## Limitations

- **Memory:** Proof generation requires significant RAM (2GB+ recommended)
- **CPU:** Intensive computation may cause thermal throttling on mobile
- **Battery:** Extended proof generation will drain battery
- **Circuit size:** Limited by device memory

## Included Gadgets

The `kimchi-prover` crate includes reusable gadgets for building custom circuits:

- **SHA256Gadget** - SHA-256 hash computation in-circuit
- **RsaGadget** - RSA signature verification in-circuit

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas of interest:
- Performance optimization for mobile
- Additional circuit templates
- Documentation and examples

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [o1-labs/proof-systems](https://github.com/o1-labs/proof-systems) - Kimchi implementation
- [zkmopro/mopro](https://github.com/zkmopro/mopro) - Mobile prover inspiration
- [Mina Protocol](https://minaprotocol.com/) - The lightest blockchain
