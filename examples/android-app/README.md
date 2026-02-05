# Android Example App

This example demonstrates using the Kimchi Mobile native Kotlin library directly in an Android app.

## Setup

1. Build the native libraries:
   ```bash
   cd ../..
   ./scripts/build-android.sh
   ```

2. Add the dependency to your app's `build.gradle.kts`:
   ```kotlin
   dependencies {
       // From local path (not yet published to Maven)
       implementation(files("path/to/kimchi-mobile/packages/kotlin/build/outputs/aar/kimchi-mobile-release.aar"))
   }
   ```

3. Initialize and use in your Activity/Fragment:
   ```kotlin
   import com.kimchi.mobile.KimchiMobile

   class MainActivity : AppCompatActivity() {
       override fun onCreate(savedInstanceState: Bundle?) {
           super.onCreate(savedInstanceState)

           // Initialize the prover (do this once at app startup)
           lifecycleScope.launch {
               val initialized = KimchiMobile.initialize(applicationContext, srsLog2Size = 12)
               if (initialized) {
                   // Generate a proof
                   KimchiMobile.proveThreshold(42L, 100L)
                       .onSuccess { proof ->
                           Log.d("Kimchi", "Proof generated in ${proof.generationTimeMs}ms")

                           // Verify the proof
                           KimchiMobile.verifyProof(proof.proofHandle)
                               .onSuccess { valid ->
                                   Log.d("Kimchi", "Proof valid: $valid")
                               }

                           // Free the proof when done
                           KimchiMobile.freeProof(proof.proofHandle)
                       }
                       .onFailure { error ->
                           Log.e("Kimchi", "Proof generation failed", error)
                       }
               }
           }
       }
   }
   ```

## Key Points

- The library requires API 26+ (Android 8.0)
- Proof generation is CPU-intensive; always run on a background thread
- Call `freeProof()` when done to release memory
- Use smaller SRS sizes (10-12) during development for faster testing
