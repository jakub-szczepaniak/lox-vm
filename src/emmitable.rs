use crate::{chunk::*, value::*};
pub trait Emmitable {
    fn emit_byte(&mut self, byte: u8, line: usize);
    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8, line: usize);
    fn emit_constant(&mut self, value: Value, line: usize);
    fn initialize_emiter(&mut self) {}
    fn initialize() -> Self
    where
        Self: std::marker::Sized;
    fn finalize_emiter(&mut self) {}
    fn make_constant(&mut self, value: Value) -> Option<u8>;
    fn size(&self) -> usize;
    fn write_at(&mut self, offset: usize, byte: u8);
    fn jump_offset(&self, offset: usize) -> usize;
    fn clone(&self) -> Self;
}
