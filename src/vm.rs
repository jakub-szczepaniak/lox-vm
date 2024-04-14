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

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn free(&self) {}

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(&mut chunk);

        compiler.compile(source)?;

        self.ip = 0;
        self.run(&chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretResult> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                writeln!(&mut std::io::stdout(), "       ").unwrap();
                for value in &self.stack {
                    writeln!(&mut std::io::stdout(), "[ {value} ]").unwrap();
                }
                chunk.disassemble_instruction(self.ip, &mut std::io::stdout());
            }
            let instruction = self.read_opcode(chunk);
            match instruction {
                OpCode::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
                OpCode::Constant(v) => {
                    self.stack.push(v);
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

    fn read_opcode(&mut self, chunk: &Chunk) -> OpCode {
        let result = chunk.read(self.ip);
        self.ip += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_run_the_chunk_by_vm() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.add_constant(1.0, 1);
        chunk.write_opcode(OpCode::Return, 1);

        let result = vm.run(&chunk);

        assert!(result.is_ok())
    }

    #[rstest]
    fn test_aritmethic_operations_on_stack() {
        let mut vm = VM::new();
        let mut chunk = Chunk::new();
        chunk.add_constant(12.0, 1);
        chunk.write_opcode(OpCode::Negate, 2);
    }
}
