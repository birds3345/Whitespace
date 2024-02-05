use super::parser::{self as parser_mod, parser};

use super::vm_error::VMError;
use parser_mod::parser_error::ParserError;

use parser_mod::token::TokenType;
use parser_mod::token::CommandType;

use std::io::{self, Read};
use std::collections::HashMap;

pub struct VirtualMachine<'a> {
    parser: &'a mut parser::Parser,

    collection: Vec<parser_mod::token::Token>,
    labels: HashMap<String, usize>,

    subroutine_stack: Vec<usize>,
    stack: Vec<i32>,
    heap: HashMap<u32, i32>,

    pointer: usize,

    parsed: bool,
}

impl<'a> VirtualMachine<'a> {
    pub fn new(parser: &'a mut parser::Parser) -> Self {
        Self {
            parser,
            collection: vec![],
            labels: HashMap::<String, usize>::new(),
            
            subroutine_stack: vec![],
            stack: vec![],
            heap: HashMap::<u32, i32>::new(),

            pointer: 0,

            parsed: false,
        }
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        if self.parsed {
            return Err(ParserError::new("Already parsed source"));
        };

        self.parsed = true;

        while !self.parser.is_end() {
            self.collection.push(self.parser.read_token()?);
        }

        for (index, token) in self.collection.iter().enumerate() {
            if let TokenType::Command(CommandType::Labl(label)) = &token.token_type {
                self.labels.insert(label.to_string(), index);
            }
        }

        return Ok(());
    }

    //TODO: make a separate command struct instead of using the token struct
    pub fn run(&mut self) -> Result<(), VMError> {
        while self.pointer != self.collection.len() {
            if self.pointer >= self.collection.len() {
                return Err(VMError::new(&format!("Program pointer out of range (at {})", self.pointer)));
            };

            match &self.collection[self.pointer].token_type {
                TokenType::Command(CommandType::Push(n)) => {
                    self.stack.push(*n);
                },

                TokenType::Command(CommandType::Dup) => {
                    let n = self.get_stack(0)?;

                    self.stack.push(n);
                },

                TokenType::Command(CommandType::Copy(n)) => {
                    let n = self.get_stack(*n as usize)?;

                    self.stack.push(n);
                },

                TokenType::Command(CommandType::Swap) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.pop();
                    self.stack.pop();

                    self.stack.push(n1);
                    self.stack.push(n2);
                },

                TokenType::Command(CommandType::Disc) => {
                    self.get_stack(0)?;

                    self.stack.pop();
                },

                TokenType::Command(CommandType::Slide(n)) => {
                    let top = self.get_stack(0)?;

                    self.stack.pop();

                    for _ in 0..*n {
                        self.get_stack(0)?;
                        self.stack.pop();
                    }

                    self.stack.push(top);
                },


                TokenType::Command(CommandType::Add) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.push(n1 + n2);
                },

                TokenType::Command(CommandType::Sub) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.push(n1 - n2);
                },

                TokenType::Command(CommandType::Mult) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.push(n1 * n2);
                },

                TokenType::Command(CommandType::IDiv) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.push(n1 / n2);
                },

                TokenType::Command(CommandType::Mod) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    self.stack.push(n1 % n2);
                },


                TokenType::Command(CommandType::Store) => {
                    let n1 = self.get_stack(0)?;
                    let n2 = self.get_stack(1)?;

                    if n1 < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    }
                    
                    self.heap.insert(n1 as u32, n2);
                },

                TokenType::Command(CommandType::Retr) => {
                    let n1 = self.get_stack(0)?;

                    if n1 < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    }

                    let res = self.heap.get(&(n1 as u32));

                    if let None = res {
                        return Err(VMError::new(&format!("Heap index {} does not exist", n1)));
                    }

                    self.stack.push(*res.unwrap());
                },


                TokenType::Command(CommandType::Labl(_)) => {},

                TokenType::Command(CommandType::Call(label)) => {
                    self.subroutine_stack.push(self.pointer);

                    self.pointer = self.get_label(label)?;

                    continue;
                },

                TokenType::Command(CommandType::Jump(label)) => {
                    self.pointer = self.get_label(label)?;

                    continue;
                },

                TokenType::Command(CommandType::JumpZ(label)) => {
                    if self.get_stack(0)? == 0 {
                        self.pointer = self.get_label(label)?;

                        continue;
                    }
                },

                TokenType::Command(CommandType::JumpN(label)) => {
                    if self.get_stack(0)? < 0 {
                        self.pointer = self.get_label(label)?;

                        continue;
                    }
                },

                TokenType::Command(CommandType::EndS) => {
                    if let Some(addr) = self.subroutine_stack.get(self.subroutine_stack.len() - 1) {
                        self.pointer = *addr;

                        self.subroutine_stack.pop();
                    }
                },

                TokenType::Command(CommandType::EndP) => {
                    break;
                }


                TokenType::Command(CommandType::OutC) => {
                    print!("{}", self.get_stack(0)? as u8 as char);
                },

                TokenType::Command(CommandType::OutI) => {
                    print!("{}", self.get_stack(0)?);
                },

                TokenType::Command(CommandType::ReadC) => {
                    let loc = self.get_stack(0)?;

                    let mut input: [u8; 1] = [0];

                    if let Err(_) = io::stdin().read_exact(&mut input) {
                        return Err(VMError::new("Could not read from user input"));
                    };

                    if let Some(n) = self.stack.get_mut(loc as usize) {
                        *n = input[0] as i32;
                    }
                    else {
                        return Err(VMError::new("Address does not exist"));
                    }
                },

                TokenType::Command(CommandType::ReadI) => {
                    let loc = self.get_stack(0)?;

                    let mut input = String::new();

                    if let Err(_) = io::stdin().read_line(&mut input) {
                        return Err(VMError::new("Could not read from user input"));
                    };

                    if let Ok(num) = input.trim().parse::<i32>() {
                        if let Some(n) = self.stack.get_mut(loc as usize) {
                            *n = num;
                        }
                        else {
                            return Err(VMError::new("Address does not exist"));
                        }
                    } else {
                        return Err(VMError::new(&format!("Could not read number {}", input)));
                    }
                },

                _ => {
                    return Err(VMError::new("Invalid command"));
                },
            };

            self.pointer += 1;
        }

        return Ok(());
    }

    fn get_stack(&self, index: usize) -> Result<i32, VMError> {
        if self.stack.is_empty() {
            return Err(VMError::new("Stack is empty"));
        }

        let index = (self.stack.len() as isize) - 1 - index as isize;

        if index < 0 {
            return Err(VMError::new("Stack index is negative"));
        }

        let res = self.stack.get(index as usize);

        if let None = res {
            return Err(VMError::new("Stack index out of bounds"));
        }

        Ok(*res.unwrap())
    }

    fn get_label(&self, label: &str) -> Result<usize, VMError> {
        let label_addr = self.labels.get(label);

        if let None = label_addr {
            return Err(VMError::new(&format!("Label {} does not exist", label)));
        }

        Ok(*label_addr.unwrap())
    }
}