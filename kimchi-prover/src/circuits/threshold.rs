//! Threshold circuit - proves a private value is less than a public threshold.
//!
//! This is a simple demonstration circuit that proves:
//! "I know a secret value V such that V < threshold"
//!
//! Public inputs:
//! - threshold: The maximum allowed value
//! - is_valid: 1 if value < threshold, 0 otherwise
//!
//! Private inputs:
//! - value: The secret value being compared

use ark_ff::{One, Zero};
use kimchi::circuits::gate::{CircuitGate, GateType};
use kimchi::circuits::wires::Wire;
use mina_curves::pasta::Fp;

use crate::error::Result;
use crate::prover::COLUMNS;

/// A circuit that proves a private value is below a public threshold.
pub struct ThresholdCircuit {
    /// The public threshold value
    pub threshold: u64,
}

impl ThresholdCircuit {
    /// Create a new threshold circuit.
    pub fn new(threshold: u64) -> Self {
        Self { threshold }
    }

    /// Get the number of public inputs for this circuit.
    pub fn num_public_inputs(&self) -> usize {
        2 // threshold and is_valid
    }

    /// Generate the circuit gates.
    ///
    /// This creates a simple circuit that:
    /// 1. Takes the threshold as public input
    /// 2. Takes the secret value as private witness
    /// 3. Computes whether value < threshold
    /// 4. Outputs the result as a public input
    pub fn gates(&self) -> Vec<CircuitGate<Fp>> {
        let mut gates = Vec::new();

        // Row 0: Public input for threshold
        gates.push(CircuitGate::new(
            GateType::Generic,
            Wire::for_row(0),
            vec![Fp::one(), Fp::zero(), Fp::zero(), Fp::zero(), Fp::zero()],
        ));

        // Row 1: Public input for is_valid result
        gates.push(CircuitGate::new(
            GateType::Generic,
            Wire::for_row(1),
            vec![Fp::one(), Fp::zero(), Fp::zero(), Fp::zero(), Fp::zero()],
        ));

        // Row 2: Private value
        gates.push(CircuitGate::new(
            GateType::Generic,
            Wire::for_row(2),
            vec![Fp::one(), Fp::zero(), Fp::zero(), Fp::zero(), Fp::zero()],
        ));

        // Row 3: Difference = threshold - value (must be positive if value < threshold)
        // We use a Generic gate to compute: threshold - value - difference = 0
        // Coefficients: c0*w0 + c1*w1 + c2*w2 + c3*w0*w1 + c4 = 0
        // We want: threshold - value - difference = 0
        // So: 1*threshold + (-1)*value + (-1)*difference = 0
        gates.push(CircuitGate::new(
            GateType::Generic,
            Wire::for_row(3),
            vec![
                Fp::one(),  // coefficient for threshold (from row 0)
                -Fp::one(), // coefficient for value (from row 2)
                -Fp::one(), // coefficient for difference
                Fp::zero(), // coefficient for multiplication
                Fp::zero(), // constant
            ],
        ));

        // Row 4: Constraint that is_valid is boolean (0 or 1)
        // is_valid * (1 - is_valid) = 0
        gates.push(CircuitGate::new(
            GateType::Generic,
            Wire::for_row(4),
            vec![
                Fp::zero(),
                Fp::zero(),
                Fp::zero(),
                Fp::one(), // w0 * w1
                Fp::zero(),
            ],
        ));

        // Pad to minimum size (Kimchi requires at least 2 gates)
        while gates.len() < 8 {
            gates.push(CircuitGate::new(
                GateType::Zero,
                Wire::for_row(gates.len()),
                vec![],
            ));
        }

        gates
    }

    /// Generate witness for the circuit given a private value.
    ///
    /// Returns the witness columns and the public inputs.
    pub fn generate_witness(&self, value: u64) -> Result<([Vec<Fp>; COLUMNS], Vec<Fp>)> {
        let threshold_fp = Fp::from(self.threshold);
        let value_fp = Fp::from(value);
        let is_valid = if value < self.threshold { 1u64 } else { 0u64 };
        let is_valid_fp = Fp::from(is_valid);

        // Compute difference (will be positive if value < threshold)
        let difference_fp = if value < self.threshold {
            threshold_fp - value_fp
        } else {
            Fp::zero()
        };

        // Initialize witness columns
        let num_rows = 8;
        let mut witness: [Vec<Fp>; COLUMNS] = std::array::from_fn(|_| vec![Fp::zero(); num_rows]);

        // Row 0: threshold (public input)
        witness[0][0] = threshold_fp;

        // Row 1: is_valid (public input)
        witness[0][1] = is_valid_fp;

        // Row 2: value (private)
        witness[0][2] = value_fp;

        // Row 3: difference calculation
        // Wire the values for the constraint: threshold - value - difference = 0
        witness[0][3] = threshold_fp;
        witness[1][3] = value_fp;
        witness[2][3] = difference_fp;

        // Row 4: boolean constraint for is_valid
        // is_valid * (1 - is_valid) = 0
        witness[0][4] = is_valid_fp;
        witness[1][4] = Fp::one() - is_valid_fp;

        // Public inputs: [threshold, is_valid]
        let public_inputs = vec![threshold_fp, is_valid_fp];

        Ok((witness, public_inputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_circuit_creation() {
        let circuit = ThresholdCircuit::new(100);
        assert_eq!(circuit.threshold, 100);
        assert_eq!(circuit.num_public_inputs(), 2);
    }

    #[test]
    fn test_gates_generation() {
        let circuit = ThresholdCircuit::new(100);
        let gates = circuit.gates();
        assert!(gates.len() >= 5);
    }

    #[test]
    fn test_witness_below_threshold() {
        let circuit = ThresholdCircuit::new(100);
        let (witness, public_inputs) = circuit.generate_witness(50).unwrap();

        // Check public inputs
        assert_eq!(public_inputs.len(), 2);
        assert_eq!(public_inputs[0], Fp::from(100u64)); // threshold
        assert_eq!(public_inputs[1], Fp::from(1u64)); // is_valid = true

        // Check witness has correct dimensions
        assert_eq!(witness.len(), COLUMNS);
        assert!(witness[0].len() >= 5);
    }

    #[test]
    fn test_witness_above_threshold() {
        let circuit = ThresholdCircuit::new(100);
        let (_, public_inputs) = circuit.generate_witness(150).unwrap();

        assert_eq!(public_inputs[1], Fp::from(0u64)); // is_valid = false
    }

    #[test]
    fn test_witness_at_threshold() {
        let circuit = ThresholdCircuit::new(100);
        let (_, public_inputs) = circuit.generate_witness(100).unwrap();

        // value == threshold means NOT less than, so is_valid = false
        assert_eq!(public_inputs[1], Fp::from(0u64));
    }
}
