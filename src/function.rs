use crate::{chunk::*, value::Value};
use std::cell::RefCell;

use std::{fmt::Display, io::Write};
#[derive(Debug)]
pub struct Function {
    arity: u8,
    pub name: String,
    pub chunk: RefCell<Chunk>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        panic!("Cannot compare 2 functions")
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function {
            arity: self.arity,
            name: self.name.clone(),
            chunk: RefCell::new(self.chunk.borrow().clone()),
        }
    }
}

impl Function {
    pub fn new(name: &str) -> Self {
        Self {
            arity: 0,
            name: name.to_string(),
            chunk: Chunk::new().into(),
        }
    }

    pub fn size(&self) -> usize {
        self.chunk.borrow().size()
    }

    pub fn initialize_emiter(&mut self) {}

    pub fn finalize_emiter(&mut self) {}

    pub fn make_constant(&mut self, name: Value) -> u8 {
        self.chunk.borrow_mut().make_constant(name).unwrap()
    }

    pub fn write_at(&mut self, offset: usize, byte: u8) {
        self.chunk.borrow_mut().write_at(offset, byte)
    }

    pub fn emit_byte(&mut self, byte: u8, line: usize) {
        self.chunk.borrow_mut().emit_byte(byte, line)
    }

    pub fn emit_constant(&mut self, val: Value, line: usize) {
        self.chunk.borrow_mut().emit_constant(val, line)
    }

    pub fn disassemble(&self, name: &str, output: &mut impl Write) {
        self.chunk.borrow().disassemble(&self.name, output)
    }

    pub fn disassemble_instruction(&self, offset: usize, output: &mut impl Write) {
        self.chunk.borrow().disassemble_instruction(offset, output);
    }

    pub fn read(&self, ip: usize) -> OpCode {
        self.chunk.borrow().read(ip)
    }

    pub fn jump_offset(&self, ip: usize) -> usize {
        self.chunk.borrow().jump_offset(ip)
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.chunk.borrow().read_constant(index)
    }

    pub fn read_line(&self, index: usize) -> usize {
        self.chunk.borrow().read_line(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
}
