use crate::{chunk::*, compiler::*, value::Value};
use std::fmt::Debug;
use std::io::Write;
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
        // need to reset the chunk here!!!
        self.chunk.reset();
        let mut compiler = Compiler::new(&mut self.chunk);
        compiler.compile(source)?;
        #[cfg(feature = "debug_print_code")]
        if !compiler.had_error() {
            self.chunk.disassemble("Debug", &mut std::io::stdout());
        }

        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                writeln!(&mut std::io::stdout()).unwrap();
                write!(&mut std::io::stdout(), "Stack:        ").unwrap();
                for value in &self.stack {
                    write!(&mut std::io::stdout(), "[ {value} ]").unwrap();
                }
                writeln!(&mut std::io::stdout()).unwrap();
                self.chunk
                    .disassemble_instruction(self.ip, &mut std::io::stdout());
            }
            let instruction = self.read_opcode();
            match instruction {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Print => {
                    println!("{}", self.pop());
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push(constant.clone());
                }
                OpCode::Negate => {
                    self.validate_unary()?;
                    let val = self.pop();
                    self.push(-val)
                }
                OpCode::Add => {
                    if self.operands_numbers() {
                        self.binary_op(|a, b| a + b)
                    } else if self.operands_strings() {
                        self.binary_op(|a, b| Value::Str(format!("{a}{b}")))
                    } else {
                        self.runtime_error("Both operands have to be string or number!")?
                    }
                }
                OpCode::Substract => {
                    self.validate_binary()?;
                    self.binary_op(|a, b| a - b)
                }
                OpCode::Multiply => {
                    self.validate_binary()?;
                    self.binary_op(|a, b| a * b)
                }
                OpCode::Divide => {
                    self.validate_binary()?;
                    self.divide_op()?
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),
                OpCode::Not => {
                    let value = self.pop();
                    self.push(Value::Boolean(value.is_falsy()));
                }
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a == b));
                }
                OpCode::Less => {
                    self.validate_binary()?;
                    self.binary_op(|a: Value, b: Value| Value::Boolean(a < b));
                }
                OpCode::Greater => {
                    self.validate_binary()?;
                    self.binary_op(|a: Value, b: Value| Value::Boolean(a > b));
                }
            }
        }
    }

    fn validate_unary(&mut self) -> Result<(), InterpretResult> {
        if !self.peek(0).is_number() {
            self.runtime_error("Operand must be a number")
        } else {
            Ok(())
        }
    }
    fn validate_binary(&mut self) -> Result<(), InterpretResult> {
        if !self.operands_numbers() {
            self.runtime_error("Both operands need to be numbers")
        } else {
            Ok(())
        }
    }

    fn operands_numbers(&mut self) -> bool {
        self.peek(0).is_number() && self.peek(1).is_number()
    }

    fn operands_strings(&mut self) -> bool {
        self.peek(0).is_string() && self.peek(1).is_string()
    }

    fn divide_op(&mut self) -> Result<(), InterpretResult> {
        if let Value::Number(divider) = self.peek(0) {
            if *divider == 0.0 {
                return self.runtime_error("Cannot divide by 0!");
            }
        }
        let b = self.pop();
        let a = self.pop();
        self.push(a / b);
        Ok(())
    }

    fn binary_op(&mut self, operation: fn(a: Value, b: Value) -> Value) {
        let b = self.pop();
        let a = self.pop();
        self.stack.push(operation(a, b))
    }

    fn read_opcode(&mut self) -> OpCode {
        let result = self.chunk.read(self.ip);
        self.ip += 1;
        result
    }
    fn read_constant(&mut self) -> Value {
        let index = self.chunk.read(self.ip) as usize;
        self.ip += 1;
        self.chunk.read_constant(index)
    }

    fn runtime_error(&mut self, message: &str) -> Result<(), InterpretResult> {
        let line = self.chunk.read_line(self.ip - 1);

        eprintln!("{}", message);
        eprintln!("[line {}] in script", line);
        self.reset_stack();
        Err(InterpretResult::RuntimeError)
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    fn push(&mut self, val: Value) {
        self.stack.push(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_run_the_chunk_by_vm() {
        let mut chunk = Chunk::new();
        chunk.emit_constant(Value::Number(1.0), 1);
        chunk.emit_byte(OpCode::Return.into(), 1);
        let mut vm = VM::new(chunk);

        let result = vm.run();

        assert!(result.is_ok())
    }
}
