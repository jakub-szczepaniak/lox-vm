use crate::{chunk::*, scanner::*, token::*, value::Value, InterpretResult, OpCode};
use std::cell::RefCell;

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
    had_error: RefCell<bool>,
}
struct ParseRule<'a, T>
where
    T: Emmitable,
{
    precedence: Precedence,
    prefix: Option<fn(&mut Compiler<'a, T>)>,
    infix: Option<fn(&mut Compiler<'a, T>)>,
}
impl<'a, T: Emmitable> Default for ParseRule<'a, T> {
    fn default() -> ParseRule<'a, T> {
        Self {
            precedence: Precedence::None,
            prefix: None,
            infix: None,
        }
    }
}
impl<'a, T: Emmitable> Clone for ParseRule<'a, T> {
    fn clone(&self) -> Self {
        Self {
            precedence: self.precedence,
            prefix: self.prefix.clone(),
            infix: self.infix.clone(),
        }
    }
}

#[derive(PartialEq, Copy, Clone, PartialOrd)]
enum Precedence {
    None = 0,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => panic!("Should never happen!"),
        }
    }
}

impl Precedence {
    fn next(self) -> Precedence {
        if self == Precedence::Primary {
            panic!("No next precedence available!")
        }
        let precedence = self as u8;
        (precedence + 1).into()
    }

    fn previous(self) -> Precedence {
        if self == Precedence::None {
            panic!("No previous precedence available")
        }
        let precedence = self as u8;
        (precedence - 1).into()
    }
}

pub struct Compiler<'a, T: Emmitable> {
    chunk: &'a mut T,
    parser: Parser,
    scanner: Scanner,
    rules: Vec<ParseRule<'a, T>>,
}

impl<'a, T: Emmitable> Compiler<'a, T> {
    pub fn new(chunk: &'a mut T) -> Self {
        // todo! = when all parse rules are defined, we do not need to insert at the index anymore.
        let mut rules = vec![
            ParseRule::<T> {
                precedence: Precedence::None,
                infix: None,
                prefix: None
            };
            40
        ];
        rules[TT::LeftParen as usize] = ParseRule::<T> {
            precedence: Precedence::None,
            prefix: Some(|c| c.grouping()),
            infix: None,
        };
        rules[TT::Minus as usize] = ParseRule::<T> {
            precedence: Precedence::Term,
            prefix: Some(|c| c.unary()),
            infix: Some(|c| c.binary()),
        };
        rules[TT::Plus as usize] = ParseRule::<T> {
            precedence: Precedence::Term,
            prefix: None,
            infix: Some(|c| c.binary()),
        };

        rules[TT::Slash as usize] = ParseRule::<T> {
            precedence: Precedence::Factor,
            prefix: None,
            infix: Some(|c| c.binary()),
        };

        rules[TT::Star as usize] = ParseRule::<T> {
            precedence: Precedence::Factor,
            prefix: None,
            infix: Some(|c| c.binary()),
        };

        rules[TT::Number as usize] = ParseRule::<T> {
            precedence: Precedence::None,
            prefix: Some(|c| c.number()),
            infix: None,
        };
        Self {
            parser: Parser::default(),
            chunk,
            scanner: Scanner::new(""),
            rules,
        }
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult> {
        self.initialize();
        self.scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.end_compiler();
        self.consume(TT::EndOfFile, "Expected end of expression");
        self.finalize();
        Ok(())
    }

    fn initialize(&mut self) {
        self.chunk.initialize_emiter()
    }

    fn finalize(&mut self) {
        self.chunk.finalize_emiter();
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();
        loop {
            self.parser.current = self.scanner.scan_token();

            if self.parser.current.ttype != TT::Error {
                break;
            }
            let message = self.parser.current.lexeme.as_str();
            self.error_at_current(message);
        }
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

    fn binary(&mut self) {
        let op_type = self.parser.previous.ttype;
        let rule = &self.rules[op_type as usize];
        self.parse_precendence(rule.precedence.next());

        match op_type {
            TT::Plus => self.emit_byte(OpCode::Add.into()),
            TT::Minus => self.emit_byte(OpCode::Substract.into()),
            TT::Star => self.emit_byte(OpCode::Multiply.into()),
            TT::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => unreachable!("Should not be here!"),
        }
    }

    fn unary(&mut self) {
        let operator = self.parser.previous.ttype;
        self.parse_precendence(Precedence::Unary);

        if operator == TT::Minus {
            self.chunk
                .emit_byte(OpCode::Negate.into(), self.parser.previous.line)
        } else {
            unreachable!("should not happen")
        }
    }

    fn parse_precendence(&mut self, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = &self.rules[self.parser.previous.ttype as usize].prefix {
            prefix_rule(self);

            while precedence <= self.rules[self.parser.current.ttype as usize].precedence {
                self.advance();
                if let Some(infix_rule) = &self.rules[self.parser.previous.ttype as usize].infix {
                    infix_rule(self);
                }
            }
        } else {
            self.error("Expected expression");
        }
    }

    fn expression(&mut self) {
        self.parse_precendence(Precedence::Assignment)
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TT::RightParen, "Expected ')' after expression.")
    }

    fn consume(&mut self, ttype: TT, message: &str) {
        if self.parser.current.ttype == ttype {
            self.advance();
            return;
        }

        self.error_at_current(message)
    }

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
