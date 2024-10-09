use std::{cell::RefCell, rc::Rc};

use crate::program::ProgramReaderFromBytes;

use super::{block::{AsBlock, Block}, builder::Builder, message::{Init, Message}, SendMessage, Value, VM};

pub trait Repository {
    fn get_contract_program(&self, address: Block) -> Option<Block>;
    fn get_contract_data(&self, address: Block) -> Option<Block>;
    fn save_transaction(&mut self, transaction: TransactionPart);
    fn get_all_messages(&self, limit: u64, offset: u64) -> Vec<Message>;
    fn get_messages_by_contract(&self, address: Block, limit: u64, offset: u64) -> Vec<Message>;
}

#[derive(Clone)]
pub struct Environment {
    message: Message,
    order: Vec<Message>,
    repository: Rc<RefCell<dyn Repository>>
}

#[derive(Clone)]
pub struct ContractState {
    pub message: Message,
    pub data: Block,
    pub children: Vec<TransactionPart>,
}

impl AsBlock for ContractState {
    fn get_as_block(&self) -> Block {
        let mut builder = Builder::new();
        builder.write_block_with_len(self.message.get_as_block());
        builder.write_block_with_len(self.data.clone());
        builder.write_u64(self.children.len() as u64);
        for child in self.children.clone() {
            builder.write_block_with_len(child.get_as_block());
        }
        builder.build()
    }
}

#[derive(Clone)]
pub enum TransactionPart {
    Message(Message),
    State(ContractState),
}

impl AsBlock for TransactionPart {
    fn get_as_block(&self) -> Block {
        match self {
            TransactionPart::Message(message) => {
                let mut builder = Builder::new();
                builder.write_u8(0);
                builder.write_block_with_len(message.get_as_block());
                builder.build()
            },
            TransactionPart::State(contract_state) => {
                let mut builder = Builder::new();
                builder.write_u8(1);
                builder.write_block_with_len(contract_state.get_as_block());
                builder.build()
            },
        }
    }
}

impl Environment {
    fn new(message: Message, repository: Rc<RefCell<dyn Repository>>) -> Self {
        Self {
            message,
            order: Vec::new(),
            repository: repository,
        }
    }

    fn get_vm(&mut self) -> Option<VM> {
        let init;
        if self.message.init.is_some() {
            init = self.message.init.clone();
            if init.clone().unwrap().get_as_block().hash().unpack() != self.message.clone().receiver.unpack() {
                return None;
            }
        } else {
            let repository = self.repository.borrow();
            let address = self.message.receiver.clone();
            init = Some(Init { program: repository.get_contract_program(address.clone())?, data: repository.get_contract_data(address.clone())? });
        }
        if let Some(init) = init {
            let program = ProgramReaderFromBytes::new(&init.clone().program.unpack()).load()?;
            let entrypoint = program.get_entrypoint(self.message.message_type)?;
            return Some(VM::new(program.get_code(), entrypoint, init.data, self.message.clone(), self))
        }
        None
    }

    fn run(&mut self) -> Option<Block> {
        let mut vm = self.get_vm()?;
        vm.run();
        Some(vm.get_data())
    }

    fn run_view(&mut self) -> Option<Vec<Value>> {
        let mut vm = self.get_vm()?;
        vm.run();
        Some(vm.stack())
    }

    fn execute(message: Message, repository: Rc<RefCell<dyn Repository>>) -> TransactionPart {
        let mut env = Self::new(message.clone(), repository.clone());
        let data = env.run();
        if let Some(data) = data {
            return TransactionPart::State(ContractState { 
                message,
                data: data,
                children: env.order.iter().map(|x| Self::execute(x.clone(), repository.clone())).collect()
            });
        } else {
            return TransactionPart::Message(message);
        }
    }

    pub fn view(message: Message, repository: Rc<RefCell<dyn Repository>>) -> Vec<Value> {
        let mut env = Self::new(message.clone(), repository.clone());
        env.run_view().or(Some(Vec::new())).unwrap()
    }

    pub fn start_transaction(message: Message, repository: Rc<RefCell<dyn Repository>>) -> TransactionPart {
        let transaction = Self::execute(message, repository.clone());
        repository.borrow_mut().save_transaction(transaction.clone());
        transaction
    }
}

impl SendMessage for Environment {
    fn send_message(&mut self, message: Message) {
        self.order.push(message);
    }
}
