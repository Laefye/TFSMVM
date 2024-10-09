use std::{io::{BufRead, BufReader, BufWriter, Write}, net::{TcpListener, TcpStream}, time::{SystemTime, UNIX_EPOCH}};

use crate::{repositories::PolaDBRepository, vm::{block::{AsBlock, Block}, builder::Builder, env::{Environment, Repository}, message::{Message, MessageType}}};


pub struct Server {
    repository: PolaDBRepository,
    listener: TcpListener,
}

impl Server {
    pub fn new() -> Self {
        Self {
            repository: PolaDBRepository::new(),
            listener: TcpListener::bind("127.0.0.1:4959").unwrap(),
        }
    }

    pub fn handle(&self, stream: TcpStream) {
        let mut buf_reader = BufReader::new(&stream);
        let mut buf_writer = BufWriter::new(&stream);
        let _ = buf_writer.write("connected\r\n".as_bytes());
        let _ = buf_writer.flush();
        loop {
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(sz) => {
                    if sz == 0 {break;}
                    let words: Vec<String> = line.trim().split(" ").map(|x| x.to_string()).collect();
                    if words.len() == 2 {
                        if words[0] == "send" {
                            let hex = hex::decode(words[1].clone()).ok();
                            if hex.is_none() {
                                let _ = buf_writer.write("invalid hex\r\n".as_bytes());
                                let _ = buf_writer.flush();
                                continue;
                            }
                            let message = Message::from_block(Block::new(&hex.unwrap()));
                            if message.is_none() {
                                let _ = buf_writer.write("invalid message\r\n".as_bytes());
                                let _ = buf_writer.flush();
                                continue;
                            }
                            let message = message.unwrap();
                            if message.timestamp < (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - 10000) as u64 {
                                let _ = buf_writer.write("invalid message time\r\n".as_bytes());
                                let _ = buf_writer.flush();
                                continue;
                            } else if message.timestamp > (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() + 10000) as u64 {
                                let _ = buf_writer.write("invalid message time\r\n".as_bytes());
                                let _ = buf_writer.flush();
                                continue;
                            } else if message.sender.len() > 0 {
                                let _ = buf_writer.write("sender must be empty\r\n".as_bytes());
                                let _ = buf_writer.flush();
                                continue;
                            }
                            match message.message_type {
                                MessageType::Internal => {
                                    let _ = buf_writer.write("cant be internal message\r\n".as_bytes());
                                    let _ = buf_writer.flush();
                                    continue;
                                },
                                MessageType::External => {
                                    let transaction = Environment::start_transaction(message, self.repository.get_ref());
                                    let _ = buf_writer.write((transaction.get_as_block().to_string() + "\r\n").as_bytes());
                                    let _ = buf_writer.flush();
                                    continue;
                                },
                                MessageType::View => {
                                    let stack = Environment::view(message, self.repository.get_ref());
                                    let mut builder = Builder::new();
                                    builder.write_u64(stack.len() as u64);
                                    for value in stack {
                                        builder.write_block_with_len(value.get_as_block());
                                    }
                                    let _ = buf_writer.write((builder.build().to_string() + "\r\n").as_bytes());
                                    let _ = buf_writer.flush();
                                    continue;
                                },
                            };
                        }
                    } else if words.len() == 3 {
                        if words[0] == "get_all_messages" {
                            let limit = words[1].parse::<u64>().ok();
                            let offset = words[2].parse::<u64>().ok();
                            if limit.is_some() && offset.is_some() {
                                let limit = limit.unwrap();
                                let offset = offset.unwrap();
                                let messages = self.repository.get_ref().borrow().get_all_messages(limit, offset);
                                let mut builder = Builder::new();
                                builder.write_u64(messages.len() as u64);
                                for message in messages  {
                                    builder.write_block_with_len(message.get_as_block());
                                }
                                let _ = buf_writer.write((builder.build().to_string() + "\r\n").as_bytes());
                                let _ = buf_writer.flush(); 
                            }
                        }
                    } else if words.len() == 4 {
                        if words[0] == "get_messages_by_contract" {
                            let address = Block::from_string(words[1].clone());
                            let limit = words[2].parse::<u64>().ok();
                            let offset = words[3].parse::<u64>().ok();
                            if address.is_some() && limit.is_some() && offset.is_some() {
                                let address = address.unwrap();
                                let limit = limit.unwrap();
                                let offset = offset.unwrap();
                                let messages = self.repository.get_ref().borrow().get_messages_by_contract(address, limit, offset);
                                let mut builder = Builder::new();
                                builder.write_u64(messages.len() as u64);
                                for message in messages  {
                                    builder.write_block_with_len(message.get_as_block());
                                }
                                let _ = buf_writer.write((builder.build().to_string() + "\r\n").as_bytes());
                                let _ = buf_writer.flush(); 
                            }
                        }
                    }
                },
                Err(_) => break,
            }
        }
    }

    pub fn listen(&self) {
        let listener = self.listener.incoming();
        for stream in listener {
            let stream = stream.unwrap();
            self.handle(stream);
        }
    }
}
