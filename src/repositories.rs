use std::{cell::RefCell, rc::Rc};

use polodb_core::{bson::doc, CollectionT, Database};
use serde::{Deserialize, Serialize};

use crate::vm::{block::{AsBlock, Block}, env::{Repository, TransactionPart}, message::{Init, Message, MessageType}};

pub struct PolaDBRef {
    poladb: Database,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerdeContract {
    pub address: String,
    pub program: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerdeContractState {
    pub address: String,
    pub data: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerdeInit {
    pub program: String,
    pub data: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerdeMessage {
    pub id: String,
    pub message_type: String,
    pub sender: String,
    pub receiver: String,
    pub init: Option<SerdeInit>,
    pub opcode: u64,
    pub body: String,
    pub timestamp: u64,
}

impl SerdeMessage {
    pub fn get_message_type(message: &Message) -> String {
        match message.message_type {
            crate::vm::message::MessageType::Internal => "internal",
            crate::vm::message::MessageType::External => "external",
            crate::vm::message::MessageType::View => "view",
        }.to_string()
    }
    
    pub fn get_message_type_inverse(&self) -> Option<MessageType> {
        Some(if self.message_type == "internal".to_string() {
            MessageType::Internal
        } else if self.message_type == "external".to_string() {
            MessageType::External
        } else if self.message_type == "view".to_string() {
            MessageType::View
        } else {
            todo!()
        })
    }

    pub fn to_message(&self) -> Option<Message> {
        Some(
            Message {
                message_type: self.get_message_type_inverse()?,
                sender: Block::from_string(self.sender.clone())?,
                receiver: Block::from_string(self.receiver.clone())?,
                init: match &self.init {
                    Some(init) => Some(Init {
                        program: Block::from_string(init.program.clone())?,
                        data: Block::from_string(init.data.clone())?,
                    }),
                    None => None,
                },
                opcode: self.opcode,
                body: Block::from_string(self.body.clone())?,
                timestamp: self.timestamp,
            }
        )
    }

    pub fn from_message(message: &Message) -> SerdeMessage {
        SerdeMessage {
            id: message.get_as_block().hash().to_string(),
            message_type: Self::get_message_type(&message),
            sender: message.sender.to_string(),
            receiver: message.receiver.to_string(),
            init: message.init.clone().map(|x| SerdeInit { program: x.program.to_string(), data: x.data.to_string() }),
            opcode: message.opcode,
            body: message.body.to_string(),
            timestamp: message.timestamp,
        }
    }
}

impl Repository for PolaDBRef {
    fn get_contract_program(&self, address: Block) -> Option<Block> {
        let contracts = self.poladb.collection::<SerdeContract>("contracts");
        let contract = contracts.find(doc! { "address": address.to_string()}).run().unwrap()
            .map(|x| x.unwrap()).max_by_key(|x| x.timestamp);
        if let Some(contract) = contract  {
            Some(Block::from_string(contract.program)?)
        } else {
            None
        }
    }

    fn save_transaction(&mut self, transaction: TransactionPart) {
        match transaction {
            TransactionPart::Message(message) => {
                let message = SerdeMessage::from_message(&message);
                self.poladb.collection::<SerdeMessage>("messages").insert_one(&message).unwrap();
            },
            TransactionPart::State(contract_state) => {
                let message = contract_state.message.clone();
                if let Some(init) = contract_state.message.init {
                    let contract = SerdeContract {
                        address: contract_state.message.receiver.to_string(),
                        program: init.program.to_string(),
                        timestamp: contract_state.message.timestamp,
                    };
                    self.poladb.collection::<SerdeContract>("contracts").insert_one(&contract).unwrap();
                } 
                let serde_state = SerdeContractState {
                    address: contract_state.message.receiver.to_string(),
                    data: contract_state.data.to_string(),
                    timestamp: contract_state.message.timestamp,
                };
                self.poladb.collection::<SerdeContractState>("contract_states").insert_one(&serde_state).unwrap();
                let message = SerdeMessage::from_message(&message);
                self.poladb.collection::<SerdeMessage>("messages").insert_one(&message).unwrap();
                for child in contract_state.children {
                    self.save_transaction(child);
                }
            },
        }
    }
    
    fn get_all_messages(&self, limit: u64, offset: u64) -> Vec<Message> {
        let mut messages: Vec<SerdeMessage> = self.poladb.collection::<SerdeMessage>("messages").find(doc! {}).run().unwrap().map(|x| x.unwrap()).collect();
        messages.sort_by_key(|x| x.timestamp);
        messages.reverse();
        let messages = messages.iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|x| x).map(|x| x.to_message().unwrap());
        messages.collect()
    }
    
    fn get_messages_by_contract(&self, address: Block, limit: u64, offset: u64) -> Vec<Message> {
        let mut messages: Vec<SerdeMessage> = self.poladb.collection::<SerdeMessage>("messages").find(doc! {}).run().unwrap().map(|x| x.unwrap()).collect();
        messages.sort_by_key(|x| x.timestamp);
        messages.reverse();
        let messages = messages.iter()
            .filter(|x| x.sender == address.to_string() || x.receiver == address.to_string())
            .skip(offset as usize)
            .take(limit as usize)
            .map(|x| x).map(|x| x.to_message().unwrap());
        messages.collect()
    }
    
    fn get_contract_data(&self, address: Block) -> Option<Block> {
        let contracts = self.poladb.collection::<SerdeContractState>("contract_states");
        let contract = contracts.find(doc! { "address": address.to_string()}).run().unwrap()
            .map(|x| x.unwrap()).max_by_key(|x| x.timestamp);
        if let Some(contract) = contract  {
            println!("{} {}", address.to_string(), contract.clone().data);
            Some(Block::from_string(contract.data)?)
        } else {
            None
        }
    }

}

impl PolaDBRef {
    pub fn new() -> Self {
        Self {
            poladb: Database::open_path("tfsm_instance").unwrap(),
        }
    }
}

pub struct PolaDBRepository {
    poladb: Rc<RefCell<PolaDBRef>>,
}

impl PolaDBRepository {
    pub fn new() -> Self {
        Self {
            poladb: Rc::new(RefCell::new(PolaDBRef::new())),
        }
    }

    pub fn get_ref(&self) -> Rc<RefCell<PolaDBRef>> {
        self.poladb.clone()
    }
}
