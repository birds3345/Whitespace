mod vm;

use std::{fs, io::Read};

fn main() {
    let arg = std::env::args().nth(1).expect("No source passed into program");

    let mut source: String = String::new();

    match fs::File::open(arg.clone()) {
        Ok(mut file) => {
            if let Err(_) = file.read_to_string(&mut source) {
                panic!("Could not read from file");
            }
        },

        _ => {
            source = arg;
        },
    };
    
    let mut parser = vm::parser::parser::Parser::new(source);
    let mut virtual_machine = vm::virtual_machine::VirtualMachine::new(&mut parser);

    if let Err(parser_error) = virtual_machine.parse() {
        panic!("{}", parser_error);
    };

    if let Err(vm_error) = virtual_machine.run() {
        panic!("{}", vm_error);
    };
}
