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

            match self.collection[self.pointer].token_type {
                TokenType::Command(CommandType::Push(ref n)) => {
                    self.stack.push(*n);
                },

                TokenType::Command(CommandType::Dup) => {
                    self.stack.push(self.get_stack(0)?);
                },

                TokenType::Command(CommandType::Copy(ref n)) => {
                    self.stack.push(self.get_stack(*n as usize)?);
                },

                TokenType::Command(CommandType::Swap) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n1);
                    self.stack.push(n2);
                },

                TokenType::Command(CommandType::Disc) => {
                    self.pop_stack(0)?;
                },

                TokenType::Command(CommandType::Slide(ref n)) => {
                    let n = *n;

                    let top = self.pop_stack(0)?;

                    for _ in 0..n {
                        self.pop_stack(0)?;
                    };

                    self.stack.push(top);
                },


                TokenType::Command(CommandType::Add) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n2 + n1);
                },

                TokenType::Command(CommandType::Sub) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n2 - n1);
                },

                TokenType::Command(CommandType::Mult) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n2 * n1);
                },

                TokenType::Command(CommandType::IDiv) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n2 / n1);
                },

                TokenType::Command(CommandType::Mod) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;

                    self.stack.push(n2 % n1);
                },


                TokenType::Command(CommandType::Store) => {
                    let n1 = self.pop_stack(0)?;
                    let n2 = self.pop_stack(0)?;
                    
                    if n2 < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    };
                    
                    self.heap.insert(n2 as u32, n1);
                },

                TokenType::Command(CommandType::Retr) => {
                    let n1 = self.pop_stack(0)?;
                    
                    if n1 < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    }
                    
                    let res = self.heap.get(&(n1 as u32));

                    if let None = res {
                        self.heap.insert(n1 as u32, 0);

                        self.stack.push(0);
                    }
                    else
                    {
                        self.stack.push(*res.unwrap());
                    };
                },


                TokenType::Command(CommandType::Labl(_)) => {},

                TokenType::Command(CommandType::Call(ref label)) => {
                    self.subroutine_stack.push(self.pointer);

                    self.pointer = self.get_label(label)? + 1;

                    continue;
                },

                TokenType::Command(CommandType::Jump(ref label)) => {
                    //+ 1 because labels will attempt to skip commands if it's a subroutine
                    self.pointer = self.get_label(label)? + 1;

                    continue;
                },

                TokenType::Command(CommandType::JumpZ(ref label)) => {
                    let label = label.clone();

                    if self.pop_stack(0)? == 0 {
                        self.pointer = self.get_label(&label)? + 1;

                        continue;
                    };
                },

                TokenType::Command(CommandType::JumpN(ref label)) => {
                    let label = label.clone();
                    
                    if self.pop_stack(0)? < 0 {
                        self.pointer = self.get_label(&label)? + 1;

                        continue;
                    };
                },

                TokenType::Command(CommandType::EndS) => {
                    if let Some(addr) = self.subroutine_stack.pop() {
                        //+ 1 so it doesn't jump to the call command
                        self.pointer = addr + 1;
                    };

                    continue;
                },

                TokenType::Command(CommandType::EndP) => {
                    break;
                },


                TokenType::Command(CommandType::OutC) => {
                    print!("{}", char::from_u32(self.pop_stack(0)? as u32).unwrap());
                },

                TokenType::Command(CommandType::OutI) => {
                    print!("{}", self.pop_stack(0)?);
                },

                TokenType::Command(CommandType::ReadC) => {
                    let loc = self.get_stack(0)?;

                    if loc < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    };

                    let mut input: [u8; 1] = [0];

                    if let Err(_) = io::stdin().read_exact(&mut input) {
                        return Err(VMError::new("Could not read from user input"));
                    };

                    self.heap.insert(loc as u32, input[0] as i32);
                },

                TokenType::Command(CommandType::ReadI) => {
                    let loc = self.get_stack(0)?;

                    if loc < 0 {
                        return Err(VMError::new("Heap index can not be negative"));
                    };

                    let mut input = String::new();

                    if let Err(_) = io::stdin().read_line(&mut input) {
                        return Err(VMError::new("Could not read from user input"));
                    };

                    if let Ok(num) = input.trim().parse::<i32>() {
                        self.heap.insert(loc as u32, num);

                    } else {
                        return Err(VMError::new(&format!("Could not read number {}", input)));
                    };
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

    fn pop_stack(&mut self, index: usize) -> Result<i32, VMError> {
        let res = self.get_stack(index)?;

        self.stack.pop();

        return Ok(res);
    }

    fn get_label(&self, label: &str) -> Result<usize, VMError> {
        let label_addr = self.labels.get(label);

        if let None = label_addr {
            return Err(VMError::new(&format!("Label {} does not exist", label)));
        }

        Ok(*label_addr.unwrap())
    }
}