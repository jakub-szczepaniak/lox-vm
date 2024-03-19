use std::io;

use chunk::*;

mod chunk;
mod value;
fn main() {
    let mut chunk = Chunk::new();

    chunk.write_opcode(OpCode::OpReturn);
    chunk.add_constant(1.2);
    chunk.write_opcode(OpCode::OpConstant);
    chunk.write(0);
    chunk.write_opcode(OpCode::OpReturn);
    chunk.disassemble("test chunk", &mut io::stdout());

    chunk.free();
}
