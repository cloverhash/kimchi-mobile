//! Kimchi proof generation and verification.
//!
//! This module provides the main prover interface for generating and verifying
//! Kimchi proofs compatible with Mina.

use crate::error::{ProverError, Result};

use kimchi::circuits::constraints::ConstraintSystem;
use kimchi::circuits::gate::CircuitGate;
use kimchi::groupmap::GroupMap;
use kimchi::proof::ProverProof;
use kimchi::prover_index::ProverIndex;
use kimchi::verifier::verify;
use kimchi::verifier_index::VerifierIndex;
use mina_curves::pasta::{Fp, Vesta, VestaParameters};
use mina_poseidon::constants::PlonkSpongeConstantsKimchi;
use mina_poseidon::sponge::{DefaultFqSponge, DefaultFrSponge};
use poly_commitment::ipa::{OpeningProof, SRS};
use std::sync::Arc;

/// Number of columns in Kimchi witness
pub const COLUMNS: usize = 15;

/// Type aliases for Kimchi's sponge types with Mina's parameters
pub type VestaBaseSponge =
    DefaultFqSponge<VestaParameters, PlonkSpongeConstantsKimchi, FULL_ROUNDS>;
pub type VestaScalarSponge = DefaultFrSponge<Fp, PlonkSpongeConstantsKimchi, FULL_ROUNDS>;

/// Type alias for the opening proof used by Vesta
pub type VestaOpeningProof = OpeningProof<Vesta, FULL_ROUNDS>;

/// Configuration for the prover.
#[derive(Clone, Debug)]
pub struct ProverConfig {
    /// SRS depth (log2 of max circuit size)
    pub srs_log2_size: usize,
    /// Enable debug output
    pub debug: bool,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            srs_log2_size: 14, // 2^14 = 16384 rows
            debug: false,
        }
    }
}

/// The main Kimchi prover for generating and verifying Mina-compatible proofs.
pub struct KimchiProver {
    config: ProverConfig,
    srs: Option<Arc<SRS<Vesta>>>,
}

impl KimchiProver {
    /// Create a new prover with default configuration.
    pub fn new() -> Self {
        Self {
            config: ProverConfig::default(),
            srs: None,
        }
    }

    /// Create a new prover with custom configuration.
    pub fn with_config(config: ProverConfig) -> Self {
        Self { config, srs: None }
    }

    /// Get the prover configuration.
    pub fn config(&self) -> &ProverConfig {
        &self.config
    }

    /// Initialize the SRS (Structured Reference String).
    /// This is a one-time setup that can be reused across multiple proofs.
    pub fn init_srs(&mut self) -> Result<()> {
        if self.srs.is_some() {
            return Ok(());
        }

        let depth = 1 << self.config.srs_log2_size;

        if self.config.debug {
            log::info!("Creating SRS with depth {}...", depth);
        }

        let srs = SRS::<Vesta>::create_parallel(depth);

        if self.config.debug {
            log::info!("SRS created successfully");
        }

        self.srs = Some(Arc::new(srs));
        Ok(())
    }

    /// Get the SRS, initializing if needed
    fn get_srs(&mut self) -> Result<Arc<SRS<Vesta>>> {
        if self.srs.is_none() {
            self.init_srs()?;
        }
        Ok(self.srs.clone().unwrap())
    }

    /// Setup a circuit and create prover/verifier indices
    pub fn setup(
        &mut self,
        gates: Vec<CircuitGate<Fp>>,
        num_public_inputs: usize,
    ) -> Result<(
        ProverIndex<FULL_ROUNDS, Vesta, SRS<Vesta>>,
        VerifierIndex<FULL_ROUNDS, Vesta, SRS<Vesta>>,
    )> {
        let srs = self.get_srs()?;

        if self.config.debug {
            log::info!("Creating constraint system with {} gates...", gates.len());
        }

        // Create constraint system
        let cs = ConstraintSystem::create(gates)
            .public(num_public_inputs)
            .build()
            .map_err(|e| ProverError::SetupError(format!("Constraint system error: {:?}", e)))?;

        if self.config.debug {
            log::info!(
                "Constraint system created, domain size: {}",
                cs.domain.d1.size
            );
        }

        // Get the endomorphism coefficient
        let (endo_q, _endo_r) = poly_commitment::ipa::endos::<mina_curves::pasta::Pallas>();

        // Create prover index
        let prover_index = ProverIndex::create(cs, endo_q, srs, false);

        // Create verifier index from prover index
        let verifier_index = prover_index.verifier_index();

        if self.config.debug {
            log::info!("Prover and verifier indices created");
        }

        Ok((prover_index, verifier_index))
    }

    /// Generate a proof
    pub fn prove(
        &self,
        prover_index: &ProverIndex<FULL_ROUNDS, Vesta, SRS<Vesta>>,
        witness: [Vec<Fp>; COLUMNS],
    ) -> Result<ProverProof<Vesta, VestaOpeningProof, FULL_ROUNDS>> {
        if self.config.debug {
            log::info!("Generating proof...");
        }

        let group_map = <Vesta as poly_commitment::commitment::CommitmentCurve>::Map::setup();

        let mut rng = rand::rngs::OsRng;

        let proof = ProverProof::create::<VestaBaseSponge, VestaScalarSponge, _>(
            &group_map,
            witness,
            &[], // no runtime tables
            prover_index,
            &mut rng,
        )
        .map_err(|e| ProverError::ProvingError(format!("Proof generation failed: {:?}", e)))?;

        if self.config.debug {
            log::info!("Proof generated successfully");
        }

        Ok(proof)
    }

    /// Verify a proof
    pub fn verify(
        &self,
        verifier_index: &VerifierIndex<FULL_ROUNDS, Vesta, SRS<Vesta>>,
        proof: &ProverProof<Vesta, VestaOpeningProof, FULL_ROUNDS>,
        public_inputs: &[Fp],
    ) -> Result<bool> {
        if self.config.debug {
            log::info!("Verifying proof...");
        }

        let group_map = <Vesta as poly_commitment::commitment::CommitmentCurve>::Map::setup();

        let result = verify::<
            FULL_ROUNDS,
            Vesta,
            VestaBaseSponge,
            VestaScalarSponge,
            VestaOpeningProof,
        >(&group_map, verifier_index, proof, public_inputs);

        match result {
            Ok(_) => {
                if self.config.debug {
                    log::info!("Proof verified successfully");
                }
                Ok(true)
            }
            Err(e) => {
                if self.config.debug {
                    log::warn!("Proof verification failed: {:?}", e);
                }
                Ok(false)
            }
        }
    }
}

impl Default for KimchiProver {
    fn default() -> Self {
        Self::new()
    }
}

/// Re-export FULL_ROUNDS constant for use in type signatures
pub use mina_poseidon::pasta::FULL_ROUNDS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_init() {
        let mut prover = KimchiProver::with_config(ProverConfig {
            srs_log2_size: 10, // Smaller for faster tests
            debug: false,
        });

        let result = prover.init_srs();
        assert!(result.is_ok());
    }
}
