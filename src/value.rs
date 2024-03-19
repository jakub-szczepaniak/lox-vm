use std::io::Write;

pub type Value = f64;

pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        ValueArray { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn free(&mut self) {
        self.values = Vec::new()
    }

    pub fn print_value(&self, output: &mut impl Write, index: usize) {
        write!(output, "{}", self.values[index]).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;

    #[rstest]
    fn test_write_value_to_value_array() {
        let mut va = ValueArray::new();

        va.write(12.0);
        assert_eq!(va.len(), 1)
    }

    #[rstest]
    fn test_free_the_value_array() {
        let mut va = ValueArray::new();
        va.write(1.0);
        va.free();
        assert_eq!(va.len(), 0)
    }
    #[rstest]
    fn test_print_value_by_index() {
        let mut output = Vec::new();
        let mut va = ValueArray::new();
        va.write(1.2);
        va.print_value(&mut output, 0);

        assert_eq!(output, b"1.2")
    }
}
