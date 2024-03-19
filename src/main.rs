use std::io;

use chunk::*;

mod chunk;
fn main() {
    let mut chunk = Chunk::new();

    chunk.write_opcode(OpCode::OpReturn);

    chunk.disassemble("test chunk", &mut io::stdout());

    chunk.free();
}
