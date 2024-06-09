use crate::{chunk::*, Emmitable};

use std::fmt::Display;
#[derive(Debug)]
struct Function<T: Emmitable + OpCodable> {
    arity: u8,
    name: String,
    chunk: Option<T>,
}

impl<T: Emmitable + OpCodable> Display for Function<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl<T: Emmitable + OpCodable> PartialEq for Function<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T: Emmitable + OpCodable> PartialOrd for Function<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        panic!("Cannot compare 2 functions")
    }
}

impl<T: Emmitable + OpCodable> Clone for Function<T> {
    fn clone(&self) -> Self {
        panic!("Should not clone the function!");
    }
}

impl<T: Emmitable + OpCodable> Function<T> {
    pub fn new(name: &str) -> Self {
        Self {
            arity: 0,
            name: name.to_string(),
            chunk: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn implements_display() {
        let func: Function<Chunk> = Function::new("my function");
        assert_eq!(func.to_string(), "<fn my function>");
    }
    #[rstest]
    fn implements_partial_equality_false_for_different() {
        let first: Function<Chunk> = Function::new("first");
        let second: Function<Chunk> = Function::new("second");
        assert_ne!(first, second);
    }

    #[rstest]
    fn implements_partial_equality_false_for_same() {
        let first: Function<Chunk> = Function::new("name");
        let second: Function<Chunk> = Function::new("name");
        assert_ne!(first, second)
    }

    #[rstest]
    #[should_panic]
    fn implements_partial_order_always_panics() {
        let first: Function<Chunk> = Function::new("first");
        let second: Function<Chunk> = Function::new("second");

        assert!(first < second);
        assert!(first <= second);
        assert!(first > second);
        assert!(first >= second);
    }

    #[rstest]
    #[should_panic]
    fn implements_clone() {
        let first: Function<Chunk> = Function::new("name");
        let _cloned = first.clone();
    }
}
