pub type Value = f64;

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

    pub fn print_at(&self, index: usize) {
        print!("{}", self.values[index]);
    }

    pub fn read_at(&self, index: usize) -> Value {
        self.values[index]
    }
}
