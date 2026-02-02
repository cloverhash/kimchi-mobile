//! Core types for the Kimchi mobile prover.

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use mina_curves::pasta::Fp;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A field element in the Pallas scalar field (used by Kimchi/Mina).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldElement(pub Fp);

impl FieldElement {
    /// Create a field element from a u64 value.
    pub fn from_u64(value: u64) -> Self {
        Self(Fp::from(value))
    }

    /// Create a field element from a decimal string.
    pub fn from_decimal(s: &str) -> Result<Self, String> {
        Fp::from_str(s)
            .map(Self)
            .map_err(|_| format!("Invalid decimal string: {}", s))
    }

    /// Get the inner Fp value.
    pub fn inner(&self) -> &Fp {
        &self.0
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.0.serialize_compressed(&mut bytes).unwrap();
        bytes
    }

    /// Create from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Fp::deserialize_compressed(bytes)
            .map(Self)
            .map_err(|e| format!("Deserialization error: {}", e))
    }
}

impl From<u64> for FieldElement {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<Fp> for FieldElement {
    fn from(fp: Fp) -> Self {
        Self(fp)
    }
}

/// Public inputs to a circuit.
#[derive(Clone, Debug, Default)]
pub struct PublicInput {
    pub values: Vec<FieldElement>,
}

impl PublicInput {
    /// Create a new empty public input.
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Add a field element to the public inputs.
    pub fn push(&mut self, value: FieldElement) {
        self.values.push(value);
    }

    /// Add a u64 value as a field element.
    pub fn push_u64(&mut self, value: u64) {
        self.values.push(FieldElement::from_u64(value));
    }

    /// Get the inner Fp values.
    pub fn to_fp_vec(&self) -> Vec<Fp> {
        self.values.iter().map(|f| f.0).collect()
    }
}

/// Witness (private inputs) for a circuit.
#[derive(Clone, Debug, Default)]
pub struct Witness {
    /// Values for each column in the witness.
    pub columns: Vec<Vec<FieldElement>>,
}

impl Witness {
    /// Create a new empty witness.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    /// Create a witness with the specified number of columns.
    pub fn with_columns(num_columns: usize) -> Self {
        Self {
            columns: vec![Vec::new(); num_columns],
        }
    }

    /// Set a value at a specific column and row.
    pub fn set(&mut self, column: usize, row: usize, value: FieldElement) {
        while self.columns.len() <= column {
            self.columns.push(Vec::new());
        }
        while self.columns[column].len() <= row {
            self.columns[column].push(FieldElement::from_u64(0));
        }
        self.columns[column][row] = value;
    }

    /// Get a value at a specific column and row.
    pub fn get(&self, column: usize, row: usize) -> Option<&FieldElement> {
        self.columns.get(column)?.get(row)
    }
}

/// Serializable witness data for transport.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WitnessData {
    /// Column data as hex strings.
    pub columns: Vec<Vec<String>>,
}

impl From<&Witness> for WitnessData {
    fn from(witness: &Witness) -> Self {
        Self {
            columns: witness
                .columns
                .iter()
                .map(|col| col.iter().map(|f| hex::encode(f.to_bytes())).collect())
                .collect(),
        }
    }
}
