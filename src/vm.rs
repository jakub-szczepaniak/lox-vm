use crate::chunk::*;

#[derive(PartialEq, Debug)]
pub enum InterpretResult {
    InterpretOK,
    InterpretCompilerError,
    InterpretRuntimeError,
}

pub struct VM {
    ip: usize,
}

impl VM {
    pub fn new() -> Self {
        Self { ip: 0 }
    }

    pub fn free(&self) {}

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.run(chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            chunk.disassemble_instruction(self.ip, &mut std::io::stdout());
            let instruction = self.read_opcode(chunk);
            match instruction {
                OpCode::OpReturn => return InterpretResult::InterpretOK,
                OpCode::OpConstant(v) => {
                    println!("{v}");
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
