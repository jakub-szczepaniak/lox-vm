use crate::{scanner::*, token::*, InterpretResult};

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: &str) -> Result<(), InterpretResult> {
        let mut scanner = Scanner::new(source);
        let mut line = 0;
        loop {
            let token = scanner.scan_token();
            if token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!(
                "{:10} '{}' '{}'",
                token.ttype,
                token.lexeme,
                token.literal.unwrap_or(Literal::Nil)
            );
            if token.ttype == TT::EndOfFile {
                break;
            }
        }
        Ok(())
    }
}
