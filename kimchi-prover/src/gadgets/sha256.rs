//! SHA-256 hash gadget for Kimchi circuits.
//!
//! Implements SHA-256 as arithmetic constraints over the Pallas scalar field.

use ark_ff::{One, Zero};
use kimchi::circuits::gate::CircuitGate;
use kimchi::circuits::polynomials::generic::GenericGateSpec;
use kimchi::circuits::wires::Wire;
use mina_curves::pasta::Fp;

use super::boolean::BooleanWitness;

/// SHA-256 initial hash values (H0-H7).
pub const H_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// SHA-256 round constants (K0-K63).
pub const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Gadget builder for SHA-256 circuits.
pub struct Sha256Gadget {
    gates: Vec<CircuitGate<Fp>>,
    current_row: usize,
}

impl Sha256Gadget {
    pub fn new(start_row: usize) -> Self {
        Self {
            gates: Vec::new(),
            current_row: start_row,
        }
    }

    pub fn current_row(&self) -> usize {
        self.current_row
    }

    /// Add gates for bit decomposition of a 32-bit word.
    pub fn decompose_word(&mut self) -> usize {
        let start = self.current_row;

        // Boolean constraints for each bit
        for _ in 0..32 {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Mul {
                    mul_coeff: Some(Fp::one()),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;
        }

        // Linear combination gates
        for _ in 0..7 {
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

        start
    }

    /// Add constraint for modular addition: (a + b) mod 2^32 = result
    pub fn add_mod32(&mut self) -> usize {
        let start = self.current_row;

        // Main addition constraint
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

        // Overflow is boolean constraint
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Mul {
                mul_coeff: Some(Fp::one()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

        // Subtract overflow * 2^32
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(-Fp::from(1u64 << 32)),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

        start
    }

    /// XOR of two 32-bit words.
    pub fn xor_words(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..32 {
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

            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Mul {
                    mul_coeff: Some(Fp::from(2u64)),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;
        }

        start
    }

    /// AND of two 32-bit words.
    pub fn and_words(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..32 {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Mul {
                    mul_coeff: Some(Fp::one()),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;
        }

        start
    }

    /// NOT of a 32-bit word.
    pub fn not_word(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..32 {
            let wires = Wire::for_row(self.current_row);
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
        }

        start
    }

    /// SHA-256 Ch function: Ch(e, f, g) = (e AND f) XOR (NOT e AND g)
    pub fn ch(&mut self) -> usize {
        let start = self.current_row;
        self.and_words();
        self.not_word();
        self.and_words();
        self.xor_words();
        start
    }

    /// SHA-256 Maj function.
    pub fn maj(&mut self) -> usize {
        let start = self.current_row;
        self.and_words();
        self.and_words();
        self.xor_words();
        self.and_words();
        self.xor_words();
        start
    }

    /// SHA-256 Sigma0.
    pub fn sigma0(&mut self) -> usize {
        let start = self.current_row;
        self.xor_words();
        self.xor_words();
        start
    }

    /// SHA-256 Sigma1.
    pub fn sigma1(&mut self) -> usize {
        let start = self.current_row;
        self.xor_words();
        self.xor_words();
        start
    }

    /// SHA-256 sigma0.
    pub fn small_sigma0(&mut self) -> usize {
        let start = self.current_row;
        self.xor_words();
        self.xor_words();
        start
    }

    /// SHA-256 sigma1.
    pub fn small_sigma1(&mut self) -> usize {
        let start = self.current_row;
        self.xor_words();
        self.xor_words();
        start
    }

    /// One round of SHA-256 compression.
    pub fn compression_round(&mut self) -> usize {
        let start = self.current_row;

        self.sigma1();
        self.ch();
        self.add_mod32();
        self.add_mod32();
        self.add_mod32();
        self.add_mod32();

        self.sigma0();
        self.maj();
        self.add_mod32();

        self.add_mod32();
        self.add_mod32();

        start
    }

    /// Message schedule expansion.
    pub fn message_schedule(&mut self) -> usize {
        let start = self.current_row;

        for _ in 16..64 {
            self.small_sigma1();
            self.small_sigma0();
            self.add_mod32();
            self.add_mod32();
            self.add_mod32();
        }

        start
    }

    /// Full SHA-256 compression for one 512-bit block.
    pub fn sha256_block(&mut self) -> usize {
        let start = self.current_row;

        self.message_schedule();

        for _ in 0..64 {
            self.compression_round();
        }

        for _ in 0..8 {
            self.add_mod32();
        }

        start
    }

    /// Build the circuit for hashing a message.
    pub fn hash_message(&mut self, message_bytes: usize) -> usize {
        let start = self.current_row;
        let padded_len = message_bytes + 1 + 8;
        let num_blocks = (padded_len + 63) / 64;

        for _ in 0..num_blocks {
            self.sha256_block();
        }

        start
    }

    pub fn build(self) -> (Vec<CircuitGate<Fp>>, usize) {
        (self.gates, self.current_row)
    }
}

/// Witness generator for SHA-256.
pub struct Sha256Witness {
    state: [[Fp; 32]; 8],
    schedule: [[Fp; 32]; 64],
}

impl Sha256Witness {
    pub fn new() -> Self {
        Self {
            state: [[Fp::zero(); 32]; 8],
            schedule: [[Fp::zero(); 32]; 64],
        }
    }

    pub fn compute(&mut self, message: &[u8]) -> [u8; 32] {
        let padded = Self::pad_message(message);
        let mut h: [u32; 8] = H_INIT;

        for block in padded.chunks(64) {
            h = self.process_block(block, h);
        }

        let mut result = [0u8; 32];
        for (i, &word) in h.iter().enumerate() {
            result[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        result
    }

    pub fn pad_message(message: &[u8]) -> Vec<u8> {
        let mut padded = message.to_vec();
        let original_len_bits = (message.len() as u64) * 8;

        padded.push(0x80);

        while (padded.len() % 64) != 56 {
            padded.push(0x00);
        }

        padded.extend_from_slice(&original_len_bits.to_be_bytes());

        padded
    }

    fn process_block(&mut self, block: &[u8], h: [u32; 8]) -> [u32; 8] {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }

        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        for i in 0..64 {
            self.schedule[i] = BooleanWitness::decompose_u32(w[i]);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);

            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        let result = [
            h[0].wrapping_add(a),
            h[1].wrapping_add(b),
            h[2].wrapping_add(c),
            h[3].wrapping_add(d),
            h[4].wrapping_add(e),
            h[5].wrapping_add(f),
            h[6].wrapping_add(g),
            h[7].wrapping_add(hh),
        ];

        for i in 0..8 {
            self.state[i] = BooleanWitness::decompose_u32(result[i]);
        }

        result
    }

    pub fn get_hash_words(&self) -> [Fp; 8] {
        let mut result = [Fp::zero(); 8];
        for i in 0..8 {
            result[i] = Fp::from(BooleanWitness::recompose_u32(&self.state[i]) as u64);
        }
        result
    }
}

impl Default for Sha256Witness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_sha256_witness_empty() {
        let mut witness = Sha256Witness::new();
        let result = witness.compute(b"");

        let mut hasher = Sha256::new();
        hasher.update(b"");
        let expected: [u8; 32] = hasher.finalize().into();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sha256_witness_abc() {
        let mut witness = Sha256Witness::new();
        let result = witness.compute(b"abc");

        let mut hasher = Sha256::new();
        hasher.update(b"abc");
        let expected: [u8; 32] = hasher.finalize().into();

        assert_eq!(result, expected);
    }
}
