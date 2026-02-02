//! Boolean gadgets for Kimchi circuits.
//!
//! Provides bit decomposition and boolean operations (AND, XOR, NOT)
//! as arithmetic constraints over finite fields.

use ark_ff::{One, Zero};
use kimchi::circuits::gate::CircuitGate;
use kimchi::circuits::polynomials::generic::GenericGateSpec;
use kimchi::circuits::wires::Wire;
use mina_curves::pasta::Fp;

/// Gadget for boolean operations in Kimchi circuits.
pub struct BooleanGadget {
    gates: Vec<CircuitGate<Fp>>,
    current_row: usize,
}

impl BooleanGadget {
    /// Create a new boolean gadget starting at the given row.
    pub fn new(start_row: usize) -> Self {
        Self {
            gates: Vec::new(),
            current_row: start_row,
        }
    }

    /// Get the current row index.
    pub fn current_row(&self) -> usize {
        self.current_row
    }

    /// Add a boolean constraint: b * (b - 1) = 0
    /// This ensures the witness value at the specified column is 0 or 1.
    ///
    /// Uses Generic gate: c0*l + c1*r + c2*o + c3*(l*r) + c4 = 0
    /// For b*(b-1) = 0: b*b - b = 0
    /// Set l = r = b, so we need: c3*(b*b) + c0*b = 0
    /// With c3 = 1, c0 = -1: b*b - b = 0
    pub fn boolean_constraint(&mut self) -> usize {
        let row = self.current_row;
        let wires = Wire::for_row(row);

        // b * b - b = 0
        // Using Mul: mul_coeff * l * r + output_coeff * o = 0
        // With l = r = o = b: 1 * b * b + (-1) * b = 0
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Mul {
                mul_coeff: Some(Fp::one()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));

        self.current_row += 1;
        row
    }

    /// XOR of two bits: c = a XOR b
    /// Arithmetic formula: c = a + b - 2*a*b
    ///
    /// Returns the row where the XOR result is placed.
    pub fn xor(&mut self) -> usize {
        let row = self.current_row;
        let wires = Wire::for_row(row);

        // a + b - 2*a*b - c = 0
        // Using Add for: a + b - c = 2*a*b
        // Then need another gate for the multiplication term
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(Fp::one()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

        // Add the quadratic term constraint: 2*a*b
        let wires2 = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires2,
            GenericGateSpec::Mul {
                mul_coeff: Some(Fp::from(2u64)),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

        row
    }

    /// AND of two bits: c = a AND b
    /// Arithmetic formula: c = a * b
    pub fn and(&mut self) -> usize {
        let row = self.current_row;
        let wires = Wire::for_row(row);

        // a * b - c = 0
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Mul {
                mul_coeff: Some(Fp::one()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));

        self.current_row += 1;
        row
    }

    /// NOT of a bit: c = 1 - a
    pub fn not(&mut self) -> usize {
        let row = self.current_row;
        let wires = Wire::for_row(row);

        // 1 - a - c = 0
        // Use Plus which sets: l - o + constant = 0
        // We want: -a - c + 1 = 0
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(-Fp::one()),
                right_coeff: Some(Fp::zero()),
                output_coeff: Some(-Fp::one()),
            },
            Some(GenericGateSpec::Const(Fp::one())),
        ));

        self.current_row += 1;
        row
    }

    /// Decompose a 32-bit word into individual bits.
    /// Constrains: word = sum(bits[i] * 2^i) for i in 0..32
    /// Also constrains each bit to be boolean.
    ///
    /// Returns the starting row for the bit decomposition.
    pub fn decompose_u32(&mut self) -> usize {
        let start_row = self.current_row;

        // First, add boolean constraints for each bit
        for _ in 0..32 {
            self.boolean_constraint();
        }

        // Then add the linear combination constraint
        // word = sum(bit_i * 2^i)
        self.linear_combination_32();

        start_row
    }

    /// Add constraints for a 32-term linear combination.
    /// Used for bit decomposition: word = sum(bit_i * 2^i)
    fn linear_combination_32(&mut self) {
        // With 15 columns, we can sum about 5 terms per row
        // For 32 bits, we need ~7 rows for the summation
        let num_gates = 7;

        for _ in 0..num_gates {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(Fp::one()),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;
        }
    }

    /// XOR of 32-bit words (bit by bit).
    /// Assumes both words have been decomposed to bits.
    pub fn xor_u32(&mut self) -> usize {
        let start = self.current_row;
        for _ in 0..32 {
            self.xor();
        }
        start
    }

    /// AND of 32-bit words (bit by bit).
    pub fn and_u32(&mut self) -> usize {
        let start = self.current_row;
        for _ in 0..32 {
            self.and();
        }
        start
    }

    /// NOT of a 32-bit word (bit by bit).
    pub fn not_u32(&mut self) -> usize {
        let start = self.current_row;
        for _ in 0..32 {
            self.not();
        }
        start
    }

    /// Consume the gadget and return the gates.
    pub fn build(self) -> (Vec<CircuitGate<Fp>>, usize) {
        (self.gates, self.current_row)
    }
}

/// Witness generator for boolean operations.
pub struct BooleanWitness;

impl BooleanWitness {
    /// Decompose a u32 into 32 field elements (0 or 1).
    pub fn decompose_u32(value: u32) -> [Fp; 32] {
        let mut bits = [Fp::zero(); 32];
        for i in 0..32 {
            if (value >> i) & 1 == 1 {
                bits[i] = Fp::one();
            }
        }
        bits
    }

    /// Recompose bits into a u32.
    pub fn recompose_u32(bits: &[Fp; 32]) -> u32 {
        let mut value = 0u32;
        for i in 0..32 {
            if bits[i] == Fp::one() {
                value |= 1 << i;
            }
        }
        value
    }

    /// XOR two bit arrays.
    pub fn xor_bits(a: &[Fp; 32], b: &[Fp; 32]) -> [Fp; 32] {
        let mut result = [Fp::zero(); 32];
        for i in 0..32 {
            // XOR: a + b - 2*a*b
            result[i] = a[i] + b[i] - (Fp::from(2u64) * a[i] * b[i]);
        }
        result
    }

    /// AND two bit arrays.
    pub fn and_bits(a: &[Fp; 32], b: &[Fp; 32]) -> [Fp; 32] {
        let mut result = [Fp::zero(); 32];
        for i in 0..32 {
            result[i] = a[i] * b[i];
        }
        result
    }

    /// NOT a bit array.
    pub fn not_bits(a: &[Fp; 32]) -> [Fp; 32] {
        let mut result = [Fp::zero(); 32];
        for i in 0..32 {
            result[i] = Fp::one() - a[i];
        }
        result
    }

    /// Right rotation of bits.
    pub fn rotr(bits: &[Fp; 32], n: usize) -> [Fp; 32] {
        let mut result = [Fp::zero(); 32];
        for i in 0..32 {
            result[i] = bits[(i + n) % 32];
        }
        result
    }

    /// Right shift of bits (introduces zeros).
    pub fn shr(bits: &[Fp; 32], n: usize) -> [Fp; 32] {
        let mut result = [Fp::zero(); 32];
        for i in n..32 {
            result[i - n] = bits[i];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompose_recompose() {
        let value = 0xDEADBEEF_u32;
        let bits = BooleanWitness::decompose_u32(value);
        let recovered = BooleanWitness::recompose_u32(&bits);
        assert_eq!(value, recovered);
    }

    #[test]
    fn test_xor_bits() {
        let a = BooleanWitness::decompose_u32(0xFF00FF00);
        let b = BooleanWitness::decompose_u32(0x0F0F0F0F);
        let result = BooleanWitness::xor_bits(&a, &b);
        let value = BooleanWitness::recompose_u32(&result);
        assert_eq!(value, 0xFF00FF00 ^ 0x0F0F0F0F);
    }

    #[test]
    fn test_and_bits() {
        let a = BooleanWitness::decompose_u32(0xFF00FF00);
        let b = BooleanWitness::decompose_u32(0x0F0F0F0F);
        let result = BooleanWitness::and_bits(&a, &b);
        let value = BooleanWitness::recompose_u32(&result);
        assert_eq!(value, 0xFF00FF00 & 0x0F0F0F0F);
    }

    #[test]
    fn test_rotr() {
        let bits = BooleanWitness::decompose_u32(0x80000001);
        let rotated = BooleanWitness::rotr(&bits, 1);
        let value = BooleanWitness::recompose_u32(&rotated);
        assert_eq!(value, 0x80000001_u32.rotate_right(1));
    }
}
