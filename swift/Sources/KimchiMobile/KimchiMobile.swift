import Foundation
import KimchiFfi

// Import CommonCrypto for SHA256
import CommonCrypto

/// Main interface for Kimchi mobile proof generation and verification.
///
/// Usage:
/// ```swift
/// // Initialize once at app startup
/// try KimchiMobile.initialize()
///
/// // Generate a threshold proof
/// let proof = try await KimchiMobile.proveThreshold(value: 50, threshold: 100)
///
/// // Verify locally
/// let valid = try KimchiMobile.verifyProof(proofHandle: proof.proofHandle)
///
/// // Or share for remote verification
/// let shareable = proof.toShareable(metadata: ["app": "MyApp"])
/// let json = shareable.toJson()
/// ```
public final class KimchiMobile {

    private static var initialized = false
    private static let initLock = NSLock()

    /// Initialize the Kimchi prover. Must be called once before generating proofs.
    ///
    /// - Parameter srsLog2Size: Log2 of the SRS size (default: 14 = 16384 rows).
    ///   Larger values support bigger circuits but use more memory.
    public static func initialize(srsLog2Size: UInt32? = nil) throws {
        initLock.lock()
        defer { initLock.unlock() }

        guard !initialized else { return }

        try initProver(srsLog2Size: srsLog2Size)
        initialized = true
    }

    /// Check if the prover has been initialized.
    public static func isInitialized() -> Bool {
        return initialized
    }

    /// Generate a proof that a private value is less than a public threshold.
    ///
    /// This proves: "I know a secret value V such that V < threshold"
    /// without revealing what V actually is.
    ///
    /// - Parameters:
    ///   - value: The private value (will not be revealed in the proof)
    ///   - threshold: The public threshold to compare against
    /// - Returns: ProofResult containing the proof and metadata
    ///
    /// Example:
    /// ```swift
    /// // Prove that my secret number (50) is less than 100
    /// let proof = try await KimchiMobile.proveThreshold(value: 50, threshold: 100)
    /// print("Proof generated in \(proof.generationTimeMs)ms")
    ///
    /// // Verify locally
    /// let valid = try KimchiMobile.verifyProof(proofHandle: proof.proofHandle)
    ///
    /// // Or share for remote verification
    /// let shareable = proof.toShareable()
    /// sendToVerifier(shareable.toJson())
    /// ```
    public static func proveThreshold(value: UInt64, threshold: UInt64) async throws -> ProofResult {
        guard initialized else {
            throw KimchiMobileError.notInitialized
        }

        return try await Task.detached(priority: .userInitiated) {
            // Generate the proof
            let ffiResult = try KimchiFfi.proveThreshold(value: value, threshold: threshold)

            // Export verifier index for WASM verification
            let verifierIndex = try KimchiFfi.exportVerifierIndex(proofHandle: ffiResult.proofHandle)

            return ProofResult(
                proofHandle: Int64(ffiResult.proofHandle),
                proofBytes: ffiResult.proofBytes,
                verifierIndex: verifierIndex,
                publicInputs: ffiResult.publicInputs,
                generationTimeMs: Int64(ffiResult.generationTimeMs),
                proofSizeBytes: Int64(ffiResult.proofSizeBytes)
            )
        }.value
    }

    /// Verify a proof by its handle.
    ///
    /// - Parameter proofHandle: Handle returned from proof generation
    /// - Returns: true if the proof is valid
    public static func verifyProof(proofHandle: Int64) throws -> Bool {
        guard initialized else {
            throw KimchiMobileError.notInitialized
        }

        return try KimchiFfi.verifyProof(proofHandle: UInt64(proofHandle))
    }

    /// Export the verifier index for a stored proof.
    ///
    /// - Parameter proofHandle: Handle to the proof
    /// - Returns: Hex-encoded verifier index for WASM verification
    public static func exportVerifierIndex(proofHandle: Int64) throws -> String {
        guard initialized else {
            throw KimchiMobileError.notInitialized
        }

        return try KimchiFfi.exportVerifierIndex(proofHandle: UInt64(proofHandle))
    }

    /// Get the SRS log2 size used by the prover.
    public static func getSrsLog2Size() throws -> UInt32 {
        return try KimchiFfi.getSrsLog2Size()
    }

    /// Free a proof from memory when no longer needed.
    ///
    /// - Parameter proofHandle: Handle to the proof to free
    public static func freeProof(proofHandle: Int64) throws {
        try KimchiFfi.freeProof(proofHandle: UInt64(proofHandle))
    }

    /// Get the library version.
    public static func version() -> String {
        return getVersion()
    }
}

// MARK: - Data Types

/// Result of proof generation.
public struct ProofResult {
    /// Handle to the proof stored in memory
    public let proofHandle: Int64
    /// Hex-encoded proof bytes
    public let proofBytes: String
    /// Hex-encoded verifier index (for WASM verification)
    public let verifierIndex: String
    /// Public inputs as hex-encoded field elements
    public let publicInputs: [String]
    /// Time taken to generate the proof in milliseconds
    public let generationTimeMs: Int64
    /// Size of the proof in bytes
    public let proofSizeBytes: Int64

    public init(
        proofHandle: Int64,
        proofBytes: String,
        verifierIndex: String,
        publicInputs: [String],
        generationTimeMs: Int64,
        proofSizeBytes: Int64
    ) {
        self.proofHandle = proofHandle
        self.proofBytes = proofBytes
        self.verifierIndex = verifierIndex
        self.publicInputs = publicInputs
        self.generationTimeMs = generationTimeMs
        self.proofSizeBytes = proofSizeBytes
    }

    /// Convert to a shareable format for transmission to a verifier.
    public func toShareable(metadata: [String: String] = [:]) -> ShareableProof {
        var fullMetadata = metadata
        fullMetadata["proofSize"] = String(proofSizeBytes)
        fullMetadata["generationTime"] = String(generationTimeMs)

        return ShareableProof(
            proof: proofBytes,
            verifierIndex: verifierIndex,
            publicInputs: publicInputs,
            metadata: fullMetadata
        )
    }
}

/// A proof format suitable for sharing/transmission to verifiers.
public struct ShareableProof {
    /// Hex-encoded proof bytes
    public let proof: String
    /// Hex-encoded verifier index (for WASM verification)
    public let verifierIndex: String
    /// Public inputs as hex-encoded field elements
    public let publicInputs: [String]
    /// Additional metadata
    public let metadata: [String: String]

    public init(
        proof: String,
        verifierIndex: String,
        publicInputs: [String],
        metadata: [String: String]
    ) {
        self.proof = proof
        self.verifierIndex = verifierIndex
        self.publicInputs = publicInputs
        self.metadata = metadata
    }

    /// Convert to JSON string for transmission.
    public func toJson() -> String {
        let publicInputsJson = publicInputs.map { "\"\($0)\"" }.joined(separator: ",")
        let metadataJson = metadata.map { "\"\($0.key)\": \"\($0.value)\"" }.joined(separator: ",")
        return """
            {
                "proof": "\(proof)",
                "verifierIndex": "\(verifierIndex)",
                "publicInputs": [\(publicInputsJson)],
                "metadata": {\(metadataJson)}
            }
            """
    }
}

// MARK: - Errors

public enum KimchiMobileError: LocalizedError {
    case notInitialized
    case invalidInput(String)
    case proofGenerationFailed(String)
    case verificationFailed(String)

    public var errorDescription: String? {
        switch self {
        case .notInitialized:
            return "Kimchi prover not initialized. Call KimchiMobile.initialize() first."
        case .invalidInput(let message):
            return "Invalid input: \(message)"
        case .proofGenerationFailed(let message):
            return "Proof generation failed: \(message)"
        case .verificationFailed(let message):
            return "Verification failed: \(message)"
        }
    }
}

// MARK: - Extensions

extension Data {
    /// Convert data to hex string.
    public var hexString: String {
        return map { String(format: "%02x", $0) }.joined()
    }

    /// Initialize from hex string.
    public init?(hexString: String) {
        let len = hexString.count / 2
        var data = Data(capacity: len)
        var index = hexString.startIndex
        for _ in 0..<len {
            let nextIndex = hexString.index(index, offsetBy: 2)
            guard let byte = UInt8(hexString[index..<nextIndex], radix: 16) else {
                return nil
            }
            data.append(byte)
            index = nextIndex
        }
        self = data
    }

    /// Compute SHA-256 hash of data.
    public func sha256() -> Data {
        var hash = [UInt8](repeating: 0, count: Int(CC_SHA256_DIGEST_LENGTH))
        withUnsafeBytes {
            _ = CC_SHA256($0.baseAddress, CC_LONG(count), &hash)
        }
        return Data(hash)
    }
}
