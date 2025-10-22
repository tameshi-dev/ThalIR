use crate::values::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoOp {
    Keccak256 {
        data: Value,
        offset: Value,
        length: Value,
    },

    Sha256 {
        data: Value,
        offset: Value,
        length: Value,
    },

    Ripemd160 {
        data: Value,
        offset: Value,
        length: Value,
    },

    Blake2 {
        data: Value,
        offset: Value,
        length: Value,
        rounds: Option<Value>,
    },

    EcRecover {
        hash: Value,
        v: Value,
        r: Value,
        s: Value,
    },

    EcVerify {
        message: Value,
        signature: Value,
        public_key: Value,
    },

    ModExp {
        base: Value,
        exponent: Value,
        modulus: Value,
    },

    Bn256Add {
        x1: Value,
        y1: Value,
        x2: Value,
        y2: Value,
    },

    Bn256Mul {
        x: Value,
        y: Value,
        scalar: Value,
    },

    Bn256Pairing {
        points: Vec<(Value, Value)>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precompile {
    EcRecover = 0x01,
    Sha256 = 0x02,
    Ripemd160 = 0x03,
    Identity = 0x04,
    ModExp = 0x05,
    Bn256Add = 0x06,
    Bn256Mul = 0x07,
    Bn256Pairing = 0x08,
    Blake2 = 0x09,
}

impl Precompile {
    pub fn address(&self) -> [u8; 20] {
        let mut addr = [0u8; 20];
        addr[19] = *self as u8;
        addr
    }

    pub fn from_address(addr: &[u8; 20]) -> Option<Self> {
        if addr[0..19] != [0u8; 19] {
            return None;
        }

        match addr[19] {
            0x01 => Some(Precompile::EcRecover),
            0x02 => Some(Precompile::Sha256),
            0x03 => Some(Precompile::Ripemd160),
            0x04 => Some(Precompile::Identity),
            0x05 => Some(Precompile::ModExp),
            0x06 => Some(Precompile::Bn256Add),
            0x07 => Some(Precompile::Bn256Mul),
            0x08 => Some(Precompile::Bn256Pairing),
            0x09 => Some(Precompile::Blake2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub x: [u8; 32],
    pub y: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashType {
    Keccak256,
    Sha256,
    Ripemd160,
    Blake2,
}

impl HashType {
    pub fn output_size(&self) -> usize {
        match self {
            HashType::Keccak256 | HashType::Sha256 | HashType::Blake2 => 32,
            HashType::Ripemd160 => 20,
        }
    }
}
