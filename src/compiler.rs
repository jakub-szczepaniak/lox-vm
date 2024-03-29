use crate::scanner::*;

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: String) {
        let mut scanner = Scanner {
            source: &source,
            tokens: Vec::new(),
        };
        scanner.tokenize();
        for token in &scanner.tokens {
            println!("{}", token);
        }
    }
}
