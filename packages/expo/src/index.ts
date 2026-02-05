import { requireNativeModule } from 'expo-modules-core';

/**
 * Result of a successful proof generation
 */
export interface ProofResult {
  /** Handle for referencing the proof in memory */
  proofHandle: string;
  /** Hex-encoded proof bytes */
  proofBytes: string;
  /** Hex-encoded public inputs */
  publicInputs: string[];
  /** Time taken to generate proof in milliseconds */
  generationTimeMs: number;
  /** Size of the proof in bytes */
  proofSizeBytes: number;
}

/**
 * Shareable proof format for transmission/verification
 */
export interface ShareableProof {
  /** Hex-encoded proof bytes */
  proof: string;
  /** Hex-encoded public inputs */
  publicInputs: string[];
  /** Hex-encoded verifier index */
  verifierIndex: string;
}

interface KimchiNativeModule {
  initialize(srsLog2Size?: number): Promise<void>;
  isInitialized(): boolean;
  proveThreshold(value: string, threshold: string): Promise<ProofResult>;
  verifyProof(proofHandle: string): Promise<boolean>;
  exportVerifierIndex(proofHandle: string): Promise<string>;
  freeProof(proofHandle: string): Promise<void>;
  getSrsLog2Size(): number | null;
  getVersion(): string;
}

const KimchiNative = requireNativeModule<KimchiNativeModule>('Kimchi');

/**
 * Initialize the Kimchi prover with optional SRS size configuration.
 * Must be called before generating proofs.
 *
 * @param srsLog2Size - Log2 of SRS size (default: 14 = 16384 rows)
 *                      Smaller values (10-12) are faster for testing
 */
export async function initialize(srsLog2Size?: number): Promise<void> {
  return KimchiNative.initialize(srsLog2Size);
}

/**
 * Check if the prover has been initialized
 */
export function isInitialized(): boolean {
  return KimchiNative.isInitialized();
}

/**
 * Generate a zero-knowledge proof that a value is less than a threshold.
 *
 * @param value - The secret value to prove
 * @param threshold - The public threshold
 * @returns Proof result containing handle and encoded proof data
 *
 * @example
 * ```typescript
 * const proof = await proveThreshold(42n, 100n);
 * // Proves: 42 < 100 without revealing 42
 * ```
 */
export async function proveThreshold(value: bigint, threshold: bigint): Promise<ProofResult> {
  return KimchiNative.proveThreshold(value.toString(), threshold.toString());
}

/**
 * Verify a proof using its handle.
 *
 * @param proofHandle - Handle returned from proveThreshold
 * @returns True if the proof is valid
 */
export async function verifyProof(proofHandle: string): Promise<boolean> {
  return KimchiNative.verifyProof(proofHandle);
}

/**
 * Export the verifier index for a proof.
 * This can be used for remote verification with the WASM verifier.
 *
 * @param proofHandle - Handle returned from proveThreshold
 * @returns Hex-encoded verifier index
 */
export async function exportVerifierIndex(proofHandle: string): Promise<string> {
  return KimchiNative.exportVerifierIndex(proofHandle);
}

/**
 * Free a proof from memory.
 * Call this when done with a proof to release resources.
 *
 * @param proofHandle - Handle returned from proveThreshold
 */
export async function freeProof(proofHandle: string): Promise<void> {
  return KimchiNative.freeProof(proofHandle);
}

/**
 * Get the current SRS log2 size, or null if not initialized
 */
export function getSrsLog2Size(): number | null {
  return KimchiNative.getSrsLog2Size();
}

/**
 * Get the library version string
 */
export function getVersion(): string {
  return KimchiNative.getVersion();
}

/**
 * Create a shareable proof object for transmission.
 * Includes everything needed for remote verification.
 *
 * @param proofResult - Result from proveThreshold
 * @returns ShareableProof suitable for JSON serialization
 */
export async function createShareableProof(proofResult: ProofResult): Promise<ShareableProof> {
  const verifierIndex = await exportVerifierIndex(proofResult.proofHandle);
  return {
    proof: proofResult.proofBytes,
    publicInputs: proofResult.publicInputs,
    verifierIndex,
  };
}
