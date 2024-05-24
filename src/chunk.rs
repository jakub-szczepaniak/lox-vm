use crate::emmitable::*;
use crate::value::*;
use std::fmt::Display;
use std::io::Write;
#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Constant,
    Return,
    Negate,
    Add,
    Substract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpCode")
    }
}

pub trait OpCodable {
    fn read(&self, ip: usize) -> OpCode;
    fn read_constant(&self, index: usize) -> Value;
    fn disassemble_instruction(&self, offset: usize, output: &mut impl Write) -> usize;
    fn disassemble(&self, chunk_name: &str, output: &mut impl Write);
    fn reset(&mut self);
    fn read_line(&self, index: usize) -> usize;
}

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn get_constant(&self, index: usize) -> Value {
        self.constants.read_at(index)
    }

    pub fn get_line(&self, index: usize) -> usize {
        self.lines[index]
    }

    fn simple_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        writeln!(output, "{name}").unwrap();
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        let constant_index = self.code[offset + 1];
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
    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8, line: usize) {
        self.write(byte1.into(), line);
        self.write(byte2, line);
    }
    fn emit_constant(&mut self, value: Value, line: usize) {
        if let Some(index) = self.make_constant(value) {
            self.emit_bytes(OpCode::Constant, index, line)
        }
    }
    fn make_constant(&mut self, value: Value) -> Option<u8> {
        let index = self.constants.write(value);
        u8::try_from(index).ok()
    }
}

impl OpCodable for Chunk {
    fn reset(&mut self) {
        self.code = Vec::new();
        self.lines = Vec::new();
        self.constants.free();
    }
    fn disassemble(&self, chunk_name: &str, output: &mut impl Write) {
        writeln!(output, "=={}==", chunk_name).unwrap();

        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset, output)
        }
    }
    fn disassemble_instruction(&self, offset: usize, output: &mut impl Write) -> usize {
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
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset, output),
            OpCode::Nil => self.simple_instruction("OP_NIL", offset, output),
            OpCode::True => self.simple_instruction("OP_TRUE", offset, output),
            OpCode::False => self.simple_instruction("OP_FALSE", offset, output),
            OpCode::Not => self.simple_instruction("OP_NOT", offset, output),
            OpCode::Equal => self.simple_instruction("OP_EQUAL", offset, output),
            OpCode::Less => self.simple_instruction("OP_LESS", offset, output),
            OpCode::Greater => self.simple_instruction("OP_GREATER", offset, output),
            OpCode::Print => self.simple_instruction("OP_PRINT", offset, output),
            OpCode::Pop => self.simple_instruction("OP_POP", offset, output),
            OpCode::DefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL", offset, output),
            OpCode::GetGlobal => self.constant_instruction("OP_GET_GLOBAL", offset, output),
            OpCode::SetGlobal => self.constant_instruction("OP_SET_GLOBAL", offset, output),
        }
    }
    fn read(&self, ip: usize) -> OpCode {
        self.code[ip].into()
    }
    fn read_constant(&self, index: usize) -> Value {
        self.get_constant(index)
    }
    fn read_line(&self, index: usize) -> usize {
        self.get_line(index)
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
            7 => Self::Nil,
            8 => Self::True,
            9 => Self::False,
            10 => Self::Not,
            11 => Self::Equal,
            12 => Self::Greater,
            13 => Self::Less,
            14 => Self::Print,
            15 => Self::Pop,
            16 => Self::DefineGlobal,
            17 => Self::GetGlobal,
            18 => Self::SetGlobal,
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
            OpCode::Divide => 6,
            OpCode::Nil => 7,
            OpCode::True => 8,
            OpCode::False => 9,
            OpCode::Not => 10,
            OpCode::Equal => 11,
            OpCode::Greater => 12,
            OpCode::Less => 13,
            OpCode::Print => 14,
            OpCode::Pop => 15,
            OpCode::DefineGlobal => 16,
            OpCode::GetGlobal => 17,
            OpCode::SetGlobal => 18,
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
        chunk.write(OpCode::Return as u8, 123);
        assert_eq!(chunk.code.len(), 1)
    }

    #[rstest]
    fn test_write_constant_to_chunk() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Constant as u8, 1);
        let const_index = chunk.make_constant(Value::Number(1.2)).unwrap();
        chunk.write(const_index, 1);
        assert_eq!(chunk.code.len(), 2)
    }

    #[rstest]
    fn test_free_the_chunk() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Return as u8, 23);
        chunk.make_constant(Value::Number(1.2));
        chunk.reset();
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

        chunk.write(actual as u8, line);

        chunk.disassemble(chunk_name, &mut output);
        assert_eq!(output, expected)
    }

    #[rstest]
    fn test_disassemble_chunk_with_const() {
        let mut chunk = Chunk::new();
        let mut output = Vec::new();
        chunk.write(OpCode::Constant as u8, 1);
        let const_index = chunk.make_constant(Value::Number(12.4)).unwrap();
        chunk.write(const_index, 1);

        chunk.disassemble("test chunk", &mut output);

        assert_eq!(
            output,
            b"==test chunk==\n0000    1 OP_CONSTANT         0 '12.4'\n"
        )
    }

    #[rstest]
    fn test_reading_byte_from_chunk() {
        let mut chunk = Chunk::new();
        chunk.write(OpCode::Return as u8, 1);

        let result = chunk.read(0);

        assert_eq!(result, OpCode::Return)
    }
}
