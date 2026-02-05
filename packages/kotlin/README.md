# Kimchi Mobile - Kotlin

Native Android library for zero-knowledge proof generation using the Kimchi proof system.

## Installation

This package is not yet published to Maven. Install directly from the repository:

```kotlin
// build.gradle.kts
dependencies {
    implementation(files("path/to/kimchi-mobile/packages/kotlin/build/outputs/aar/kimchi-mobile-release.aar"))
}
```

Build the AAR first:
```bash
cd kimchi-mobile
./scripts/build-android.sh
cd packages/kotlin && ./gradlew assembleRelease
```

## Usage

```kotlin
import com.kimchi.mobile.KimchiMobile

// Initialize (once at app startup)
val initialized = KimchiMobile.initialize(context, srsLog2Size = 14)

// Generate proof that value < threshold
KimchiMobile.proveThreshold(42L, 100L)
    .onSuccess { proof ->
        println("Proof generated in ${proof.generationTimeMs}ms")
        println("Proof size: ${proof.proofSizeBytes} bytes")

        // Verify locally
        KimchiMobile.verifyProof(proof.proofHandle)
            .onSuccess { valid -> println("Valid: $valid") }

        // Export for remote verification
        KimchiMobile.exportVerifierIndex(proof.proofHandle)
            .onSuccess { vi -> println("Verifier index: $vi") }

        // Free memory
        KimchiMobile.freeProof(proof.proofHandle)
    }
```

## API Reference

### `KimchiMobile.initialize(context: Context, srsLog2Size: Int? = null): Boolean`
Initialize the prover. Call once at app startup.
- `srsLog2Size`: Log2 of SRS size (default: 14). Use 10-12 for testing.

### `KimchiMobile.proveThreshold(value: Long, threshold: Long): Result<ProofResult>`
Generate a ZK proof that `value < threshold`.

### `KimchiMobile.verifyProof(proofHandle: Long): Result<Boolean>`
Verify a proof locally.

### `KimchiMobile.exportVerifierIndex(proofHandle: Long): Result<String>`
Export the verifier index for remote verification.

### `KimchiMobile.freeProof(proofHandle: Long): Boolean`
Release proof memory. Call when done with a proof.

## Building from Source

```bash
cd kimchi-mobile
./scripts/build-android.sh
```

## Requirements

- Android API 26+ (Android 8.0)
- ARM64, ARMv7, or x86_64 device/emulator
