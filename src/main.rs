mod vm;

fn main() {
    let source = std::env::args().nth(1).expect("No source passed into program");

    let mut parser = vm::parser::parser::Parser::new(source);
    let mut virtual_machine = vm::virtual_machine::VirtualMachine::new(&mut parser);

    if let Err(parser_error) = virtual_machine.parse() {
        panic!("{}", parser_error);
    };

    if let Err(vm_error) = virtual_machine.run() {
        panic!("{}", vm_error);
    };
}
