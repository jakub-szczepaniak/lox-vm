use crate::{chunk::*, value::Value};
use std::io::*;

#[derive(PartialEq, Debug)]
pub enum InterpretResult {
    InterpretOK,
    InterpretCompilerError,
    InterpretRuntimeError,
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

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.run(chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
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
                OpCode::OpReturn => {
                    println!("{}", self.stack.pop().unwrap());
                    return InterpretResult::InterpretOK;
                }
                OpCode::OpConstant(v) => {
                    self.stack.push(v);
                }
            }
        }
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
        chunk.write_opcode(OpCode::OpReturn, 1);

        let result = vm.run(&chunk);

        assert_eq!(result, InterpretResult::InterpretOK)
    }
}
