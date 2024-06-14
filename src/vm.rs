use crate::{compiler::*, function::*, opcode::*, value::Value};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
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

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    function: Function,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
            function: Function::new(""),
        }
    }

    pub fn free(&self) {}

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        // need to reset the chunk here!!!
        let mut compiler: Compiler = Compiler::new();
        self.function = compiler.compile(source)?;
        #[cfg(feature = "debug_print_code")]
        if !compiler.had_error() {
            self.function.disassemble("Debug", &mut std::io::stdout());
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
                self.function
                    .disassemble_instruction(self.ip, &mut std::io::stdout());
            }
            let instruction = self.read_opcode();
            match instruction {
                OpCode::GetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(name) = constant {
                        if let Some(v) = self.globals.get(&name) {
                            self.push(v.clone())
                        } else {
                            return self.runtime_error(&format!("Undefined variable {name}"));
                        }
                    } else {
                        panic!("Unable to read constant!");
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    self.push(self.stack[slot].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack[slot] = self.peek(0).clone();
                }
                OpCode::DefineGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(k) = constant {
                        let v = self.pop();
                        self.globals.insert(k, v);
                    } else {
                        panic!("Not able to read constant from the table!")
                    }
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if self.peek(0).is_falsy() {
                        self.ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.ip += offset
                }
                OpCode::Loop => {
                    let offset: usize = self.read_short();
                    self.ip -= offset;
                }

                OpCode::SetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(var_name) = constant {
                        let value = self.peek(0).clone();
                        if let Entry::Occupied(mut entry) = self.globals.entry(var_name.clone()) {
                            *entry.get_mut() = value;
                        } else {
                            return self
                                .runtime_error(&format!("Undefined variable '{}'", &var_name));
                        }
                    } else {
                        panic!("Not able to read constant from the table!")
                    }
                }
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Print => {
                    println!("{}", self.pop());
                }
                OpCode::Pop => {
                    self.pop();
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
        let result = self.function.read(self.ip);
        self.ip += 1;
        result
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.function.read(self.ip).into();
        self.ip += 1;
        result
    }

    fn read_short(&mut self) -> usize {
        let short = self.function.jump_offset(self.ip);
        self.ip += 2;
        short
    }

    fn read_constant(&mut self) -> Value {
        let index = self.function.read(self.ip) as usize;
        self.ip += 1;
        self.function.read_constant(index)
    }

    fn runtime_error(&mut self, message: &str) -> Result<(), InterpretResult> {
        let line = self.function.read_line(self.ip - 1);

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
}
