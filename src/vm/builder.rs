use super::block::Block;

#[derive(Clone)]
pub struct Builder {
    bytes: Vec<u8>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        self.bytes.extend(value.to_be_bytes());
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.extend(value.to_be_bytes());
    }

    pub fn write_block(&mut self, value: Block) {
        self.bytes.extend(value.unpack());
    }

    pub fn write_block_with_len(&mut self, value: Block) {
        self.write_u64(value.len() as u64);
        self.write_block(value);
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn build(&self) -> Block {
        Block::new(&self.bytes)
    }
}

