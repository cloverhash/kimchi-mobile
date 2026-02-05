# @kimchi/expo

Expo module for zero-knowledge proof generation using the Kimchi proof system.

## Installation

This package is not yet published to npm. Install directly from the repository:

```bash
# From local path
npm install path/to/kimchi-mobile/packages/expo

# Or from git
npm install github:user/kimchi-mobile#main --subdir=packages/expo
```

**Note:** This module requires native code and cannot run in Expo Go. You must use `expo prebuild` or a development build.

```bash
npx expo prebuild
npx expo run:ios  # or run:android
```

## Usage

```typescript
import {
  initialize,
  proveThreshold,
  verifyProof,
  freeProof,
  createShareableProof
} from '@kimchi/expo';

// Initialize (once at app startup)
await initialize(14); // SRS log2 size, use 10-12 for testing

// Generate proof that value < threshold
const proof = await proveThreshold(42n, 100n);
console.log(`Proof generated in ${proof.generationTimeMs}ms`);
console.log(`Proof size: ${proof.proofSizeBytes} bytes`);

// Verify locally
const isValid = await verifyProof(proof.proofHandle);
console.log(`Valid: ${isValid}`);

// Create shareable proof for remote verification
const shareable = await createShareableProof(proof);
console.log('Send this to verifier:', JSON.stringify(shareable));

// Free memory when done
await freeProof(proof.proofHandle);
```

## API Reference

### `initialize(srsLog2Size?: number): Promise<void>`
Initialize the prover. Call once at app startup.
- `srsLog2Size`: Log2 of SRS size (default: 14). Use 10-12 for testing.

### `proveThreshold(value: bigint, threshold: bigint): Promise<ProofResult>`
Generate a ZK proof that `value < threshold`.

### `verifyProof(proofHandle: string): Promise<boolean>`
Verify a proof locally.

### `exportVerifierIndex(proofHandle: string): Promise<string>`
Export the verifier index for remote verification.

### `freeProof(proofHandle: string): Promise<void>`
Release proof memory. Call when done with a proof.

### `createShareableProof(proofResult: ProofResult): Promise<ShareableProof>`
Create a shareable proof with verifier index included.

### `isInitialized(): boolean`
Check if the prover has been initialized.

### `getVersion(): string`
Get the library version string.

## Types

```typescript
interface ProofResult {
  proofHandle: string;
  proofBytes: string;
  publicInputs: string[];
  generationTimeMs: number;
  proofSizeBytes: number;
}

interface ShareableProof {
  proof: string;
  publicInputs: string[];
  verifierIndex: string;
}
```

## Remote Verification

Shareable proofs can be verified in a browser or Node.js using the WASM verifier:

```typescript
import { initVerifier, verifyProof } from '@kimchi/wasm';

await initVerifier(12);
const isValid = verifyProof(
  shareable.proof,
  shareable.verifierIndex,
  shareable.publicInputs
);
```

## Building from Source

```bash
cd kimchi-mobile
./scripts/build-expo.sh
```

## Requirements

- Expo SDK 49+
- iOS 15.0+ or Android API 26+
- Development build (not Expo Go)
