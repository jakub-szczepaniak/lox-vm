use crate::{chunk::Chunk, scanner::*, token::*, InterpretResult};

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
}

pub struct Compiler<'a> {
    chunk: &'a Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Self { chunk }
    }

    pub fn compile(&self, source: &str) -> Result<(), InterpretResult> {
        let mut scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.consume(TT::EndOfFile, "Expected end of expression");
        Ok(())
    }

    fn advance(&self) {}
    fn expression(&self) {}

    fn consume(&self, ttype: TT, message: &str) {}
}
