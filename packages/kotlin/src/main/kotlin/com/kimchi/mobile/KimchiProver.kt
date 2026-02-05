/**
 * Kimchi Mobile Prover - Kotlin wrapper for Mina-compatible ZK proofs on Android.
 *
 * This library provides a simple API for generating zero-knowledge proofs
 * using Kimchi (Mina's proof system) directly on Android devices.
 */
package com.kimchi.mobile

import android.content.Context
import android.util.Log
import uniffi.kimchi_ffi.KimchiException

/**
 * Main entry point for Kimchi mobile prover functionality.
 */
object KimchiMobile {
    private const val TAG = "KimchiMobile"
    private var initialized = false

    /**
     * Initialize the Kimchi prover. Call this once at app startup.
     *
     * This sets up the cryptographic parameters needed for proof generation.
     * It may take a few seconds on first run.
     *
     * @param context Android context (used for native library loading)
     * @param srsLog2Size Log2 of SRS size (default 14 = 16384 rows). Use 10-12 for testing.
     * @return true if initialization succeeded
     */
    @JvmStatic
    @JvmOverloads
    fun initialize(context: Context, srsLog2Size: Int? = null): Boolean {
        if (initialized) {
            Log.d(TAG, "Already initialized")
            return true
        }

        return try {
            // Load native library
            System.loadLibrary("kimchi_ffi")

            // Initialize the prover
            uniffi.kimchi_ffi.initProver(srsLog2Size?.toUInt())

            initialized = true
            Log.i(TAG, "Kimchi prover initialized successfully")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize Kimchi prover", e)
            false
        }
    }

    /**
     * Check if the prover is initialized.
     */
    @JvmStatic
    fun isInitialized(): Boolean = initialized

    /**
     * Get the library version.
     */
    @JvmStatic
    fun getLibraryVersion(): String {
        return try {
            uniffi.kimchi_ffi.getVersion()
        } catch (e: Exception) {
            "unknown"
        }
    }

    /**
     * Get the SRS log2 size used by the prover.
     */
    @JvmStatic
    fun getSrsLog2Size(): Int? {
        return try {
            uniffi.kimchi_ffi.getSrsLog2Size().toInt()
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Generate a proof that a private value is less than a public threshold.
     *
     * This proves: "I know a secret value V such that V < threshold"
     * without revealing what V actually is.
     *
     * @param value The private value (will not be revealed in the proof)
     * @param threshold The public threshold to compare against
     * @return ProofResult containing the proof and metadata
     *
     * Example:
     * ```kotlin
     * // Prove that my secret number (50) is less than 100
     * val result = KimchiMobile.proveThreshold(value = 50, threshold = 100)
     * if (result.isSuccess) {
     *     val proof = result.getOrNull()!!
     *     println("Proof generated in ${proof.generationTimeMs}ms")
     *
     *     // Verify locally
     *     val valid = KimchiMobile.verifyProof(proof.proofHandle)
     *
     *     // Or share for remote verification
     *     val shareable = proof.toShareable()
     *     sendToVerifier(shareable.toJson())
     * }
     * ```
     */
    @JvmStatic
    fun proveThreshold(value: Long, threshold: Long): Result<ProofResult> {
        if (!initialized) {
            return Result.failure(IllegalStateException("Prover not initialized. Call initialize() first."))
        }

        return try {
            Log.d(TAG, "Generating threshold proof: value < $threshold")

            // Generate the proof
            val ffiResult = uniffi.kimchi_ffi.proveThreshold(
                value.toULong(),
                threshold.toULong()
            )

            // Export verifier index for WASM verification
            val verifierIndex = uniffi.kimchi_ffi.exportVerifierIndex(ffiResult.proofHandle)

            val result = ProofResult(
                proofHandle = ffiResult.proofHandle.toLong(),
                proofBytes = ffiResult.proofBytes,
                verifierIndex = verifierIndex,
                publicInputs = ffiResult.publicInputs,
                generationTimeMs = ffiResult.generationTimeMs.toLong(),
                proofSizeBytes = ffiResult.proofSizeBytes.toLong()
            )

            Log.i(TAG, "Proof generated in ${result.generationTimeMs}ms, size: ${result.proofSizeBytes} bytes")
            Result.success(result)
        } catch (e: KimchiException) {
            Log.e(TAG, "Proof generation failed", e)
            Result.failure(e)
        } catch (e: Exception) {
            Log.e(TAG, "Unexpected error during proof generation", e)
            Result.failure(e)
        }
    }

    /**
     * Free a proof from memory.
     *
     * @param proofHandle Handle to the proof to free
     */
    @JvmStatic
    fun freeProof(proofHandle: Long): Boolean {
        return try {
            uniffi.kimchi_ffi.freeProof(proofHandle.toULong())
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to free proof", e)
            false
        }
    }

    /**
     * Verify a proof by its handle.
     *
     * @param proofHandle Handle to the proof to verify
     * @return true if the proof is valid
     */
    @JvmStatic
    fun verifyProof(proofHandle: Long): Result<Boolean> {
        if (!initialized) {
            return Result.failure(IllegalStateException("Prover not initialized. Call initialize() first."))
        }

        return try {
            val valid = uniffi.kimchi_ffi.verifyProof(proofHandle.toULong())
            Result.success(valid)
        } catch (e: KimchiException) {
            Log.e(TAG, "Proof verification failed", e)
            Result.failure(e)
        } catch (e: Exception) {
            Log.e(TAG, "Unexpected error during verification", e)
            Result.failure(e)
        }
    }

    /**
     * Export the verifier index for a stored proof.
     *
     * @param proofHandle Handle to the proof
     * @return Hex-encoded verifier index for WASM verification
     */
    @JvmStatic
    fun exportVerifierIndex(proofHandle: Long): Result<String> {
        if (!initialized) {
            return Result.failure(IllegalStateException("Prover not initialized. Call initialize() first."))
        }

        return try {
            val verifierIndex = uniffi.kimchi_ffi.exportVerifierIndex(proofHandle.toULong())
            Result.success(verifierIndex)
        } catch (e: KimchiException) {
            Log.e(TAG, "Failed to export verifier index", e)
            Result.failure(e)
        } catch (e: Exception) {
            Log.e(TAG, "Unexpected error exporting verifier index", e)
            Result.failure(e)
        }
    }
}

/**
 * Result of proof generation.
 */
data class ProofResult(
    /** Handle to the proof stored in memory */
    val proofHandle: Long,
    /** Hex-encoded proof bytes */
    val proofBytes: String,
    /** Hex-encoded verifier index (for WASM verification) */
    val verifierIndex: String,
    /** Public inputs as hex-encoded field elements */
    val publicInputs: List<String>,
    /** Time taken to generate the proof in milliseconds */
    val generationTimeMs: Long,
    /** Size of the proof in bytes */
    val proofSizeBytes: Long
) {
    /**
     * Convert to a shareable format for transmission to a verifier.
     */
    fun toShareable(metadata: Map<String, String> = emptyMap()): ShareableProof {
        return ShareableProof(
            proof = proofBytes,
            verifierIndex = verifierIndex,
            publicInputs = publicInputs,
            metadata = metadata + mapOf(
                "proofSize" to proofSizeBytes.toString(),
                "generationTime" to generationTimeMs.toString()
            )
        )
    }
}

/**
 * A proof format suitable for sharing/transmission to verifiers.
 */
data class ShareableProof(
    /** Hex-encoded proof bytes */
    val proof: String,
    /** Hex-encoded verifier index (for WASM verification) */
    val verifierIndex: String,
    /** Public inputs as hex-encoded field elements */
    val publicInputs: List<String>,
    /** Additional metadata */
    val metadata: Map<String, String>
) {
    /**
     * Convert to JSON string for transmission.
     */
    fun toJson(): String {
        val publicInputsJson = publicInputs.joinToString(",") { "\"$it\"" }
        val metadataJson = metadata.entries.joinToString(",") { "\"${it.key}\": \"${it.value}\"" }
        return """
            {
                "proof": "$proof",
                "verifierIndex": "$verifierIndex",
                "publicInputs": [$publicInputsJson],
                "metadata": {$metadataJson}
            }
        """.trimIndent()
    }
}

// Helper extension for hex encoding
private fun ByteArray.toHexString(): String = joinToString("") { "%02x".format(it) }
