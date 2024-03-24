use chunk::*;
use std::io;
use vm::*;

mod chunk;
mod value;
mod vm;
fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();

    chunk.add_constant(14.0, 123);
    chunk.add_constant(0.1, 2);
    chunk.write_opcode(OpCode::OpDivide, 124);
    chunk.write_opcode(OpCode::OpReturn, 133);
    //chunk.disassemble("test chunk", &mut io::stdout());
    let result = vm.interpret(&chunk);
    match result {
        InterpretResult::InterpretRuntimeError => println!("Runtime error"),
        _ => println!("OK"),
    }
    chunk.free();

    vm.free();
}
