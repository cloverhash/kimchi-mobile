//! # Kimchi Mobile Prover
//!
//! A mobile-friendly wrapper around Kimchi for generating Mina-compatible ZK proofs.
//!
//! ## Overview
//!
//! This crate provides the core proving infrastructure for generating
//! zero-knowledge proofs on mobile devices using the Kimchi proof system.
//!
//! ## Example
//!
//! ```rust,ignore
//! use kimchi_prover::{KimchiProver, ProverConfig, ThresholdCircuit};
//!
//! let mut prover = KimchiProver::new();
//! prover.init_srs()?;
//!
//! // Create a threshold circuit
//! let circuit = ThresholdCircuit::new(100); // threshold = 100
//!
//! // Setup the circuit
//! let (prover_index, verifier_index) = prover.setup(
//!     circuit.gates(),
//!     circuit.num_public_inputs()
//! )?;
//!
//! // Generate witness for a private value
//! let (witness, public_inputs) = circuit.generate_witness(50)?; // value = 50
//!
//! // Generate proof
//! let proof = prover.prove(&prover_index, witness)?;
//!
//! // Verify
//! let valid = prover.verify(&verifier_index, &proof, &public_inputs)?;
//! assert!(valid);
//! ```

pub mod circuits;
pub mod error;
pub mod gadgets;
pub mod prover;
pub mod types;

pub use error::{ProverError, Result};
pub use prover::{KimchiProver, ProverConfig, VestaOpeningProof, COLUMNS, FULL_ROUNDS};
pub use types::FieldElement;

// Re-export circuit types
pub use circuits::ThresholdCircuit;

// Re-export gadget types
pub use gadgets::{RsaGadget, RsaWitness, Sha256Gadget, Sha256Witness};

// Re-export key types from the proof-systems crates
pub use mina_curves::pasta::{Fp, Fq, Pallas, Vesta};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_element() {
        let fe = FieldElement::from_u64(12345);
        assert!(!fe.to_bytes().is_empty());
    }

    #[test]
    fn test_prover_creation() {
        let _prover = KimchiProver::new();
    }
}
