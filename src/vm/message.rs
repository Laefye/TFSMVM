use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{block::{AsBlock, Block}, builder::Builder, slice::Slice};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    Internal,
    External,
    View,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Init {
    pub program: Block,
    pub data: Block,
}

impl Init {
    pub fn from_block(block: Block) -> Option<Self> {
        let mut slice = Slice::new(block);
        let program = slice.read_block_with_len()?;
        let data = slice.read_block_with_len()?;
        Some(Init { program, data })
    }
}

impl AsBlock for Init {
    fn get_as_block(&self) -> Block {
        let mut builder = Builder::new();
        builder.write_block_with_len(self.program.clone());
        builder.write_block_with_len(self.data.clone());
        builder.build()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub sender: Block,
    pub receiver: Block,
    pub init: Option<Init>,
    pub opcode: u64,
    pub body: Block,
    pub timestamp: u64,
}

impl Message {
    pub fn new(message_type: MessageType, body: Block, opcode: u64, sender: Block, receiver: Block, init: Option<Init>) -> Self {
        Self {
            message_type,
            body,
            sender,
            receiver,
            opcode,
            init,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        }
    }

    pub fn from_block(block: Block) -> Option<Message> {
        let mut slice = Slice::new(block);
        let message_type = match slice.read_u8()? {
            0 => MessageType::External,
            1 => MessageType::Internal,
            2 => MessageType::View,
            _ => return None,
        };
        let sender = slice.read_block_with_len()?;
        let receiver = slice.read_block_with_len()?;
        let init = Init::from_block(slice.read_block_with_len()?);
        let opcode = slice.read_u64()?;
        let body = slice.read_block_with_len()?;
        let timestamp = slice.read_u64()?;
        Some(Message { message_type, sender, receiver, init, opcode, body, timestamp })
    }
}

impl AsBlock for Message {
    fn get_as_block(&self) -> Block {
        let mut builder = Builder::new();
        builder.write_u8(match self.message_type {
            MessageType::Internal => 1,
            MessageType::External => 0,
            MessageType::View => 2,
        });
        builder.write_block_with_len(self.sender.clone());
        builder.write_block_with_len(self.receiver.clone());
        builder.write_block_with_len(match self.init.clone() {
            Some(init) => init.get_as_block(),
            None => Block::empty(),
        });
        builder.write_u64(self.opcode);
        builder.write_block_with_len(self.body.clone());
        builder.write_u64(self.timestamp);
        builder.build()
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{} {} -> {} #{} = {} (initial?: {})", self.get_as_block().hash().to_string(), self.sender.to_string(), self.receiver.to_string(), self.opcode, self.body.to_string(), self.init.is_some())
    }
}
