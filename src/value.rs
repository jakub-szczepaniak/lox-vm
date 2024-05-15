use crate::object::Obj;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Obj(usize), // index to the location on the list of objects
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Nil => write!(f, "Nil"),
            Value::Obj(o) => write!(f, "{o}"),
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
pub struct ValueArray {
    values: Vec<Value>,
    objects: Vec<Obj>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            objects: Vec::new(),
        }
    }

    //returns the index of value that was inserted
    pub fn write(&mut self, value: Value) -> usize {
        let count = self.values.len();
        self.values.push(value);
        count
    }

    pub fn free(&mut self) {
        self.values = Vec::new();
        self.objects = Vec::new();
    }

    pub fn print_at(&self, index: usize) {
        print!("{}", self.values[index]);
    }

    pub fn read_at(&self, index: usize) -> Value {
        self.values[index]
    }
    //returns the index of the Object allocated
    pub fn make_object(&mut self, obj: Obj) -> usize {
        let ind = self.objects.len();
        self.objects.push(obj);
        ind
    }
}
