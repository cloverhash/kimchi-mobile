//! Cryptographic gadgets for Kimchi circuits.
//!
//! This module provides building blocks for constructing zero-knowledge proofs
//! that verify cryptographic operations like hashing and signature verification.

pub mod boolean;
pub mod comparison;
pub mod rsa;
pub mod sha256;

pub use boolean::BooleanGadget;
pub use comparison::ComparisonGadget;
pub use rsa::{RsaGadget, RsaWitness, RSA_LIMBS};
pub use sha256::{Sha256Gadget, Sha256Witness};
