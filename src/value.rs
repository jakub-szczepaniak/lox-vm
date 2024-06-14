use crate::function::*;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Str(String),
    Func(Function),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Number(n) => Value::Number(*n),
            Value::Nil => Value::Nil,
            Value::Str(s) => Value::Str(s.clone()),
            Value::Func(f) => Value::Func(f.clone()),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Str(_))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Nil => write!(f, "Nil"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Func(fu) => write!(f, "fn {}", fu.name),
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => panic!("Invalid operation!"),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Invalid operation!"),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Invalid operation!"),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Invalid operation!"),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(a) => Value::Number(-a),
            _ => panic!("Invalid operation!"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    //returns the index of value that was inserted
    pub fn write(&mut self, value: Value) -> usize {
        let count = self.values.len();
        self.values.push(value);
        count
    }

    pub fn free(&mut self) {
        self.values = Vec::new();
    }

    pub fn read_at(&self, index: usize) -> Value {
        self.values[index].clone()
    }
}
