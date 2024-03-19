use std::io::Write;
#[derive(Debug, PartialEq)]
pub enum OpCode {
    OpReturn = 0,
}

impl From<u8> for OpCode {
    fn from(opcode: u8) -> Self {
        match opcode {
            0 => OpCode::OpReturn,
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
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn write_opcode(&mut self, code: OpCode) {
        self.code.push(code.into())
    }

    pub fn len(self) -> usize {
        self.code.len()
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
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
        let _ = match instruction {
            OpCode::OpReturn => writeln!(output, "OP_RETURN"),
        };
        offset + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::opcode_return(0, OpCode::OpReturn)]
    fn test_opcode_from_u8(#[case] from: u8, #[case] expected: OpCode) {
        assert_eq!(OpCode::from(from), expected)
    }

    #[rstest]
    #[case::opcode_return(OpCode::OpReturn, 0)]
    fn test_byte_from_opcode(#[case] from: OpCode, #[case] expected: u8) {
        assert_eq!(u8::from(from), expected)
    }

    #[rstest]
    fn test_write_opcode_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn);
        assert_eq!(chunk.len(), 1)
    }
    #[rstest]
    fn test_free_the_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::OpReturn);
        chunk.free();
        assert_eq!(chunk.len(), 0)
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
}
