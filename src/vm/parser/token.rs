#[derive(Debug, PartialEq)]
pub enum ImpType {
    Stack,
    Arithmetic,
    Heap,
    Flow,
    IO,
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Push(i32),
    Dup,
    Copy(i32),
    Swap,
    Disc,
    Slide(i32),

    Add,
    Sub,
    Mult,
    IDiv,
    Mod,
    Store,
    Retr,

    Labl(String),
    Call(String),
    Jump(String),
    JumpZ(String),
    JumpN(String),
    EndS,
    EndP,

    OutC,
    OutI,
    ReadC,
    ReadI,
}

#[derive(Debug)]
pub enum TokenType {
    Imp(ImpType),
    Command(CommandType),
}

#[derive(Debug)]
pub struct Location {
    pub start: usize,
    pub end: usize,

    pub line: usize,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,

    pub location: Location,
}