use super::{block::Block, utils::{get_u64, get_u8}};

#[derive(Clone)]
pub struct Slice {
    code: Vec<u8>,
    pointer: usize,
}

impl Slice {
    pub fn new(block: Block) -> Self {
        Self {
            code: block.unpack(),
            pointer: 0,
        }
    }

    fn get(&mut self, length: usize) -> Option<&[u8]> {
        let slice = self.code.get(self.pointer .. (self.pointer + length))?;
        self.pointer += length;
        Some(slice)
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        get_u64(self.get(size_of::<u64>())?)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        get_u8(self.get(size_of::<u8>())?)
    }

    pub fn read_block(&mut self, length: usize) -> Option<Block> {
        Some(Block::new(self.get(length)?))
    }

    pub fn read_block_with_len(&mut self) -> Option<Block> {
        let size = self.read_u64()? as usize;
        self.read_block(size)
    }

    pub fn len(&self) -> usize {
        return self.code.len() - self.pointer
    }
}

