//! Comparison gadgets for Kimchi circuits.

use ark_ff::{One, Zero};
use kimchi::circuits::gate::CircuitGate;
use kimchi::circuits::polynomials::generic::GenericGateSpec;
use kimchi::circuits::wires::Wire;
use mina_curves::pasta::Fp;

/// Gadget for comparison operations.
pub struct ComparisonGadget {
    gates: Vec<CircuitGate<Fp>>,
    current_row: usize,
}

impl ComparisonGadget {
    pub fn new(start_row: usize) -> Self {
        Self {
            gates: Vec::new(),
            current_row: start_row,
        }
    }

    pub fn current_row(&self) -> usize {
        self.current_row
    }

    /// Equality constraint: a == b.
    pub fn equal(&mut self) -> usize {
        let row = self.current_row;
        let wires = Wire::for_row(row);

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
        row
    }

    /// Greater than or equal constraint: a >= b.
    pub fn greater_or_equal(&mut self, max_bits: usize) -> usize {
        let start = self.current_row;

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

        self.range_check(max_bits);

        start
    }

    /// Range check: 0 <= value < 2^num_bits.
    pub fn range_check(&mut self, num_bits: usize) -> usize {
        let start = self.current_row;

        for _ in 0..num_bits {
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

        let num_sum_gates = (num_bits + 2) / 3;
        for _ in 0..num_sum_gates {
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

    /// Date comparison for age verification.
    pub fn age_check(&mut self, minimum_age: u32) -> usize {
        let start = self.current_row;

        // Subtract years
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

        // Month comparison
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

        // Day comparison
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

        // Adjustment is boolean
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

        // Final age = base_age - adjustment
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

        // Check age >= minimum_age
        let wires = Wire::for_row(self.current_row);
        self.gates.push(CircuitGate::create_generic_gadget(
            wires,
            GenericGateSpec::Add {
                left_coeff: Some(Fp::one()),
                right_coeff: Some(Fp::zero()),
                output_coeff: Some(-Fp::one()),
            },
            Some(GenericGateSpec::Const(-Fp::from(minimum_age as u64))),
        ));
        self.current_row += 1;

        // Range check the difference (7 bits for age up to 127)
        self.range_check(7);

        start
    }

    pub fn build(self) -> (Vec<CircuitGate<Fp>>, usize) {
        (self.gates, self.current_row)
    }
}

/// Witness generator for comparisons.
pub struct ComparisonWitness;

impl ComparisonWitness {
    pub fn compute_age(
        birth_year: u32,
        birth_month: u32,
        birth_day: u32,
        current_year: u32,
        current_month: u32,
        current_day: u32,
    ) -> u32 {
        let mut age = current_year - birth_year;

        if current_month < birth_month || (current_month == birth_month && current_day < birth_day)
        {
            age -= 1;
        }

        age
    }

    pub fn parse_mrz_date(date_str: &str) -> Option<(u32, u32, u32)> {
        if date_str.len() != 6 {
            return None;
        }

        let yy: u32 = date_str[0..2].parse().ok()?;
        let mm: u32 = date_str[2..4].parse().ok()?;
        let dd: u32 = date_str[4..6].parse().ok()?;

        let year = if yy <= 29 { 2000 + yy } else { 1900 + yy };

        Some((year, mm, dd))
    }

    pub fn decompose_for_range_check(value: u64, num_bits: usize) -> Vec<Fp> {
        let mut bits = Vec::with_capacity(num_bits);
        for i in 0..num_bits {
            if (value >> i) & 1 == 1 {
                bits.push(Fp::one());
            } else {
                bits.push(Fp::zero());
            }
        }
        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_age() {
        assert_eq!(ComparisonWitness::compute_age(1990, 1, 15, 2024, 2, 1), 34);
        assert_eq!(ComparisonWitness::compute_age(1990, 1, 15, 2024, 1, 14), 33);
        assert_eq!(ComparisonWitness::compute_age(1990, 1, 15, 2024, 1, 15), 34);
    }

    #[test]
    fn test_parse_mrz_date() {
        assert_eq!(
            ComparisonWitness::parse_mrz_date("900115"),
            Some((1990, 1, 15))
        );
        assert_eq!(
            ComparisonWitness::parse_mrz_date("050620"),
            Some((2005, 6, 20))
        );
    }

    #[test]
    fn test_gadget_construction() {
        let mut gadget = ComparisonGadget::new(0);
        gadget.age_check(18);
        let (gates, rows) = gadget.build();

        assert!(!gates.is_empty());
        println!("Age check gates: {}, rows: {}", gates.len(), rows);
    }
}
