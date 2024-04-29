use crate::value::*;
use crate::vm::{Emmitable, OpCodable};
use std::fmt::Display;
use std::io::Write;
#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Constant,
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

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: ValueArray
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new()
        }
    }

    pub fn write(&mut self, byte: u8, line: usize){
        self.code.push(byte);
        self.lines.push(line);
    }
    
    pub fn write_opcode(&mut self, code: OpCode, line: usize) {
        self.code.push(code.into());
        self.lines.push(line)
    }

    pub fn add_constant(&mut self, value: Value) -> Option<u8>{
        let index = self.constants.write(value);
        u8::try_from(index).ok()
    }

    pub fn get_constant(&self, index: usize) -> Value {
        self.constants.read_at(index)
    }

    pub fn free(&mut self) {
        self.code = Vec::new();
        self.lines = Vec::new();
        self.constants.free();
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

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(output, "    | ").unwrap();
        } else {
            write!(output, " {:4} ", self.lines[offset]).unwrap();
        }
        let instruction = &self.code[offset].into();
        match instruction {
            OpCode::Return => self.simple_instruction("OP_RETURN", offset, output),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset, output),
            OpCode::Add => self.simple_instruction("OP_ADD", offset, output),
            OpCode::Substract => self.simple_instruction("OP_SUBSTRACT", offset, output),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset, output),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset, output),
            OpCode::Constant => {
                self.constant_instruction("OP_CONSTANT", offset, output)
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
        output: &mut impl Write,
    ) -> usize {
        let constant_index = self.code[offset+1];
        let value = self.constants.read_at(constant_index as usize);
        write!(output, "{name:-16} {offset:4} '").unwrap();
        write!(output, "{value}").unwrap();
        writeln!(output, "'").unwrap();
        offset + 2
    }

}

impl Emmitable for Chunk {
    fn emit_byte(&mut self, byte: u8, line: usize) {
        self.write(byte, line)
    }
    fn emit_bytes(&mut self,byte1: OpCode, byte2: u8, line: usize) {
        self.write(byte1.into(), line);
        self.write(byte2, line);
    }
    fn emit_constant(&mut self, value: Value, line: usize) {
        if let Some(index) = self.add_constant(value){
            self.emit_bytes(OpCode::Constant, index, line)
        } else {
            ()
        }
    }


}


impl OpCodable for Chunk {
    
    fn read(&self, ip: usize) -> OpCode {
        self.code[ip].into()
    }
    fn read_constant(&self, index:usize) -> Value {
        self.get_constant(index)
    }
}
impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Constant,
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

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Constant => 0,
            OpCode::Return => 1,
            OpCode::Negate => 2, 
            OpCode::Add => 3, 
            OpCode::Substract => 4, 
            OpCode::Multiply => 5, 
            OpCode::Divide => 6
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
        chunk.add_constant(1.2);
        assert_eq!(chunk.code.len(), 1)
    }

    #[rstest]
    fn test_free_the_chunk() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Return, 23);
        chunk.add_constant(1.2);
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
        chunk.add_constant(12.4);

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
