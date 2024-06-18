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

struct CallFrame {
    func_index: usize,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    pub fn inc(&mut self, amount: usize) {
        self.ip += amount;
    }

    pub fn dec(&mut self, amount: usize) {
        self.ip -= amount;
    }
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
        }
    }

    pub fn free(&self) {}

    fn current(&mut self) -> &Function {
        let frame = self.frames.last().unwrap();

        if let Value::Func(f) = &self.stack[frame.func_index] {
            f
        } else {
            panic!("Could not find function !")
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        // need to reset the chunk here!!!
        let mut compiler: Compiler = Compiler::new();
        let function = compiler.compile(source)?;
        #[cfg(feature = "debug_print_code")]
        if !compiler.had_error() {
            function.disassemble(&mut std::io::stdout());
        }
        self.frames.push(CallFrame {
            func_index: 0,
            ip: 0,
            slot: 1,
        });
        self.stack.push(Value::Func(function));
        self.run()
    }
    fn ip(&self) -> usize {
        let frame = self.frames.last().unwrap();
        frame.ip
    }
    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
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
                let ip = self.ip();
                self.current()
                    .disassemble_instruction(ip, &mut std::io::stdout());
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
                    let slot_offset = self.current_frame().slot;
                    self.push(self.stack[slot + slot_offset].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let slot_offset = self.current_frame().slot;
                    self.stack[slot + slot_offset] = self.peek(0).clone();
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
                        self.current_frame().inc(offset);
                    }
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.current_frame().inc(offset)
                }
                OpCode::Loop => {
                    let offset: usize = self.read_short();
                    self.current_frame().dec(offset)
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
        let ip = self.ip();
        let result = self.current().read(ip);
        self.current_frame().inc(1);
        result
    }

    fn read_byte(&mut self) -> u8 {
        let ip = self.ip();
        let result = self.current().read(ip).into();
        self.current_frame().inc(1);
        result
    }

    fn read_short(&mut self) -> usize {
        let ip = self.ip();
        let short = self.current().jump_offset(ip);
        self.current_frame().inc(2);
        short
    }

    fn read_constant(&mut self) -> Value {
        let ip = self.ip();
        let index = self.current().read(ip) as usize;
        self.current_frame().inc(1);
        self.current().read_constant(index)
    }

    fn runtime_error(&mut self, message: &str) -> Result<(), InterpretResult> {
        let ip = self.ip();
        let line = self.current().read_line(ip - 1);

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
