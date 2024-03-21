use crate::value::*;
use std::fmt::Display;
use std::io::Write;
#[derive(Debug, PartialEq)]
pub enum OpCode {
    OpConstant(Value),
    OpReturn,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpCode")
    }
}
struct ChunkEntry {
    code: OpCode,
    line: usize,
}

impl Display for ChunkEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            OpCode::OpReturn => Ok(()),
            OpCode::OpConstant(val) => write!(f, "{}", val),
        }
    }
}

impl ChunkEntry {
    pub fn new(code: OpCode, line: usize) -> Self {
        Self { code, line }
    }
}

pub struct Chunk {
    code: Vec<ChunkEntry>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn write_opcode(&mut self, code: OpCode, line: usize) {
        self.code.push(ChunkEntry::new(code, line))
    }

    pub fn add_constant(&mut self, value: Value, line: usize) {
        let constant = OpCode::OpConstant(value);
        let chunk_entry = ChunkEntry {
            code: constant,
            line,
        };
        self.code.push(chunk_entry)
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
    }

    pub fn disassemble(&self, chunk_name: &str, output: &mut impl Write) {
        writeln!(output, "=={}==", chunk_name).unwrap();

        let mut offset: usize = 0;
        for instruction in &self.code {
            offset = self.disassemble_instruction(offset, instruction, output)
        }
    }
    fn disassemble_instruction(
        &self,
        offset: usize,
        instruction: &ChunkEntry,
        output: &mut impl Write,
    ) -> usize {
        write!(output, "{offset:04} {:4} ", instruction.line).unwrap();

        match instruction.code {
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset, output),
            OpCode::OpConstant(value) => {
                self.constant_instruction("OP_CONSTANT", offset, value, output)
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        writeln!(output, "{name}").unwrap();
        offset + 1
    }

    fn constant_instruction(
        &self,
        name: &str,
        offset: usize,
        value: Value,
        output: &mut impl Write,
    ) -> usize {
        write!(output, "{name:-16} {offset:4} '").unwrap();
        write!(output, "{value}").unwrap();
        writeln!(output, "'").unwrap();
        offset + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_write_opcode_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn, 123);
        assert_eq!(chunk.code.len(), 1)
    }

    #[rstest]
    fn test_write_constant_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.add_constant(1.2, 44);
        assert_eq!(chunk.code.len(), 1)
    }

    #[rstest]
    fn test_free_the_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn, 23);
        chunk.add_constant(1.2, 43);
        chunk.free();
        assert_eq!(chunk.code.len(), 0)
    }

    #[rstest]
    #[case::debug_op_return(OpCode::OpReturn, 1233, "test", b"==test==\n0000 1233 OP_RETURN\n")]
    fn test_dissasemble_the_chunk(
        #[case] actual: OpCode,
        #[case] line: usize,
        #[case] chunk_name: &str,
        #[case] expected: &[u8],
    ) {
        let mut output = Vec::new();
        let mut chunk = Chunk::new();

        chunk.write_opcode(actual, line);

        chunk.disassemble(chunk_name, &mut output);
        assert_eq!(output, expected)
    }

    #[rstest]
    fn test_disassemble_chunk_with_const() {
        let mut chunk = Chunk::new();
        let mut output = Vec::new();
        chunk.add_constant(12.4, 133);

        chunk.disassemble("test chunk", &mut output);

        assert_eq!(
            output,
            b"==test chunk==\n0000  133 OP_CONSTANT         0 '12.4'\n"
        )
    }
}
