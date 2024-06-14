use crate::{chunk::*, value::Value, Emmitable};
use std::cell::RefCell;

use std::{fmt::Display, io::Write};
#[derive(Debug)]
pub struct Function<T: Emmitable + OpCodable> {
    arity: u8,
    name: String,
    pub chunk: RefCell<T>,
}

impl<T: Emmitable + OpCodable> Display for Function<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl<T: Emmitable + OpCodable> PartialEq for Function<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T: Emmitable + OpCodable> PartialOrd for Function<T> {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        panic!("Cannot compare 2 functions")
    }
}

impl<T: Emmitable + OpCodable> Clone for Function<T> {
    fn clone(&self) -> Self {
        Function::<T> {
            arity: self.arity,
            name: self.name.clone(),
            chunk: RefCell::new(self.chunk.borrow().clone()),
        }
    }
}

impl<T: Emmitable + OpCodable> Function<T> {
    pub fn new(name: &str) -> Self {
        Self {
            arity: 0,
            name: name.to_string(),
            chunk: T::initialize().into(),
        }
    }

    pub fn size(&self) -> usize {
        self.chunk.borrow().size()
    }

    pub fn initialize_emiter(&mut self) {
        self.chunk.borrow_mut().initialize_emiter()
    }

    pub fn finalize_emiter(&mut self) {
        self.chunk.borrow_mut().finalize_emiter()
    }

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

    #[rstest]
    fn implements_display() {
        let func: Function<Chunk> = Function::new("my function");
        assert_eq!(func.to_string(), "<fn my function>");
    }
    #[rstest]
    fn implements_partial_equality_false_for_different() {
        let first: Function<Chunk> = Function::new("first");
        let second: Function<Chunk> = Function::new("second");
        assert_ne!(first, second);
    }

    #[rstest]
    fn implements_partial_equality_false_for_same() {
        let first: Function<Chunk> = Function::new("name");
        let second: Function<Chunk> = Function::new("name");
        assert_ne!(first, second)
    }

    #[rstest]
    #[should_panic]
    fn implements_partial_order_always_panics() {
        let first: Function<Chunk> = Function::new("first");
        let second: Function<Chunk> = Function::new("second");

        assert!(first < second);
        assert!(first <= second);
        assert!(first > second);
        assert!(first >= second);
    }
}
