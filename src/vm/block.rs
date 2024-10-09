use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    buffer: Vec<u8>
} 

impl Block {
    pub fn unpack(self) -> Vec<u8> {
        self.buffer
    }

    pub fn new(bytes: &[u8]) -> Self {
        Self {
            buffer: Vec::from(bytes),
        }
    }

    pub fn empty() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn from_string(value: String) -> Option<Block> {
        Some(Self::new(&hex::decode(value).ok()?))
    }

    pub fn hash(&self) -> Block {
        let mut hasher = Sha256::new();
        hasher.update(&self.buffer);
        let result = hasher.finalize();
        Self::new(&result)
    }
}

impl From<Vec<u8>> for Block {
    fn from(value: Vec<u8>) -> Self {
        Self {
            buffer: value
        }
    }
}

impl From<&[u8]> for Block {
    fn from(value: &[u8]) -> Self {
        Self {
            buffer: Vec::from(value)
        }
    }
}

impl ToString for Block {
    fn to_string(&self) -> String {
        hex::encode(&self.buffer)
    }
}

pub trait AsBlock {
    fn get_as_block(&self) -> Block;
}
