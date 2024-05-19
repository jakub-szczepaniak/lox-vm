use crate::{chunk::*, scanner::*, token::*, value::Value, InterpretResult, OpCode};
use std::cell::RefCell;

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
    had_error: RefCell<bool>,
}
#[derive(Copy)]
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
            prefix: self.prefix,
            infix: self.infix,
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
        rules[TT::Constant as usize] = ParseRule::<T> {
            precedence: Precedence::None,
            prefix: Some(|c| c.literal()),
            infix: None,
        };
        rules[TT::String as usize] = ParseRule::<T> {
            precedence: Precedence::None,
            prefix: Some(|c| c.string()),
            infix: None,
        };
        rules[TT::Bang as usize].prefix = Some(|c| c.unary());

        rules[TT::BangEquals as usize] = ParseRule::<T> {
            precedence: Precedence::Equality,
            prefix: None,
            infix: Some(|c| c.binary()),
        };
        rules[TT::Equals as usize] = rules[TT::BangEquals as usize].clone();

        rules[TT::Greater as usize] = ParseRule::<T> {
            prefix: None,
            precedence: Precedence::Comparison,
            infix: Some(|c| c.binary()),
        };
        rules[TT::GreaterEquals as usize] = rules[TT::Greater as usize].clone();
        rules[TT::Less as usize] = rules[TT::Greater as usize].clone();
        rules[TT::LessEquals as usize] = rules[TT::Greater as usize].clone();

        Self {
            parser: Parser::default(),
            chunk,
            scanner: Scanner::new(""),
            rules,
        }
    }

    pub fn had_error(&self) -> bool {
        *self.parser.had_error.borrow()
    }

    pub fn compile(&mut self, source: &str) -> Result<(), InterpretResult> {
        self.initialize();
        self.scanner = Scanner::new(source);
        self.advance();

        while !self.is_match(TT::EndOfFile) {
            self.declaration();
        }

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

    fn declaration(&mut self) {
        self.statement();

        if self.had_error() {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.is_match(TT::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TT::Semicolon, "Expected ';' after the value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TT::Semicolon, "Expected ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }
    fn synchronize(&mut self) {
        self.parser.had_error.replace(false);
        while self.parser.current.ttype != TT::EndOfFile {
            if self.parser.previous.ttype == TT::Semicolon {
                return;
            }
            if matches!(
                self.parser.current.ttype,
                TT::Class
                    | TT::Fun
                    | TT::Var
                    | TT::For
                    | TT::If
                    | TT::While
                    | TT::Print
                    | TT::Return
            ) {
                return;
            }

            self.advance();
        }
    }
    fn check(&self, ttype: TT) -> bool {
        self.parser.current.ttype == ttype
    }

    fn is_match(&mut self, ttype: TT) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
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
        if let Some(literal) = &self.parser.previous.literal {
            match literal {
                Literal::Number(v) => self.emit_constant(Value::Number(*v)),
                _ => unreachable!("should not happen"),
            }
        }
    }
    fn literal(&mut self) {
        if let Some(literal) = &self.parser.previous.literal {
            match literal {
                Literal::Boolean(b) => {
                    if *b {
                        self.emit_byte(OpCode::True.into())
                    } else {
                        self.emit_byte(OpCode::False.into())
                    }
                }
                Literal::Nil => self.emit_byte(OpCode::Nil.into()),
                _ => unreachable!("Should not happen!"),
            }
        }
    }

    fn string(&mut self) {
        if let Some(literal) = &self.parser.previous.literal {
            match literal {
                Literal::String(s) => {
                    let str = Value::Str(s.to_string());
                    self.emit_constant(str);
                }
                _ => unreachable!("Should not happen"),
            }
        }
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
            TT::Equals => self.emit_byte(OpCode::Equal.into()),
            TT::Less => self.emit_byte(OpCode::Less.into()),
            TT::Greater => self.emit_byte(OpCode::Greater.into()),
            TT::BangEquals => self.emit_bytes(OpCode::Equal, OpCode::Not.into()),
            TT::GreaterEquals => self.emit_bytes(OpCode::Greater, OpCode::Not.into()),
            TT::LessEquals => self.emit_bytes(OpCode::Less, OpCode::Not.into()),

            _ => unreachable!("Should not be here!"),
        }
    }

    fn unary(&mut self) {
        let operator = self.parser.previous.ttype;
        self.parse_precendence(Precedence::Unary);

        match operator {
            TT::Minus => self.emit_byte(OpCode::Negate.into()),
            TT::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unreachable!("Should not happen!"),
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
