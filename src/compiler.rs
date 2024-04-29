use crate::{chunk::Chunk, scanner::*, token::*, value::Value, Emmitable, InterpretResult, OpCode};
use std::{cell::RefCell, io::LineWriter};

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
    had_error: RefCell<bool>,
}

pub struct Compiler<'a, T: Emmitable> {
    chunk: &'a mut T,
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
        self.chunk.initialize_emiter();

        let mut scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.end_compiler();
        self.consume(TT::EndOfFile, "Expected end of expression");
        self.chunk.finalize_emiter();
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

    fn number(&mut self) {
        let value = self.parser.previous.lexeme.parse::<Value>();
        self.emit_constant(value.ok().unwrap())
    }

    fn expression(&self) {}

    fn consume(&self, ttype: TT, message: &str) {}

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.emit_byte(byte, self.parser.previous.line)
    }

    fn emit_constant(&mut self, val: Value) {
        self.chunk.emit_constant(val, self.parser.previous.line)   
    }



    fn emit_bytes(&mut self, byte1: OpCode, byte2: u8) {
        self.emit_byte(byte1.into());
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into())
    }

    fn end_compiler(&mut self) {
        self.emit_return()
    }

}
