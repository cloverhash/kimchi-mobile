import ExpoModulesCore
import KimchiMobile

public class KimchiModule: Module {
  public func definition() -> ModuleDefinition {
    Name("Kimchi")

    // Initialize the prover with optional SRS size
    AsyncFunction("initialize") { (srsLog2Size: UInt32?) in
      try await KimchiMobile.initialize(srsLog2Size: srsLog2Size)
    }

    // Check if prover is initialized
    Function("isInitialized") {
      KimchiMobile.isInitialized()
    }

    // Generate threshold proof
    AsyncFunction("proveThreshold") { (value: String, threshold: String) -> [String: Any] in
      guard let v = UInt64(value), let t = UInt64(threshold) else {
        throw KimchiModuleError.invalidInput("Invalid number format")
      }

      let result = try await KimchiMobile.proveThreshold(value: v, threshold: t)

      return [
        "proofHandle": String(result.proofHandle),
        "proofBytes": result.proofBytes,
        "publicInputs": result.publicInputs,
        "generationTimeMs": result.generationTimeMs,
        "proofSizeBytes": result.proofSizeBytes
      ]
    }

    // Verify a proof by handle
    AsyncFunction("verifyProof") { (proofHandle: String) -> Bool in
      guard let handle = Int64(proofHandle) else {
        throw KimchiModuleError.invalidInput("Invalid proof handle")
      }
      return try KimchiMobile.verifyProof(proofHandle: handle)
    }

    // Export verifier index
    AsyncFunction("exportVerifierIndex") { (proofHandle: String) -> String in
      guard let handle = Int64(proofHandle) else {
        throw KimchiModuleError.invalidInput("Invalid proof handle")
      }
      return try KimchiMobile.exportVerifierIndex(proofHandle: handle)
    }

    // Free proof memory
    AsyncFunction("freeProof") { (proofHandle: String) in
      guard let handle = Int64(proofHandle) else {
        throw KimchiModuleError.invalidInput("Invalid proof handle")
      }
      try KimchiMobile.freeProof(proofHandle: handle)
    }

    // Get SRS log2 size
    Function("getSrsLog2Size") { () -> UInt32? in
      try? KimchiMobile.getSrsLog2Size()
    }

    // Get library version
    Function("getVersion") {
      KimchiMobile.version()
    }
  }
}

enum KimchiModuleError: Error {
  case invalidInput(String)
}

extension KimchiModuleError: LocalizedError {
  var errorDescription: String? {
    switch self {
    case .invalidInput(let message):
      return "Invalid input: \(message)"
    }
  }
}
