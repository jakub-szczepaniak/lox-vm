use std::io;

use chunk::*;

mod chunk;
mod value;
fn main() {
    let mut chunk = Chunk::new();

    chunk.write_opcode(OpCode::OpReturn, 123);
    chunk.add_constant(12.0, 123);

    chunk.add_constant(14.0, 123);
    chunk.write_opcode(OpCode::OpReturn, 133);
    chunk.disassemble("test chunk", &mut io::stdout());
    chunk.free();
}
