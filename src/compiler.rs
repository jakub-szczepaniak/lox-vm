use crate::{chunk::Chunk, scanner::*, token::*, Emmitable, InterpretResult};
use std::cell::RefCell;

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
    had_error: RefCell<bool>,
}

pub struct Compiler<'a, T: Emmitable> {
    chunk: &'a T,
    parser: Parser,
}

impl<'a, T: Emmitable> Compiler<'a, T> {
    pub fn new(chunk: &'a mut T) -> Self {
        Self {
            parser: Parser::default(),
            chunk,
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.consume(TT::EndOfFile, "Expected end of expression");
        Ok(())
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = self.scan_token();
            if self.parser.current.ttype != TT::Error {
                break;
            }
            let message = self.parser.current.lexeme.as_str();
            self.error_at_current(message);
        }
    }

    fn scan_token(&mut self) -> Token {
        self.parser.previous.clone()
    }

    fn error_at_current(&self, message: &str) {
        let current: &Token = &self.parser.current.clone();
        self.error_at(current, message)
    }

    fn error(&self, message: &str) {
        self.error_at(&self.parser.previous.clone(), message)
    }

    fn error_at(&self, token: &Token, message: &str) {
        eprint!("[line {}] Error", token.line);
        match token.ttype {
            TT::EndOfFile => {
                eprint!(" at end");
            }
            TT::Error => {}
            _ => {
                eprint!(" at {}", token.lexeme)
            }
        }
        eprintln!(" : {}", message);
        self.parser.had_error.replace(true);
    }

    fn expression(&self) {}

    fn consume(&self, ttype: TT, message: &str) {}
}
