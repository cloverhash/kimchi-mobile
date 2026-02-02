//! FFI bindings for Kimchi mobile prover.
//!
//! This crate provides UniFFI bindings that generate native bindings for:
//! - Kotlin (Android)
//! - Swift (iOS)
//!
//! Uses proc-macro approach (no UDL file).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, RwLock};

use ark_serialize::CanonicalSerialize;
use kimchi::proof::ProverProof;
use kimchi::verifier_index::VerifierIndex;
use kimchi_prover::{
    Fp, KimchiProver, ProverConfig, ThresholdCircuit, Vesta, VestaOpeningProof, FULL_ROUNDS,
};
use poly_commitment::ipa::SRS;

// Generate UniFFI scaffolding via proc macros
uniffi::setup_scaffolding!();

/// Global initialization state.
static INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Global prover instance (lazy initialized).
static PROVER: OnceLock<Mutex<KimchiProver>> = OnceLock::new();

/// Counter for proof handles.
static PROOF_COUNTER: OnceLock<Mutex<u64>> = OnceLock::new();

/// In-memory storage for proofs (keyed by handle ID).
static PROOF_STORE: OnceLock<RwLock<HashMap<u64, StoredProof>>> = OnceLock::new();

/// Stored proof data that includes the verifier index with its SRS reference.
struct StoredProof {
    proof: ProverProof<Vesta, VestaOpeningProof, FULL_ROUNDS>,
    verifier_index: VerifierIndex<FULL_ROUNDS, Vesta, SRS<Vesta>>,
    public_inputs: Vec<Fp>,
}

/// Error types exposed via FFI.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum KimchiError {
    #[error("Setup error: {0}")]
    SetupError(String),

    #[error("Proving error: {0}")]
    ProvingError(String),

    #[error("Verification error: {0}")]
    VerificationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Proof not found: {0}")]
    ProofNotFound(String),
}

/// Result of proof generation.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ProofResult {
    /// Handle to the proof stored in memory (for verification).
    pub proof_handle: u64,
    /// Serialized proof (hex-encoded, for transmission/storage).
    pub proof_bytes: String,
    /// Public inputs as hex-encoded field elements.
    pub public_inputs: Vec<String>,
    /// Time taken in milliseconds.
    pub generation_time_ms: u64,
    /// Size of the proof in bytes.
    pub proof_size_bytes: u64,
}

/// Get the next proof ID.
fn get_next_proof_id() -> u64 {
    let counter = PROOF_COUNTER.get_or_init(|| Mutex::new(0));
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    *guard
}

/// Store a proof and return its handle.
fn store_proof(proof: StoredProof) -> u64 {
    let store = PROOF_STORE.get_or_init(|| RwLock::new(HashMap::new()));
    let id = get_next_proof_id();
    store.write().unwrap().insert(id, proof);
    id
}

fn get_stored_proof(
    id: u64,
) -> Option<std::sync::RwLockReadGuard<'static, HashMap<u64, StoredProof>>> {
    let store = PROOF_STORE.get()?;
    let guard = store.read().ok()?;
    if guard.contains_key(&id) {
        Some(guard)
    } else {
        None
    }
}

/// Initialize the prover. Call this once at app startup.
///
/// # Arguments
/// * `srs_log2_size` - Log2 of the SRS size. Larger values support bigger circuits
///   but use more memory. Default is 14 (16384 rows). Use 10-12 for testing.
#[uniffi::export]
pub fn init_prover(srs_log2_size: Option<u32>) -> Result<(), KimchiError> {
    let _ = INITIALIZED.get_or_init(|| {
        log::info!("Kimchi mobile prover initialized");
        true
    });

    // Initialize the prover with configuration
    let _ = PROVER.get_or_init(|| {
        let config = ProverConfig {
            srs_log2_size: srs_log2_size.unwrap_or(14) as usize,
            debug: false,
        };
        Mutex::new(KimchiProver::with_config(config))
    });

    // Initialize storage
    let _ = PROOF_STORE.get_or_init(|| RwLock::new(HashMap::new()));

    Ok(())
}

/// Verify a proof using its handle.
///
/// # Arguments
/// * `proof_handle` - Handle returned from proof generation
///
/// # Returns
/// `true` if the proof is valid, `false` otherwise.
#[uniffi::export]
pub fn verify_proof(proof_handle: u64) -> Result<bool, KimchiError> {
    if INITIALIZED.get().is_none() {
        return Err(KimchiError::SetupError(
            "Prover not initialized. Call init_prover() first.".into(),
        ));
    }

    // Get the stored proof
    let store_guard = get_stored_proof(proof_handle).ok_or_else(|| {
        KimchiError::ProofNotFound(format!("No proof with handle {}", proof_handle))
    })?;

    let stored = store_guard.get(&proof_handle).ok_or_else(|| {
        KimchiError::ProofNotFound(format!("No proof with handle {}", proof_handle))
    })?;

    // Get prover to verify
    let prover_mutex = PROVER
        .get()
        .ok_or_else(|| KimchiError::SetupError("Prover not initialized".into()))?;

    let prover = prover_mutex
        .lock()
        .map_err(|e| KimchiError::SetupError(format!("Failed to lock prover: {}", e)))?;

    // Verify
    prover
        .verify(&stored.verifier_index, &stored.proof, &stored.public_inputs)
        .map_err(|e| KimchiError::VerificationError(e.to_string()))
}

/// Free a proof from memory.
///
/// Call this when you no longer need to verify a proof to free memory.
#[uniffi::export]
pub fn free_proof(proof_handle: u64) -> Result<(), KimchiError> {
    let store = PROOF_STORE
        .get()
        .ok_or_else(|| KimchiError::SetupError("Store not initialized".into()))?;

    let mut guard = store
        .write()
        .map_err(|e| KimchiError::SetupError(format!("Failed to lock store: {}", e)))?;

    guard.remove(&proof_handle);
    Ok(())
}

/// Get the library version.
#[uniffi::export]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Serialize the verifier index for a stored proof.
///
/// This returns the verifier index in a format suitable for the WASM verifier.
/// The SRS is NOT included - the WASM verifier will regenerate it based on
/// the srs_log2_size from get_srs_log2_size().
///
/// # Arguments
/// * `proof_handle` - Handle to a stored proof
///
/// # Returns
/// Hex-encoded MessagePack serialized verifier index (without SRS)
#[uniffi::export]
pub fn export_verifier_index(proof_handle: u64) -> Result<String, KimchiError> {
    let store_guard = get_stored_proof(proof_handle).ok_or_else(|| {
        KimchiError::ProofNotFound(format!("No proof with handle {}", proof_handle))
    })?;

    let stored = store_guard.get(&proof_handle).ok_or_else(|| {
        KimchiError::ProofNotFound(format!("No proof with handle {}", proof_handle))
    })?;

    let vi_bytes = rmp_serde::to_vec(&stored.verifier_index).map_err(|e| {
        KimchiError::SerializationError(format!("Failed to serialize verifier index: {}", e))
    })?;

    Ok(hex::encode(vi_bytes))
}

/// Get the SRS log2 size used by the prover.
///
/// Pass this value to the WASM verifier's init_verifier() to ensure
/// the same SRS is used for verification.
///
/// # Returns
/// The log2 of the SRS size (e.g., 14 means 2^14 = 16384 rows)
#[uniffi::export]
pub fn get_srs_log2_size() -> Result<u32, KimchiError> {
    let prover_mutex = PROVER
        .get()
        .ok_or_else(|| KimchiError::SetupError("Prover not initialized".into()))?;

    let prover = prover_mutex
        .lock()
        .map_err(|e| KimchiError::SetupError(format!("Failed to lock prover: {}", e)))?;

    Ok(prover.config().srs_log2_size as u32)
}

/// Generate a proof that a private value is less than a public threshold.
///
/// This proves: "I know a secret value V such that V < threshold"
/// without revealing what V actually is.
///
/// # Arguments
/// * `value` - The private value (will not be revealed)
/// * `threshold` - The public threshold to compare against
///
/// # Returns
/// A ProofResult containing the proof handle and serialized proof data.
///
/// # Example
/// ```ignore
/// // Prove that my secret number (50) is less than 100
/// let result = prove_threshold(50, 100)?;
/// assert!(result.is_valid); // true because 50 < 100
/// ```
#[uniffi::export]
pub fn prove_threshold(value: u64, threshold: u64) -> Result<ProofResult, KimchiError> {
    if INITIALIZED.get().is_none() {
        return Err(KimchiError::SetupError(
            "Prover not initialized. Call init_prover() first.".into(),
        ));
    }

    let start_time = std::time::Instant::now();

    // Get the prover
    let prover_mutex = PROVER
        .get()
        .ok_or_else(|| KimchiError::SetupError("Prover not initialized".into()))?;

    let mut prover = prover_mutex
        .lock()
        .map_err(|e| KimchiError::SetupError(format!("Failed to lock prover: {}", e)))?;

    // Create the threshold circuit
    let circuit = ThresholdCircuit::new(threshold);

    // Setup the circuit (creates prover and verifier indices)
    let (prover_index, verifier_index) = prover
        .setup(circuit.gates(), circuit.num_public_inputs())
        .map_err(|e| KimchiError::SetupError(format!("Circuit setup failed: {}", e)))?;

    // Generate witness
    let (witness, public_inputs) = circuit
        .generate_witness(value)
        .map_err(|e| KimchiError::ProvingError(format!("Witness generation failed: {}", e)))?;

    // Generate proof
    let proof = prover
        .prove(&prover_index, witness)
        .map_err(|e| KimchiError::ProvingError(format!("Proof generation failed: {}", e)))?;

    let generation_time_ms = start_time.elapsed().as_millis() as u64;

    // Serialize proof for transmission
    let proof_bytes = rmp_serde::to_vec(&proof).map_err(|e| {
        KimchiError::SerializationError(format!("Failed to serialize proof: {}", e))
    })?;
    let proof_size_bytes = proof_bytes.len() as u64;
    let proof_hex = hex::encode(&proof_bytes);

    // Serialize public inputs
    let public_inputs_hex: Vec<String> = public_inputs
        .iter()
        .map(|fp| {
            let mut bytes = Vec::new();
            fp.serialize_compressed(&mut bytes).unwrap();
            hex::encode(bytes)
        })
        .collect();

    // Store proof for later verification
    let proof_handle = store_proof(StoredProof {
        proof,
        verifier_index,
        public_inputs,
    });

    Ok(ProofResult {
        proof_handle,
        proof_bytes: proof_hex,
        public_inputs: public_inputs_hex,
        generation_time_ms,
        proof_size_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Use smaller SRS for faster tests
        init_prover(Some(10)).expect("Failed to initialize");
    }

    #[test]
    fn test_version() {
        let version = get_version();
        assert_eq!(version, "0.1.0");
    }
}
