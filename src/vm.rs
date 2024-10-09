use std::usize;

use block::{AsBlock, Block};
use builder::Builder;
use message::{Init, Message};
use slice::Slice;
use stack::Stack;
use utils::{cond_sign, get_relative_reference, get_u16, get_u64, get_u8, operate};

pub mod block;
pub mod instructions;
pub mod stack;
pub mod slice;
pub mod builder;
pub mod utils;
pub mod message;
pub mod env;

#[derive(Clone)]
pub enum Value {
    Number(u64),
    Block(Block),
    Slice(Slice),
    Builder(Builder),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Number(number) => number.to_string(),
            Value::Block(block) => format!("[{}]", block.to_string()),
            Value::Slice(slice) => format!("SLICE {}", slice.len()),
            Value::Builder(builder) => format!("BUILDER {}", builder.len()),
        }
    }
}

impl AsBlock for Value {
    fn get_as_block(&self) -> Block {
        let mut builder = Builder::new();
        match self {
            Value::Number(number) => {
                builder.write_u8(0);
                builder.write_u64(number.clone());
            },
            Value::Block(block) => {
                builder.write_u8(1);
                builder.write_block_with_len(block.clone());
            },
            Value::Slice(_) => {
                builder.write_u8(2);
            },
            Value::Builder(_) => {
                builder.write_u8(3);
            },
        }
        builder.build()
    }
}

pub trait SendMessage {
    fn send_message(&mut self, message: Message);
}

pub struct VM<'a> {
    pc: usize,
    
    code: Vec<u8>,
    values: Stack<Value>,
    calls: Stack<usize>,
    
    data: Block,

    message: Message,
    send_message: &'a mut dyn SendMessage,
    
    stopped: bool,
}

// Impl для того чтоб в стеке можно сразу получить по типу, для уменьшение кода
impl Stack<Value> {
    pub fn get_number(&self, offset: usize) -> Option<u64> {
        if let Some(value) = self.get(offset) {
            if let Value::Number(number) = value {
                Some(number.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_block(&self, offset: usize) -> Option<Block> {
        if let Some(value) = self.get(offset) {
            if let Value::Block(block) = value {
                Some(block.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut_slice(&mut self, offset: usize) -> Option<&mut Slice> {
        if let Some(value) = self.get_mut(offset) {
            if let Value::Slice(block) = value {
                Some(block)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut_builder(&mut self, offset: usize) -> Option<&mut Builder> {
        if let Some(value) = self.get_mut(offset) {
            if let Value::Builder(block) = value {
                Some(block)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> VM<'a> {
    pub fn new(code: Vec<u8>, pc: usize, data: Block, message: Message, send_message: &'a mut dyn SendMessage) -> Self {
        Self {
            pc,
            stopped: true,
            code,
            values: Stack::new(),
            calls: Stack::new(),
            data,
            message,
            send_message,
        }
    }

    pub fn next(&mut self, length: usize) -> Option<&[u8]> {
        let slice = self.code.get(self.pc..(self.pc+length))?;
        self.pc += length;
        Some(slice)
    }

    pub fn next_u64(&mut self) -> Option<u64> {
        get_u64(self.next(size_of::<u64>())?)
    }

    pub fn next_u8(&mut self) -> Option<u8> {
        get_u8(self.next(size_of::<u8>())?)
    }

    pub fn next_u16(&mut self) -> Option<u16> {
        get_u16(self.next(size_of::<u16>())?)
    }

    fn execute(&mut self, opcode: u8) {
        if opcode == instructions::IPUSH64 {
            if let Some(value) = self.next_u64() {
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::IPUSH8 {
            if let Some(value) = self.next_u8() {
                self.values.push(Value::Number(value as u64));
            }
        } else if opcode == instructions::SPUSH {
            if let Some(offset) = self.next_u16() {
                if let Some(stacked_value) = self.values.get(offset as usize) {
                    self.values.push(stacked_value.clone());
                }
            }
        } else if opcode == instructions::DROPN {
            if let Some(length) = self.next_u16() {
                self.values.drop(length as usize);
            }
        } else if opcode == instructions::CHG {
            let first = self.next_u16();
            let second = self.next_u16();
            if first.is_some() && second.is_some() {
                let first = first.unwrap();
                let second = second.unwrap();
                self.values.change(first as usize, second as usize);
            }
        } else if opcode == instructions::SWAP {
            self.values.change(0, 1);
        } else if opcode == instructions::BPUSH {
            if let Some(size) = self.next_u64() {
                if let Some(block) = self.next(size as usize).map(|x| Block::from(x)) {
                    self.values.push(Value::Block(block));
                }
            }
        } else if opcode == instructions::BHASH {
            if let Some(block) = self.values.get_block(0) {
                match block.hash() {
                    hash => {
                        self.values.pop();
                        self.values.push(Value::Block(hash));
                    }
                }
            }
        } else if opcode == instructions::BLEN {
            if let Some(block) = self.values.get_block(0) {
                match block.len() {
                    length => {
                        self.values.push(Value::Number(length as u64));
                    }
                }
            }
        } else if opcode == instructions::MKSLICE {
            if let Some(block) = self.values.get_block(0) {
                self.values.pop();
                self.values.push(Value::Slice(Slice::new(block)));
            }
        } else if opcode == instructions::IREAD64 {
            if let Some(slice) = self.values.get_mut_slice(0) {
                if let Some(value) = slice.read_u64() {
                    self.values.push(Value::Number(value));
                }
            }
        } else if opcode == instructions::IREAD8 {
            if let Some(slice) = self.values.get_mut_slice(0) {
                if let Some(value) = slice.read_u8() {
                    self.values.push(Value::Number(value as u64));
                }
            }
        } else if opcode == instructions::BREAD {
            let length = self.values.get_number(0);
            if let Some(length) = length  {
                if let Some(slice) = self.values.get_mut_slice(1) {
                    if let Some(block) = slice.read_block(length as usize) {
                        self.values.pop();
                        self.values.push(Value::Block(block));
                    }
                }
            }
        } else if opcode == instructions::SLLEN {
            if let Some(slice) = self.values.get_mut_slice(0) {
                match slice.len() {
                    length => {
                        self.values.push(Value::Number(length as u64));
                    }
                }
            }
        } else if opcode == instructions::MKBUILDER {
            self.values.push(Value::Builder(Builder::new()));
        } else if opcode == instructions::IWRITE8 {
            if let Some(value) = self.values.get_number(0) {
                if let Some(builder) = self.values.get_mut_builder(1) {
                    builder.write_u8(value as u8);
                    self.values.pop();
                }
            }
        } else if opcode == instructions::IWRITE64 {
            if let Some(value) = self.values.get_number(0) {
                if let Some(builder) = self.values.get_mut_builder(1) {
                    builder.write_u64(value);
                    self.values.pop();
                }
            }
        } else if opcode == instructions::BWRITE {
            if let Some(value) = self.values.get_block(0) {
                if let Some(builder) = self.values.get_mut_builder(1) {
                    builder.write_block(value);
                    self.values.pop();
                }
            }
        } else if opcode == instructions::BUILD {
            if let Some(builder) = self.values.get_mut_builder(0) {
                let block = builder.build();
                self.values.pop();
                self.values.push(Value::Block(block));
            }
        } else if opcode == instructions::BLLEN {
            if let Some(builder) = self.values.get_mut_builder(0) {
                match builder.len() {
                    length => {
                        self.values.push(Value::Number(length as u64));
                    }
                }
            }
        } else if opcode == instructions::ADD {
            if let Some(value) = operate(self.values.pair(), |a, b| a + b) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::SUB {
            if let Some(value) = operate(self.values.pair(), |a, b| a - b) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::MUL {
            if let Some(value) = operate(self.values.pair(), |a, b| a * b) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::DIV {
            if let Some(value) = operate(self.values.pair(), |a, b| a / b) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::MOD {
            if let Some(value) = operate(self.values.pair(), |a, b| a % b) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::INC {
            if let Some(value) = self.values.get_mut(0) {
                if let Value::Number(number) = value {
                    *number += 1;
                }
            }
        } else if opcode == instructions::CMB {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a > b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::CML {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a < b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::CMBE {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a >= b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::CMLE {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a <= b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::CME {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a == b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::CMNE {
            if let Some(value) = operate(self.values.pair(), |a, b| cond_sign(a != b)) {
                self.values.pop_pair();
                self.values.push(Value::Number(value));
            }
        } else if opcode == instructions::JMP {
            if let Some(reference) = self.next_u64() {
                self.pc = reference as usize;
            }
        } else if opcode == instructions::JMT {
            if let Some(reference) = self.next_u64() {
                if let Some(value) = self.values.get_number(0) {
                    if value != 0 {
                        self.pc = reference as usize;
                    }
                    self.values.drop(1);
                }
            }
        } else if opcode == instructions::JMF {
            if let Some(reference) = self.next_u64() {
                if let Some(value) = self.values.get_number(0) {
                    if value == 0 {
                        self.pc = reference as usize;
                    }
                    self.values.drop(1);
                }
            }
        } else if opcode == instructions::RJMP {
            if let Some(reference) = self.next_u16() {
                let (direction, offset) = get_relative_reference(reference);
                match direction {
                    utils::Direction::Forward => self.pc += offset as usize,
                    utils::Direction::Backward => self.pc -= offset as usize,
                }
            }
        } else if opcode == instructions::RJMT {
            if let Some(reference) = self.next_u16() {
                if let Some(value) = self.values.get_number(0) {
                    if value != 0 {
                        let (direction, offset) = get_relative_reference(reference);
                        match direction {
                            utils::Direction::Forward => self.pc += offset as usize,
                            utils::Direction::Backward => self.pc -= offset as usize,
                        }
                    }
                    self.values.drop(1);
                }
            }
        } else if opcode == instructions::RJMF {
            if let Some(reference) = self.next_u16() {
                if let Some(value) = self.values.get_number(0) {
                    if value == 0 {
                        let (direction, offset) = get_relative_reference(reference);
                        match direction {
                            utils::Direction::Forward => self.pc += offset as usize,
                            utils::Direction::Backward => self.pc -= offset as usize,
                        }
                    }
                    self.values.drop(1);
                }
            }
        } else if opcode == instructions::HALT {
            self.stopped = true;
        } else if opcode == instructions::CALL {
            if let Some(ip) = self.next_u64() {
                self.calls.push(self.pc);
                self.pc = ip as usize;
            }
        } else if opcode == instructions::RET {
            if let Some(ip) = self.calls.pop() {
                self.pc = ip;
            }
        } else if opcode == instructions::LDATA {
            self.values.push(Value::Block(self.data.clone()));
        } else if opcode == instructions::SDATA {
            if let Some(block) = self.values.get_block(0) {
                self.data = block;
                self.values.pop();
            }
        } else if opcode == instructions::MESSAGE {
            self.values.push(Value::Block(self.message.get_as_block()));
        } else if opcode == instructions::SEND {
            let receiver = self.values.get_block(3);
            let init_block = self.values.get_block(2);
            let opcode = self.values.get_number(1);
            let body = self.values.get_block(0);
            if receiver.is_some() && init_block.is_some() && opcode.is_some() && body.is_some() {
                let init = Init::from_block(init_block.unwrap());
                self.send_message.send_message(Message::new(
                    message::MessageType::Internal,
                    body.unwrap(),
                    opcode.unwrap(),
                    self.message.receiver.clone(),
                    receiver.unwrap(),
                    init,
                ));
                self.values.drop(4);
            }
        }
    }

    pub fn run(&mut self) {
        self.stopped = false;
        while let Some(opcode) = self.next_u8() {
            if self.stopped {
                break;
            }
            self.execute(opcode);
        }
        println!("VM STACK:");
        for message in self.stack() {
            println!("  {}", message.to_string());
        }
    }

    pub fn stack(&self) -> Vec<Value> {
        self.values.get_vector()
    }

    pub fn get_data(&self) -> Block {
        self.data.clone()
    }
}
