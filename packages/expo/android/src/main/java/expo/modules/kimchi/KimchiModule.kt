package expo.modules.kimchi

import android.content.Context
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.Promise
import com.kimchi.mobile.KimchiMobile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class KimchiModule : Module() {
    private val scope = CoroutineScope(Dispatchers.Default)

    private val context: Context
        get() = requireNotNull(appContext.reactContext)

    override fun definition() = ModuleDefinition {
        Name("Kimchi")

        // Initialize the prover with optional SRS size
        AsyncFunction("initialize") { srsLog2Size: Int?, promise: Promise ->
            scope.launch {
                try {
                    val success = KimchiMobile.initialize(context, srsLog2Size)
                    if (success) {
                        promise.resolve(null)
                    } else {
                        promise.reject("INIT_ERROR", "Failed to initialize Kimchi prover", null)
                    }
                } catch (e: Exception) {
                    promise.reject("INIT_ERROR", e.message, e)
                }
            }
        }

        // Check if prover is initialized
        Function("isInitialized") {
            KimchiMobile.isInitialized()
        }

        // Generate threshold proof
        AsyncFunction("proveThreshold") { value: String, threshold: String, promise: Promise ->
            scope.launch {
                try {
                    val v = value.toULong()
                    val t = threshold.toULong()

                    KimchiMobile.proveThreshold(v.toLong(), t.toLong())
                        .onSuccess { result ->
                            promise.resolve(mapOf(
                                "proofHandle" to result.proofHandle.toString(),
                                "proofBytes" to result.proofBytes,
                                "publicInputs" to result.publicInputs,
                                "generationTimeMs" to result.generationTimeMs,
                                "proofSizeBytes" to result.proofSizeBytes
                            ))
                        }
                        .onFailure { error ->
                            promise.reject("PROVE_ERROR", error.message, error)
                        }
                } catch (e: Exception) {
                    promise.reject("PROVE_ERROR", e.message, e)
                }
            }
        }

        // Verify a proof by handle
        AsyncFunction("verifyProof") { proofHandle: String, promise: Promise ->
            scope.launch {
                try {
                    val handle = proofHandle.toLong()
                    KimchiMobile.verifyProof(handle)
                        .onSuccess { isValid ->
                            promise.resolve(isValid)
                        }
                        .onFailure { error ->
                            promise.reject("VERIFY_ERROR", error.message, error)
                        }
                } catch (e: Exception) {
                    promise.reject("VERIFY_ERROR", e.message, e)
                }
            }
        }

        // Export verifier index
        AsyncFunction("exportVerifierIndex") { proofHandle: String, promise: Promise ->
            scope.launch {
                try {
                    val handle = proofHandle.toLong()
                    KimchiMobile.exportVerifierIndex(handle)
                        .onSuccess { verifierIndex ->
                            promise.resolve(verifierIndex)
                        }
                        .onFailure { error ->
                            promise.reject("EXPORT_ERROR", error.message, error)
                        }
                } catch (e: Exception) {
                    promise.reject("EXPORT_ERROR", e.message, e)
                }
            }
        }

        // Free proof memory
        AsyncFunction("freeProof") { proofHandle: String, promise: Promise ->
            scope.launch {
                try {
                    val handle = proofHandle.toLong()
                    val success = KimchiMobile.freeProof(handle)
                    if (success) {
                        promise.resolve(null)
                    } else {
                        promise.reject("FREE_ERROR", "Failed to free proof", null)
                    }
                } catch (e: Exception) {
                    promise.reject("FREE_ERROR", e.message, e)
                }
            }
        }

        // Get SRS log2 size
        Function("getSrsLog2Size") {
            KimchiMobile.getSrsLog2Size()
        }

        // Get library version
        Function("getVersion") {
            KimchiMobile.getLibraryVersion()
        }
    }
}
