//! Error types for the Kimchi mobile prover.

use thiserror::Error;

/// Result type alias using ProverError
pub type Result<T> = std::result::Result<T, ProverError>;

/// Errors that can occur during proof generation and verification.
#[derive(Error, Debug)]
pub enum ProverError {
    /// Error during circuit setup
    #[error("Circuit setup failed: {0}")]
    SetupError(String),

    /// Error during witness generation
    #[error("Witness generation failed: {0}")]
    WitnessError(String),

    /// Error during proof generation
    #[error("Proof generation failed: {0}")]
    ProvingError(String),

    /// Error during proof verification
    #[error("Proof verification failed: {0}")]
    VerificationError(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Constraint system error
    #[error("Constraint system error: {0}")]
    ConstraintError(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for ProverError {
    fn from(err: std::io::Error) -> Self {
        ProverError::SerializationError(err.to_string())
    }
}

impl From<serde_json::Error> for ProverError {
    fn from(err: serde_json::Error) -> Self {
        ProverError::SerializationError(err.to_string())
    }
}
