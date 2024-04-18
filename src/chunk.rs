use crate::value::*;
use crate::vm::Emmitable;
use std::fmt::Display;
use std::io::Write;
#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Constant(Value),
    Negate,
    Return,
    Add,
    Substract,
    Multiply,
    Divide,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpCode")
    }
}
pub struct ChunkEntry {
    code: OpCode,
    line: usize,
}

impl Display for ChunkEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            OpCode::Return => Ok(()),
            OpCode::Constant(val) => write!(f, "{}", val),
            OpCode::Negate => Ok(()),
            _ => Ok(()),
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
    lines: Vec<(usize, usize)>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_opcode(&mut self, code: OpCode, line: usize) {
        self.code.push(ChunkEntry::new(code, line));
        self.add_line(line)
    }

    pub fn add_constant(&mut self, value: Value, line: usize) {
        let constant = OpCode::Constant(value);
        let chunk_entry = ChunkEntry {
            code: constant,
            line,
        };
        self.code.push(chunk_entry);
        self.add_line(line)
    }

    fn add_line(&mut self, line_number: usize) {
        if let Some(last) = self.lines.last_mut() {
            if last.1 == line_number {
                last.0 += 1;
                return;
            }
        }
        self.lines.push((1, line_number))
    }
    fn get_line(&self, index: usize) -> Option<usize> {
        let mut total_len: usize = 0;
        for (len, value) in &self.lines {
            total_len += *len;
            if total_len > index {
                return Some(*value);
            }
        }
        None
    }
    pub fn free(&mut self) {
        self.code = Vec::new();
        self.lines = Vec::new();
    }

    pub fn disassemble(&self, chunk_name: &str, output: &mut impl Write) {
        writeln!(output, "=={}==", chunk_name).unwrap();

        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset, output)
        }
    }
    pub fn disassemble_instruction(&self, offset: usize, output: &mut impl Write) -> usize {
        write!(output, "{offset:04}").unwrap();

        if offset > 0 && self.get_line(offset) == self.get_line(offset - 1) {
            write!(output, "    | ").unwrap();
        } else {
            write!(output, " {:4} ", self.get_line(offset).unwrap()).unwrap();
        }
        let instruction = &self.code[offset];
        match instruction.code {
            OpCode::Return => self.simple_instruction("OP_RETURN", offset, output),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset, output),
            OpCode::Add => self.simple_instruction("OP_ADD", offset, output),
            OpCode::Substract => self.simple_instruction("OP_SUBSTRACT", offset, output),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset, output),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset, output),
            OpCode::Constant(value) => {
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

    pub fn read(&self, index: usize) -> OpCode {
        self.code[index].code.clone()
    }
}

impl Emmitable for Chunk {
    fn emit_byte(&mut self, byte: u8) {
        self.write_opcode(OpCode::from(byte), 0)
    }
    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.write_opcode(OpCode::from(byte1), 0);
        self.add_constant(byte2 as Value, 0);
    }
    fn read(&self, ip: usize) -> OpCode {
        self.code[ip].code.clone()
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Constant(0.0),
            1 => Self::Return,
            2 => Self::Negate,
            3 => Self::Add,
            4 => Self::Substract,
            5 => Self::Multiply,
            6 => Self::Divide,
            _ => todo!("Undefined opcode conversion!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_write_opcode_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Return, 123);
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
        chunk.write_opcode(OpCode::Return, 23);
        chunk.add_constant(1.2, 43);
        chunk.free();
        assert_eq!(chunk.code.len(), 0)
    }

    #[rstest]
    #[case::debug_op_return(OpCode::Return, 1233, "test", b"==test==\n0000 1233 OP_RETURN\n")]
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

    #[rstest]
    fn test_reading_byte_from_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Return, 1);

        let result = chunk.read(0);

        assert_eq!(result, OpCode::Return)
    }
}
