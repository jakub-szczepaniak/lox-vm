use crate::opcode::*;
use crate::value::*;
use std::io::Write;

#[derive(Debug)]
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

    pub fn simple_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        writeln!(output, "{name}").unwrap();
        offset + 1
    }

    pub fn jump_instruction(
        &self,
        name: &str,
        offset: usize,
        sign: bool,
        output: &mut impl Write,
    ) -> usize {
        let jump = self.jump_offset(offset + 1);
        let jump_to = if sign {
            offset + 3 + jump
        } else {
            offset - 3 + jump
        };
        writeln!(output, "{name:16} {offset:4} -> {jump_to}").unwrap();
        offset + 3
    }

    pub fn constant_instruction(
        &self,
        name: &str,
        offset: usize,
        output: &mut impl Write,
    ) -> usize {
        let constant_index = self.code[offset + 1];
        let value = self.constants.read_at(constant_index as usize);
        write!(output, "{name:-16} {offset:4} '").unwrap();
        write!(output, "{value}").unwrap();
        writeln!(output, "'").unwrap();
        offset + 2
    }

    pub fn byte_instruction(&self, name: &str, offset: usize, output: &mut impl Write) -> usize {
        let slot = self.code[offset + 1];
        writeln!(output, "{name:-16} {slot:4}").unwrap();
        offset + 2
    }

    pub fn emit_byte(&mut self, byte: u8, line: usize) {
        self.write(byte, line)
    }
    pub fn emit_bytes(&mut self, byte1: OpCode, byte2: u8, line: usize) {
        self.write(byte1.into(), line);
        self.write(byte2, line);
    }
    pub fn emit_constant(&mut self, value: Value, line: usize) {
        if let Some(index) = self.make_constant(value) {
            self.emit_bytes(OpCode::Constant, index, line)
        }
    }
    pub fn make_constant(&mut self, value: Value) -> Option<u8> {
        let index = self.constants.write(value);
        u8::try_from(index).ok()
    }

    pub fn size(&self) -> usize {
        self.code.len()
    }

    pub fn write_at(&mut self, offset: usize, byte: u8) {
        self.code[offset] = byte;
    }
    pub fn jump_offset(&self, offset: usize) -> usize {
        ((self.code[offset] as usize) << 8) | self.code[offset + 1] as usize
    }

    pub fn clone(&self) -> Self {
        Self {
            code: self.code.clone(),
            lines: self.lines.clone(),
            constants: self.constants.clone(),
        }
    }

    pub fn reset(&mut self) {
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
            OpCode::GetLocal => self.byte_instruction("OP_GET_LOCAL", offset, output),
            OpCode::SetLocal => self.byte_instruction("OP_SET_LOCAL", offset, output),
            OpCode::JumpIfFalse => self.jump_instruction("OP_JUMP_IF_FALSE", offset, true, output),
            OpCode::Jump => self.jump_instruction("OP_JUMP", offset, true, output),
            OpCode::Loop => self.jump_instruction("OP_LOOP", offset, false, output),
        }
    }

    pub fn read(&self, ip: usize) -> OpCode {
        self.code[ip].into()
    }
    pub fn read_constant(&self, index: usize) -> Value {
        self.get_constant(index)
    }
    pub fn read_line(&self, index: usize) -> usize {
        self.get_line(index)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;
}
