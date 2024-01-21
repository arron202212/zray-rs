// from abc import ABCMeta, abstractmethod
// from typing import List

use crate::compiler::privacy::circuit_generation::circuit_helper::CircuitHelper;
#[derive(Clone)]
pub struct G1Point {
    x: String,
    y: String,
}
// class G1Point
// """Data class to represent curve points"""
impl G1Point {
    pub fn new(x: String, y: String) -> Self
// """Construct G1Point from coordinate integer literal strings."""
    // self.x: String = x
    // self.y: String = y
    {
        Self { x, y }
    }
    pub fn default() -> Self
// """Construct G1Point from coordinate integer literal strings."""
    // self.x: String = x
    // self.y: String = y
    {
        let zero = String::from("0");
        Self { x: zero, y: zero }
    }
    pub fn negated(&self) {
        let q = "21888242871839275222246405745257275088696311157297823662689037894645226208583";
        if self.x == "0" && self.y == "0" {
            G1Point::default()
        } else {
            G1Point::new(self.x, self.y) // hex(q - (int(self.y, 0) % q)) TODO
        }
    }

    // @staticmethod
    pub fn from_seq(seq: Vec<String>) -> Self
// """
        // Construct G1Point from a sequence of length 2 of integer literal strings
        // First entry makes up the X coordinate, second entry makes up the Y coordinate
        // """
    {
        assert!(seq.len() == 2);
        return G1Point::new(seq[0], seq[1]);
    }

    // @staticmethod
    pub fn from_it<T>(it: &T) -> Self {
        G1Point::new(it.next().unwrap().unwrap(), it.next().unwrap().unwrap())
    }

    // pub fn __str__(G1Point)
    //     return f"uint256({self.x}), uint256({self.y})"
}
use std::fmt;

impl fmt::Display for G1Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "uint256({}), uint256({})", self.x, self.y)
    }
}

// class G2Point
// """Data class to represent curve points which are encoded using two field elements"""
pub struct G2Point {
    x: G1Point,
    y: G1Point,
}
impl G2Point {
    pub fn new(x1: String, x2: String, y1: String, y2: String) -> Self {
        Self {
            x: G1Point::new(x1, x2), // not really a G1Point, but can reuse __str__
            y: G1Point::new(y1, y2),
        }
    }

    // @staticmethod
    pub fn from_seq(seq: Vec<String>) -> Self
// """
        // Construct G1Point from a sequence of length 4 of integer literal strings
        // First two entries make up the X coordinate, last two entries make up the Y coordinate
        // """
        //
    {
        assert!(seq.len() == 4);
        G2Point::new(seq[0], seq[1], seq[2], seq[3])
    }

    // @staticmethod
    pub fn from_it<T>(it: &T) -> Self {
        G2Point::new(
            it.next().unwrap().unwrap(),
            it.next().unwrap().unwrap(),
            it.next().unwrap().unwrap(),
            it.next().unwrap().unwrap(),
        )
    }

    // pub fn __str__(self)
    //     return f"[{self.x}], [{self.y}]"
}
impl fmt::Display for G2Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}], [{}]", self.x, self.y)
    }
}
// class VerifyingKey(metaclass=ABCMeta)
// """Abstract base data class for verification keys"""
pub trait VerifyingKeyMeta {
    type Output;
    // @classmethod
    // @abstractmethod
    // pub fn create_dummy_key(cls)
    //     """Generate a dummy key."""
    fn create_dummy_key() -> Self::Output
    where
        Self: Sized;
    //     pass
}

// class ProvingScheme(metaclass=ABCMeta)
// """
// Abstract base class for proving schemes

// A proving scheme provides functionality to generate a verification contract from a proving-scheme dependent verification-key
// and an abstract circuit representation
// """
pub struct ProvingSchemeBase {
    // verify_libs_contract_filename = "./verify_libs.sol"
    // snark_scalar_field_var_name = "snark_scalar_field"
    // hash_var_name = "hash"
    // """Special variable names usable by the verification contract"""

    // name = "none"
    // """Proving scheme name, overridden by child classes"""
    verify_libs_contract_filename: String,
    snark_scalar_field_var_name: String,
    hash_var_name: String,
    name: String,
}
impl ProvingSchemeBase {
    pub fn new() -> Self {
        Self {
            verify_libs_contract_filename: String::from("./verify_libs.sol"),
            snark_scalar_field_var_name: String::from("snark_scalar_field"),
            hash_var_name: String::from("hash"),
            name: String::from("none"),
        }
    }
}
// class VerifyingKey(VerifyingKey, metaclass=ABCMeta)
//     pass

pub trait ProvingScheme {
    const NAME: &'static str;
    type VerifyingKey;
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    fn hash_var_name(&self) -> String {
        String::new()
    }
    // @abstractmethod
    fn generate_verification_contract<
        V: Clone
            + std::marker::Sync
            + crate::zkay_ast::visitor::transformer_visitor::AstTransformerVisitor,
        VK,
    >(
        &self,
        verification_key: VK,
        circuit: &CircuitHelper<V>,
        primary_inputs: Vec<String>,
        prover_key_hash: Vec<u8>,
    ) -> String;
    // """
    // Generate a verification contract for the zk-snark corresponding to circuit.

    // :param verification_key: parsed verification key which was previously generated for circuit
    // :param circuit: the circuit for which to generate the verification contract
    // :param primary_inputs: list of all public input locations (strings which represent either identifiers or array index expressions)
    // :param prover_key_hash: sha3 hash of the prover key
    // :return: verification contract text
    // """
    // pass
}
