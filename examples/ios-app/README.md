# iOS Example App

This example demonstrates using the Kimchi Mobile native Swift library directly in an iOS app.

## Setup

1. Build the native libraries:
   ```bash
   cd ../..
   ./scripts/build-ios.sh
   ```

2. Add the Swift Package to your Xcode project (not yet published to a registry):
   - File > Add Package Dependencies
   - Enter local path: `path/to/kimchi-mobile/packages/swift`
   - Or use the git URL once pushed to a remote repository

3. Use in your SwiftUI view or UIKit controller:
   ```swift
   import KimchiMobile

   struct ContentView: View {
       @State private var proofResult: String = ""

       var body: some View {
           VStack {
               Text(proofResult)
               Button("Generate Proof") {
                   Task {
                       await generateProof()
                   }
               }
           }
       }

       func generateProof() async {
           do {
               // Initialize the prover (do this once at app startup)
               try await KimchiMobile.initialize(srsLog2Size: 12)

               // Generate a proof that 42 < 100
               let proof = try await KimchiMobile.proveThreshold(value: 42, threshold: 100)
               proofResult = "Proof generated in \(proof.generationTimeMs)ms"

               // Verify the proof
               let isValid = try KimchiMobile.verifyProof(proofHandle: proof.proofHandle)
               proofResult += "\nProof valid: \(isValid)"

               // Export for remote verification
               let shareable = try proof.toShareable()
               print("Shareable proof JSON: \(try shareable.toJson())")

               // Free the proof when done
               try KimchiMobile.freeProof(proofHandle: proof.proofHandle)
           } catch {
               proofResult = "Error: \(error.localizedDescription)"
           }
       }
   }
   ```

## Key Points

- Requires iOS 15.0+ or macOS 12.0+
- Uses Swift async/await for proof generation
- Proof generation is CPU-intensive; shows activity indicator
- Call `freeProof()` when done to release memory
- Use smaller SRS sizes (10-12) during development for faster testing
