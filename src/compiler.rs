use crate::{scanner::*, InterpretResult};

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: String) -> Result<(), InterpretResult> {
        let mut scanner = Scanner::new(&source);
        scanner.tokenize()?;
        for token in &scanner.tokens {
            println!("{}", token);
        }
        Ok(())
    }
}
