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
    chunk.write_opcode(OpCode::OpReturn, 133);
    //chunk.disassemble("test chunk", &mut io::stdout());
    vm.interpret(&chunk);
    chunk.free();

    vm.free();
}
