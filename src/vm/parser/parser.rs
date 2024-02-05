use super::token::{Token, TokenType, ImpType, CommandType, Location};
use super::parser_error::ParserError;

macro_rules! make_token {
    ($self: expr, $token_type: expr, $start:expr, $end: expr) => {
        {
            let tok = Ok(Token {
                token_type: $token_type,
        
                location: Location {
                    start: $start,
                    end: $end,
    
                    line: $self.line,
                }
            });
    
            for _ in $start..$end {
                $self.consume_char();
            }
    
            tok
        }
    }
}

macro_rules! make_parser_error {
    ($self: expr, $message: expr, $line: expr) => {
        {
            let res = Err(ParserError::new(&format!($message, $line)));

            $self.consume_char();

            res
        }
    }
}

pub struct Parser {
    queue: std::collections::VecDeque<char>,

    pointer: usize,
    line: usize,
}

impl Parser {
    pub fn new(source: String) -> Self {
        let mut obj = Self {
            queue: source.chars().collect(),

            pointer: 0,
            line: 1,
        };

        obj.queue.retain(|&c| c.is_whitespace());

        return obj;
    }

    pub fn read_token(&mut self) -> Result<Token, ParserError> {
        if let TokenType::Imp(imp_type) = self.read_imp()?.token_type {
            self.read_command(imp_type)
        } else {panic!("read_imp unexpectedly returned a non imp token")}
    }

    fn read_number(&mut self) -> Result<i32, ParserError> {
        let mut bin = String::new();

        while let Some(c) = self.read_char() {
            if c != ' ' && c != '\t' {break;};

            bin += if c == ' ' {"0"} else {"1"};

            self.consume_char();
        };

        if let Some(c) = self.read_char() {
            if c != '\n' {
                return Err(ParserError::new(&format!("Number on line {} did not terminate with a linefeed", self.line)));
            };

            self.consume_char();
        } else {
            return Err(ParserError::new("Source unexpectedly ended while parsing number"));
        };

        if let Ok(res) = i32::from_str_radix(&bin, 2) {
            Ok(res)
        } else {Err(ParserError::new(&format!("Unable to parse number on line {}", self.line)))}
    }

    fn read_label(&mut self) -> Result<String, ParserError> {
        let mut label = String::new();

        while let Some(c) = self.read_char() {
            if c != ' ' && c != '\t' {break;};

            label += &c.to_string();

            self.consume_char();
        };

        if let Some(c) = self.read_char() {
            if c != '\n' {
                return Err(ParserError::new(&format!("Label on line {} did not terminate with a linefeed", self.line)));
            };

            self.consume_char();
        } else {
            return Err(ParserError::new("Source unexpectedly ended while parsing label"));
        };

        Ok(label)
    }

    fn read_command(&mut self, imp_type: ImpType) -> Result<Token, ParserError> {
        match imp_type {
            ImpType::Stack => {
                match self.read_char() {
                    Some(' ') => {
                        let token = (make_token!(self, TokenType::Command(CommandType::Push(0)), self.pointer, self.pointer + 1) as Result<Token, ParserError>).unwrap();

                        Ok(Token {
                            token_type: TokenType::Command(CommandType::Push(self.read_number()?)),
                            location: token.location,
                        })
                    },

                    Some('\n') => {
                        match self.peek_char(1) {
                            Some(' ') => make_token!(self, TokenType::Command(CommandType::Dup), self.pointer, self.pointer + 2),
                            Some('\t') => make_token!(self, TokenType::Command(CommandType::Swap), self.pointer, self.pointer + 2),
                            Some('\n') => make_token!(self, TokenType::Command(CommandType::Disc), self.pointer, self.pointer + 2),

                            _ => make_parser_error!(self, "Could not parse Stack command on line {}", self.line),
                        }
                    },

                    Some('\t') => {
                        match self.peek_char(1) {
                            Some(' ') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::Copy(0)), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::Copy(self.read_number()?)),
                                    location: token.location,
                                })
                            },

                            Some('\n') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::Slide(0)), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::Slide(self.read_number()?)),
                                    location: token.location,
                                })
                            },

                            _ => make_parser_error!(self, "Could not parse Stack command on line {}", self.line),
                        }
                    },

                    _ => make_parser_error!(self, "Could not parse Stack command on line {}", self.line),
                }
            },

            ImpType::Arithmetic => {
                match self.read_char() {
                    Some(' ') => {
                        match self.peek_char(1) {
                            Some(' ') => make_token!(self, TokenType::Command(CommandType::Add), self.pointer, self.pointer + 2),
                            Some('\t') => make_token!(self, TokenType::Command(CommandType::Sub), self.pointer, self.pointer + 2),
                            Some('\n') => make_token!(self, TokenType::Command(CommandType::Mult), self.pointer, self.pointer + 2),

                            _ => make_parser_error!(self, "Could not parse Arithmetic command on line {}", self.line),
                        }
                    },

                    Some('\t') => {
                        match self.peek_char(1) {
                            Some(' ') => make_token!(self, TokenType::Command(CommandType::IDiv), self.pointer, self.pointer + 2),
                            Some('\t') => make_token!(self, TokenType::Command(CommandType::Mod), self.pointer, self.pointer + 2),

                            _ => make_parser_error!(self, "Could not parse Arithmetic command on line {}", self.line),
                        }
                    },

                    _ => make_parser_error!(self, "Could not parse Arithmetic command on line {}", self.line),
                }
            },

            ImpType::Heap => {
                match self.read_char() {
                    Some(' ') => make_token!(self, TokenType::Command(CommandType::Store), self.pointer, self.pointer + 1),
                    Some('\t') => make_token!(self, TokenType::Command(CommandType::Retr), self.pointer, self.pointer + 1),

                    _ => make_parser_error!(self, "Could not parse Heap command on line {}", self.line),
                }
            },

            ImpType::Flow => {
                match self.read_char() {
                    Some(' ') => {
                        match self.peek_char(1) {
                            Some(' ') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::Labl(String::new())), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::Labl(self.read_label()?)),
                                    location: token.location,
                                })
                            },

                            Some('\t') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::Call(String::new())), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::Call(self.read_label()?)),
                                    location: token.location,
                                })
                            },

                            Some('\n') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::Jump(String::new())), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::Jump(self.read_label()?)),
                                    location: token.location,
                                })
                            },

                            _ => make_parser_error!(self, "Could not parse Flow command on line {}", self.line),
                        }
                    },

                    Some('\t') => {
                        match self.peek_char(1) {
                            Some(' ') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::JumpZ(String::new())), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::JumpZ(self.read_label()?)),
                                    location: token.location,
                                })
                            },

                            Some('\t') => {
                                let token = (make_token!(self, TokenType::Command(CommandType::JumpN(String::new())), self.pointer, self.pointer + 2) as Result<Token, ParserError>).unwrap();
        
                                Ok(Token {
                                    token_type: TokenType::Command(CommandType::JumpN(self.read_label()?)),
                                    location: token.location,
                                })
                            },
                            
                            Some('\n') => make_token!(self, TokenType::Command(CommandType::EndS), self.pointer, self.pointer + 2),
    
                            _ => make_parser_error!(self, "Could not parse Flow command on line {}", self.line),
                        }
                    },

                    Some('\n') => {
                        match self.peek_char(1) {
                            Some('\n') => make_token!(self, TokenType::Command(CommandType::EndP), self.pointer, self.pointer + 2),
    
                            _ => make_parser_error!(self, "Could not parse Flow command on line {}", self.line),
                        }
                    },

                    _ => make_parser_error!(self, "Could not parse Flow command on line {}", self.line),
                }
            },

            ImpType::IO => {
                match self.read_char() {
                    Some(' ') => {
                        match self.peek_char(1) {
                            Some(' ') => make_token!(self, TokenType::Command(CommandType::OutC), self.pointer, self.pointer + 2),
                            Some('\t') => make_token!(self, TokenType::Command(CommandType::OutI), self.pointer, self.pointer + 2),
    
                            _ => make_parser_error!(self, "Could not parse IO command on line {}", self.line),
                        }
                    },

                    Some('\t') => {
                        match self.peek_char(1) {
                            Some(' ') => make_token!(self, TokenType::Command(CommandType::ReadC), self.pointer, self.pointer + 2),
                            Some('\t') => make_token!(self, TokenType::Command(CommandType::ReadI), self.pointer, self.pointer + 2),

                            _ => make_parser_error!(self, "Could not parse IO command on line {}", self.line),
                        }
                    },

                    _ => make_parser_error!(self, "Could not parse IO command on line {}", self.line),
                }
            },
        }
    }

    fn read_imp(&mut self) -> Result<Token, ParserError> {
        match self.read_char() {
            Some(' ') => make_token!(self, TokenType::Imp(ImpType::Stack), self.pointer, self.pointer + 1),
            Some('\n') => make_token!(self, TokenType::Imp(ImpType::Flow), self.pointer, self.pointer + 1),

            Some('\t') => match self.peek_char(1) {
                Some(' ') => make_token!(self, TokenType::Imp(ImpType::Arithmetic), self.pointer, self.pointer + 2),
                Some('\t') => make_token!(self, TokenType::Imp(ImpType::Heap), self.pointer, self.pointer + 2),
                Some('\n') => make_token!(self, TokenType::Imp(ImpType::IO), self.pointer, self.pointer + 2),
                
                _ => make_parser_error!(self, "Could not parse IMP on line {}", self.line),
            },

            _ => make_parser_error!(self, "Could not parse IMP on line {}", self.line),
        }
    }

    fn read_char(&self) -> Option<char> {
        self.queue.get(0).copied()
    }

    fn peek_char(&self, amount: usize) -> Option<char> {
        self.queue.get(amount).copied()
    }

    fn consume_char(&mut self) -> Option<char> {
        let res = self.read_char();

        self.queue.pop_front();

        self.pointer += 1;
        if let Some(c) = res {
            if c == '\n' {self.line += 1;};
        }

        return res;
    }

    pub fn is_end(&self) -> bool {
        self.queue.is_empty()
    }
}
