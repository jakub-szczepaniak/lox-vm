use crate::value::*;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub enum OpCode {
    OpConstant = 0,
    OpReturn,
}

impl From<u8> for OpCode {
    fn from(opcode: u8) -> Self {
        match opcode {
            0 => OpCode::OpConstant,
            1 => OpCode::OpReturn,
            _ => unimplemented!("Not yet implemented!"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write_opcode(&mut self, code: OpCode) {
        self.code.push(code.into())
    }

    pub fn write(&mut self, value: u8) {
        self.code.push(value)
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value);
        self.constants.len() as u8 - 1
    }

    pub fn code_len(self) -> usize {
        self.code.len()
    }

    pub fn const_len(self) -> usize {
        self.constants.len()
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
        self.constants = ValueArray::new();
    }

    pub fn disassemble(&self, chunk_name: &str, output: &mut impl Write) {
        writeln!(output, "=={}==", chunk_name).unwrap();

        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset, output);
        }
    }
    fn disassemble_instruction(&self, offset: usize, output: &mut impl Write) -> usize {
        write!(output, "{offset:04} ").unwrap();
        let instruction: OpCode = self.code[offset].into();
        match instruction {
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset, output),
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset, output),
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        writeln!(output, "{name}").unwrap();
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        let constant_index = self.code[offset + 1];
        write!(output, "{name:-16} {constant_index:4} '").unwrap();
        self.constants.print_value(output, constant_index as usize);
        writeln!(output, "'").unwrap();
        offset + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::opcode_return(1, OpCode::OpReturn)]
    #[case::opcode_constant(0, OpCode::OpConstant)]
    fn test_opcode_from_u8(#[case] from: u8, #[case] expected: OpCode) {
        assert_eq!(OpCode::from(from), expected)
    }

    #[rstest]
    #[case::opcode_return(OpCode::OpConstant, 0)]
    #[case::opcode_return(OpCode::OpReturn, 1)]
    fn test_byte_from_opcode(#[case] from: OpCode, #[case] expected: u8) {
        assert_eq!(u8::from(from), expected)
    }

    #[rstest]
    fn test_write_opcode_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn);
        assert_eq!(chunk.code_len(), 1)
    }

    #[rstest]
    fn test_write_constant_to_chunk() {
        let mut chunk = Chunk::new();
        let index = chunk.add_constant(1.2);
        assert_eq!(index, 0)
    }

    #[rstest]
    fn test_free_the_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn);
        chunk.add_constant(1.2);
        chunk.free();
        assert_eq!(chunk.code_len(), 0)
    }
    #[rstest]
    fn test_free_the_consts() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn);
        chunk.add_constant(1.2);
        chunk.free();
        assert_eq!(chunk.const_len(), 0)
    }
    #[rstest]
    #[case::debug_op_return(OpCode::OpReturn, "test", b"==test==\n0000 OP_RETURN\n")]
    fn test_dissasemble_the_chunk(
        #[case] actual: OpCode,
        #[case] chunk_name: &str,
        #[case] expected: &[u8],
    ) {
        let mut output = Vec::new();
        let mut chunk = Chunk::new();

        chunk.write_opcode(actual);

        chunk.disassemble(chunk_name, &mut output);
        assert_eq!(output, expected)
    }

    #[rstest]
    fn test_disassemble_chunk_with_const() {
        let mut chunk = Chunk::new();
        let mut output = Vec::new();
        let index = chunk.add_constant(12.4);
        chunk.write_opcode(OpCode::OpConstant);
        chunk.write(index);

        chunk.disassemble("test chunk", &mut output);

        assert_eq!(
            output,
            b"==test chunk==\n0000 OP_CONSTANT         0 '12.4'\n"
        )
    }
}
