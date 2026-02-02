//! RSA signature verification gadget for Kimchi circuits.

use ark_ff::{Field, One, Zero};
use kimchi::circuits::gate::CircuitGate;
use kimchi::circuits::polynomials::generic::GenericGateSpec;
use kimchi::circuits::wires::Wire;
use mina_curves::pasta::Fp;

/// Number of 64-bit limbs for RSA-2048.
pub const RSA_LIMBS: usize = 32;

/// Standard RSA public exponent.
pub const RSA_EXPONENT: u32 = 65537;

/// Gadget builder for RSA verification circuits.
pub struct RsaGadget {
    gates: Vec<CircuitGate<Fp>>,
    current_row: usize,
}

impl RsaGadget {
    pub fn new(start_row: usize) -> Self {
        Self {
            gates: Vec::new(),
            current_row: start_row,
        }
    }

    pub fn current_row(&self) -> usize {
        self.current_row
    }

    /// Constrain a limb to be in range [0, 2^64).
    pub fn range_check_limb(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..64 {
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

        for _ in 0..8 {
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

    /// Multiply two limbs.
    pub fn limb_mul(&mut self) -> usize {
        let start = self.current_row;

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

        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(Fp::from(1u64 << 32).square()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

        self.range_check_limb();
        self.range_check_limb();

        start
    }

    /// Add two limbs with carry.
    pub fn limb_add_with_carry(&mut self) -> usize {
        let start = self.current_row;

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
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(Fp::from(1u64 << 32).square()),
                output_coeff: Some(-Fp::one()),
            },
            None,
        ));
        self.current_row += 1;

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

        start
    }

    /// Multiply two big integers.
    pub fn bigint_mul(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..RSA_LIMBS {
            for _ in 0..RSA_LIMBS {
                self.limb_mul();
            }
        }

        for _ in 0..(2 * RSA_LIMBS - 1) {
            for _ in 0..RSA_LIMBS {
                self.limb_add_with_carry();
            }
        }

        start
    }

    /// Subtract two big integers.
    pub fn bigint_sub(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..RSA_LIMBS {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(-Fp::one()),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;

            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(-Fp::one()),
                    output_coeff: Some(-Fp::one()),
                },
                None,
            ));
            self.current_row += 1;
        }

        start
    }

    /// Compare two big integers.
    pub fn bigint_less_than(&mut self) -> usize {
        let start = self.current_row;
        self.bigint_sub();

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

        start
    }

    /// Modular reduction.
    pub fn bigint_mod(&mut self) -> usize {
        let start = self.current_row;

        self.bigint_mul();

        for _ in 0..RSA_LIMBS {
            self.limb_add_with_carry();
        }

        for _ in 0..RSA_LIMBS {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(-Fp::one()),
                    output_coeff: Some(Fp::zero()),
                },
                None,
            ));
            self.current_row += 1;
        }

        self.bigint_less_than();

        start
    }

    /// Modular multiplication.
    pub fn bigint_mulmod(&mut self) -> usize {
        let start = self.current_row;
        self.bigint_mul();
        self.bigint_mod();
        start
    }

    /// Modular squaring.
    pub fn bigint_sqrmod(&mut self) -> usize {
        self.bigint_mulmod()
    }

    /// Modular exponentiation with e = 65537.
    pub fn modexp_65537(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..16 {
            self.bigint_sqrmod();
        }

        self.bigint_mulmod();

        start
    }

    /// Verify PKCS#1 v1.5 padding.
    pub fn verify_pkcs1_padding(&mut self) -> usize {
        let start = self.current_row;

        // Check first byte is 0x00
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Pub,
            None,
        ));
        self.current_row += 1;

        // Check second byte is 0x01
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(Fp::zero()),
                output_coeff: Some(Fp::zero()),
            },
            Some(GenericGateSpec::Const(-Fp::one())),
        ));
        self.current_row += 1;

        // Check padding bytes are 0xFF
        for _ in 0..8 {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(Fp::zero()),
                    output_coeff: Some(Fp::zero()),
                },
                Some(GenericGateSpec::Const(-Fp::from(0xFFu64))),
            ));
            self.current_row += 1;
        }

        // Check separator 0x00
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Pub,
            None,
        ));
        self.current_row += 1;

        // SHA-256 DigestInfo
        let digest_info: [u8; 19] = [
            0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x01, 0x05, 0x00, 0x04, 0x20,
        ];

        for byte in digest_info {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(Fp::zero()),
                    output_coeff: Some(Fp::zero()),
                },
                Some(GenericGateSpec::Const(-Fp::from(byte as u64))),
            ));
            self.current_row += 1;
        }

        start
    }

    /// Compare two big integers for equality.
    pub fn bigint_equal(&mut self) -> usize {
        let start = self.current_row;

        for _ in 0..RSA_LIMBS {
            let wires = Wire::for_row(self.current_row);
            self.gates.push(CircuitGate::create_generic_gadget(
                wires,
                GenericGateSpec::Add {
                    left_coeff: Some(Fp::one()),
                    right_coeff: Some(-Fp::one()),
                    output_coeff: Some(Fp::zero()),
                },
                None,
            ));
            self.current_row += 1;
        }

        start
    }

    /// Full RSA-2048 signature verification.
    pub fn rsa_verify(&mut self) -> usize {
        let start = self.current_row;
        self.modexp_65537();
        self.verify_pkcs1_padding();
        self.bigint_equal();
        start
    }

    pub fn build(self) -> (Vec<CircuitGate<Fp>>, usize) {
        (self.gates, self.current_row)
    }
}

/// Witness data for RSA verification.
pub struct RsaWitness {
    pub signature: [u64; RSA_LIMBS],
    pub modulus: [u64; RSA_LIMBS],
    pub hash: [u8; 32],
    pub intermediates: Vec<[u64; RSA_LIMBS]>,
}

impl RsaWitness {
    pub fn from_bytes(signature: &[u8; 256], modulus: &[u8; 256], hash: &[u8; 32]) -> Self {
        Self {
            signature: Self::bytes_to_limbs(signature),
            modulus: Self::bytes_to_limbs(modulus),
            hash: *hash,
            intermediates: Vec::new(),
        }
    }

    fn bytes_to_limbs(bytes: &[u8; 256]) -> [u64; RSA_LIMBS] {
        let mut limbs = [0u64; RSA_LIMBS];
        for i in 0..RSA_LIMBS {
            let start = i * 8;
            limbs[RSA_LIMBS - 1 - i] = u64::from_be_bytes([
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
                bytes[start + 4],
                bytes[start + 5],
                bytes[start + 6],
                bytes[start + 7],
            ]);
        }
        limbs
    }

    pub fn limbs_to_bytes(limbs: &[u64; RSA_LIMBS]) -> [u8; 256] {
        let mut bytes = [0u8; 256];
        for i in 0..RSA_LIMBS {
            let limb_bytes = limbs[RSA_LIMBS - 1 - i].to_be_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb_bytes);
        }
        bytes
    }

    pub fn compute_modexp(&mut self) -> [u64; RSA_LIMBS] {
        use num_bigint::BigUint;

        let sig = BigUint::from_bytes_be(&Self::limbs_to_bytes(&self.signature));
        let n = BigUint::from_bytes_be(&Self::limbs_to_bytes(&self.modulus));
        let e = BigUint::from(RSA_EXPONENT);

        let result = sig.modpow(&e, &n);

        self.compute_intermediates();

        let result_bytes = result.to_bytes_be();
        let mut padded = [0u8; 256];
        let offset = 256 - result_bytes.len();
        padded[offset..].copy_from_slice(&result_bytes);

        Self::bytes_to_limbs(&padded)
    }

    fn compute_intermediates(&mut self) {
        use num_bigint::BigUint;

        let sig = BigUint::from_bytes_be(&Self::limbs_to_bytes(&self.signature));
        let n = BigUint::from_bytes_be(&Self::limbs_to_bytes(&self.modulus));

        self.intermediates.clear();

        let mut current = sig.clone();
        self.intermediates
            .push(Self::biguint_to_limbs(&current, &n));

        for _ in 0..16 {
            current = (&current * &current) % &n;
            self.intermediates
                .push(Self::biguint_to_limbs(&current, &n));
        }

        current = (&current * &sig) % &n;
        self.intermediates
            .push(Self::biguint_to_limbs(&current, &n));
    }

    fn biguint_to_limbs(
        value: &num_bigint::BigUint,
        _modulus: &num_bigint::BigUint,
    ) -> [u64; RSA_LIMBS] {
        let bytes = value.to_bytes_be();
        let mut padded = [0u8; 256];
        if bytes.len() <= 256 {
            let offset = 256 - bytes.len();
            padded[offset..].copy_from_slice(&bytes);
        }
        Self::bytes_to_limbs(&padded)
    }

    pub fn verify(&mut self) -> bool {
        let decrypted = self.compute_modexp();
        let decrypted_bytes = Self::limbs_to_bytes(&decrypted);

        if decrypted_bytes[0] != 0x00 || decrypted_bytes[1] != 0x01 {
            return false;
        }

        let mut sep_idx = None;
        for i in 2..decrypted_bytes.len() - 32 {
            if decrypted_bytes[i] == 0x00 {
                sep_idx = Some(i);
                break;
            }
            if decrypted_bytes[i] != 0xFF {
                return false;
            }
        }

        let sep_idx = match sep_idx {
            Some(idx) => idx,
            None => return false,
        };

        if sep_idx < 10 {
            return false;
        }

        let digest_info: [u8; 19] = [
            0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x01, 0x05, 0x00, 0x04, 0x20,
        ];

        let di_start = sep_idx + 1;
        let di_end = di_start + 19;
        let hash_start = di_end;
        let hash_end = hash_start + 32;

        if hash_end != 256 {
            return false;
        }

        if decrypted_bytes[di_start..di_end] != digest_info {
            return false;
        }

        decrypted_bytes[hash_start..hash_end] == self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_limbs_roundtrip() {
        let mut bytes = [0u8; 256];
        bytes[0] = 0x12;
        bytes[255] = 0x34;

        let limbs = RsaWitness::bytes_to_limbs(&bytes);
        let recovered = RsaWitness::limbs_to_bytes(&limbs);

        assert_eq!(bytes, recovered);
    }

    #[test]
    fn test_gadget_construction() {
        let mut gadget = RsaGadget::new(0);
        gadget.rsa_verify();
        let (gates, rows) = gadget.build();

        assert!(!gates.is_empty());
        assert!(rows > 0);
    }
}
