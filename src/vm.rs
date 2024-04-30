use crate::{chunk::*, compiler::*, value::Value};

use std::fmt::Debug;
use thiserror::*;
#[derive(thiserror::Error, PartialEq)]
pub enum InterpretResult {
    //InterpretOK,
    #[error("Fail to compile source code")]
    CompilerError,
    #[error("Runtime error")]
    RuntimeError,
}

impl Debug for InterpretResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        Ok(())
    }
}

pub struct VM<T: Emmitable + OpCodable> {
    ip: usize,
    stack: Vec<Value>,
    chunk: T,
}

impl<T: Emmitable + OpCodable> VM<T> {
    pub fn new(t: T) -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            chunk: t,
        }
    }

    pub fn free(&self) {}

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut compiler = Compiler::new(&mut self.chunk);

        compiler.compile(source)?;

        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                writeln!(&mut std::io::stdout(), "       ").unwrap();
                for value in &self.stack {
                    writeln!(&mut std::io::stdout(), "[ {value} ]").unwrap();
                }
                self.chunk
                    .disassemble_instruction(self.ip, &mut std::io::stdout());
            }
            let instruction = self.read_opcode();
            match instruction {
                OpCode::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(-val)
                }
                OpCode::Add => self.binary_op(|a, b| a + b),
                OpCode::Substract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide => self.divide_op()?,
            }
        }
    }

    fn divide_op(&mut self) -> Result<(), InterpretResult> {
        let b = self.stack.pop().unwrap();
        match b {
            0.0 => Err(InterpretResult::RuntimeError),
            _ => {
                let a = self.stack.pop().unwrap();
                self.stack.push(a / b);
                Ok(())
            }
        }
    }

    fn binary_op(&mut self, operation: fn(a: Value, b: Value) -> Value) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(operation(a, b))
    }

    fn read_opcode(&mut self) -> OpCode {
        let result = self.chunk.read(self.ip);
        self.ip += 1;
        result
    }
    fn read_constant(&mut self) -> Value {
        let index = self.chunk.read(self.ip) as usize;
        self.ip +=1;
        self.chunk.read_constant(index)
    }
}

pub trait Emmitable {
    fn emit_byte(&mut self, byte: u8, line: usize);
    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8, line: usize);
    fn emit_constant(&mut self, value: Value, line: usize);
    fn initialize_emiter(&mut self) {}
    fn finalize_emiter(&mut self) {}
}

pub trait OpCodable {
    fn read(&self, ip: usize) -> OpCode;
    fn read_constant(&self, index: usize) -> Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_run_the_chunk_by_vm() {
        let mut chunk = Chunk::new();
        chunk.write_opcode(OpCode::Constant, 1);
        let const_index = chunk.add_constant(1.0).unwrap();
        chunk.write(const_index, 1);
        chunk.write_opcode(OpCode::Return, 1);
        let mut vm = VM::new(chunk);

        let result = vm.run();

        assert!(result.is_ok())
    }

}
