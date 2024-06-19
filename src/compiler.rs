use crate::{function::*, opcode::OpCode, scanner::*, token::*, value::Value, InterpretResult};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Local {
    name: Token,
    depth: Option<usize>,
}

#[derive(Default)]
pub struct Parser {
    previous: Token,
    current: Token,
    had_error: RefCell<bool>,
    panic_mode: RefCell<bool>,
}
#[derive(Copy)]
struct ParseRule<'a> {
    precedence: Precedence,
    prefix: Option<fn(&mut Compiler<'a>, bool)>,
    infix: Option<fn(&mut Compiler<'a>, bool)>,
}
impl<'a> Default for ParseRule<'a> {
    fn default() -> ParseRule<'a> {
        Self {
            precedence: Precedence::None,
            prefix: None,
            infix: None,
        }
    }
}
impl<'a> Clone for ParseRule<'a> {
    fn clone(&self) -> Self {
        *self
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
}

pub struct Compiler<'a> {
    function: Function,
    parser: Parser,
    scanner: Scanner,
    rules: Vec<ParseRule<'a>>,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        let mut rules = vec![
            ParseRule {
                precedence: Precedence::None,
                infix: None,
                prefix: None
            };
            40
        ];
        rules[TT::LeftParen as usize] = ParseRule {
            precedence: Precedence::None,
            prefix: Some(Compiler::grouping),
            infix: None,
        };
        rules[TT::Minus as usize] = ParseRule {
            precedence: Precedence::Term,
            prefix: Some(Compiler::unary),
            infix: Some(Compiler::unary),
        };
        rules[TT::Plus as usize] = ParseRule {
            precedence: Precedence::Term,
            prefix: None,
            infix: Some(Compiler::binary),
        };

        rules[TT::Slash as usize] = ParseRule {
            precedence: Precedence::Factor,
            prefix: None,
            infix: Some(Compiler::binary),
        };

        rules[TT::Star as usize] = ParseRule {
            precedence: Precedence::Factor,
            prefix: None,
            infix: Some(Compiler::binary),
        };

        rules[TT::Identifier as usize] = ParseRule {
            prefix: Some(Compiler::variable),
            precedence: Precedence::None,
            infix: None,
        };
        rules[TT::Number as usize] = ParseRule {
            precedence: Precedence::None,
            prefix: Some(Compiler::number),
            infix: None,
        };
        rules[TT::Constant as usize] = ParseRule {
            precedence: Precedence::None,
            prefix: Some(Compiler::literal),
            infix: None,
        };
        rules[TT::String as usize] = ParseRule {
            precedence: Precedence::None,
            prefix: Some(Compiler::string),
            infix: None,
        };
        rules[TT::Bang as usize].prefix = Some(Compiler::unary);

        rules[TT::BangEquals as usize] = ParseRule {
            precedence: Precedence::Equality,
            prefix: None,
            infix: Some(Compiler::binary),
        };
        rules[TT::Equals as usize] = rules[TT::BangEquals as usize];

        rules[TT::Greater as usize] = ParseRule {
            prefix: None,
            precedence: Precedence::Comparison,
            infix: Some(Compiler::binary),
        };
        rules[TT::GreaterEquals as usize] = rules[TT::Greater as usize];
        rules[TT::Less as usize] = rules[TT::Greater as usize];
        rules[TT::LessEquals as usize] = rules[TT::Greater as usize];

        rules[TT::And as usize] = ParseRule {
            prefix: None,
            precedence: Precedence::And,
            infix: Some(Compiler::and),
        };

        rules[TT::Or as usize] = ParseRule {
            prefix: None,
            precedence: Precedence::Or,
            infix: Some(Compiler::or),
        };

        Self {
            parser: Parser::default(),
            function: Function::new("<script>"),
            scanner: Scanner::new(""),
            rules,
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn had_error(&self) -> bool {
        *self.parser.had_error.borrow()
    }

    pub fn compile(&mut self, source: &str) -> Result<Function, InterpretResult> {
        self.initialize();
        self.scanner = Scanner::new(source);
        self.advance();

        while !self.is_match(TT::EndOfFile) {
            self.declaration();
        }

        self.end_compiler();
        self.consume(TT::EndOfFile, "Expected end of expression");
        self.finalize();

        if self.had_error() {
            Err(InterpretResult::CompilerError)
        } else {
            Ok(self.function.clone())
        }
    }

    fn initialize(&mut self) {
        self.function.initialize_emiter()
    }

    fn finalize(&mut self) {
        self.function.finalize_emiter();
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
        if self.is_match(TT::Fun) {
            self.fun_declaration();
        } else if self.is_match(TT::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.had_error() {
            self.synchronize();
        }
    }
    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expected function name");
        self.mark_initialized();
        self.function();
        self.define_variable(global);
    }

    fn function(&mut self) {}

    fn block(&mut self) {
        while !self.check(TT::RightBracket) && !self.check(TT::EndOfFile) {
            self.declaration();
        }

        self.consume(TT::RightBracket, "Expect '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        let mut index = self.locals.len();
        while index > 0 {
            index -= 1;
            if self.locals[index].depth.unwrap() > self.scope_depth {
                self.emit_byte(OpCode::Pop.into());
                self.locals.remove(index);
            }
        }
    }

    fn statement(&mut self) {
        if self.is_match(TT::Print) {
            self.print_statement();
        } else if self.is_match(TT::If) {
            self.if_statement();
        } else if self.is_match(TT::While) {
            self.while_statement();
        } else if self.is_match(TT::For) {
            self.for_statement();
        } else if self.is_match(TT::LeftBracket) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.is_match(TT::Assign) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(TT::Semicolon, "Expect ';' after expression.");

        self.define_variable(global);
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TT::Identifier, error_message);

        self.declare_variable();
        if self.scope_depth == 0 {
            let name = self.parser.previous.lexeme.clone();
            self.identifier_constant(&name)
        } else {
            0
        }
    }

    fn declare_variable(&mut self) {
        if self.scope_depth != 0 {
            let name = self.parser.previous.lexeme.clone();
            for x in self.locals.iter().rev() {
                if x.depth.unwrap() < self.scope_depth {
                    break;
                }
                if x.name.lexeme == name {
                    self.error("Already declared variable with this name in the current scope.");
                    return;
                }
            }

            let token = self.parser.previous.clone();
            self.add_local(token);
        }
    }

    fn add_local(&mut self, token: Token) {
        if self.locals.len() >= 256 {
            self.error("Too many local variables in the scope!");
            return;
        }
        let local = Local {
            name: token,
            depth: None,
        };
        self.locals.push(local);
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(&self.parser.previous.lexeme.clone(), can_assign);
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let (index, get_op, set_op) = if let Some(local_arg) = self.resolve_local(name) {
            (local_arg, OpCode::GetLocal, OpCode::SetLocal)
        } else {
            (
                self.identifier_constant(name),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            )
        };

        if can_assign && self.is_match(TT::Assign) {
            self.expression();
            self.emit_bytes(set_op, index);
        } else {
            self.emit_bytes(get_op, index)
        }
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        for (index, value) in self.locals.iter().rev().enumerate() {
            if value.name.lexeme == name {
                if value.depth.is_none() {
                    self.error("Cannot read local variable in its own initializer.")
                }
                return Some((self.locals.len() - 1 - index) as u8);
            }
        }
        None
    }

    fn identifier_constant(&mut self, name: &str) -> u8 {
        self.function.make_constant(Value::Str(name.to_string()))
    }

    fn define_variable(&mut self, index: u8) {
        if self.scope_depth == 0 {
            self.emit_bytes(OpCode::DefineGlobal, index);
        } else {
            self.mark_initialized();
        }
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let index = self.locals.len() - 1;
        self.locals[index].depth = Some(self.scope_depth);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TT::Semicolon, "Expected ';' after the value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn if_statement(&mut self) {
        self.consume(TT::LeftParen, "Expect '(' after 'if'!");
        self.expression();
        self.consume(TT::RightParen, "Expect ')' after condition");

        let jump_to = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop.into());

        self.statement();

        let else_to: usize = self.emit_jump(OpCode::Jump);

        self.finish_jump(jump_to);

        self.emit_byte(OpCode::Pop.into());

        if self.is_match(TT::Else) {
            self.statement();
        }
        self.finish_jump(else_to);
    }

    fn while_statement(&mut self) {
        let loop_start: usize = self.function.size();
        self.consume(TT::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TT::RightParen, "Expected ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop.into());
        self.statement();
        self.emit_loop(loop_start);

        self.finish_jump(exit_jump);
        self.emit_byte(OpCode::Pop.into());
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TT::LeftParen, "Expect '(' after 'for'");
        if self.is_match(TT::Semicolon) {
        } else if self.is_match(TT::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }
        let mut loop_start: usize = self.function.size();
        let exit_jump = if self.is_match(TT::Semicolon) {
            None
        } else {
            self.expression();
            self.consume(TT::Semicolon, "Expect ';' after the condition.");
            let result = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::Pop.into());
            Some(result)
        };

        if !self.is_match(TT::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.function.size();

            self.expression();
            self.emit_byte(OpCode::Pop.into());

            self.consume(TT::RightParen, "Expect ')' after for clauses.");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.finish_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(exit) = exit_jump {
            self.finish_jump(exit);
            self.emit_byte(OpCode::Pop.into());
        }
        self.end_scope();
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop.into());
        let offset: usize = self.function.size() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large")
        }

        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit_byte(opcode as u8);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.function.size() - 2
    }

    fn finish_jump(&mut self, offset: usize) {
        let jump = self.function.size() - offset - 2;
        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }
        self.function.write_at(offset, ((jump >> 8) & 0xff) as u8);
        self.function.write_at(offset + 1, (jump & 0xff) as u8);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop.into());
        self.parse_precendence(Precedence::And);
        self.finish_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.finish_jump(else_jump);
        self.emit_byte(OpCode::Pop.into());

        self.parse_precendence(Precedence::Or);

        self.finish_jump(end_jump);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TT::Semicolon, "Expected ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }
    fn synchronize(&mut self) {
        self.parser.panic_mode.replace(false);
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
        if *self.parser.panic_mode.borrow() {
            return;
        }
        self.parser.panic_mode.replace(true);

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

    fn number(&mut self, _can_assign: bool) {
        if let Some(literal) = &self.parser.previous.literal {
            match literal {
                Literal::Number(v) => self.emit_constant(Value::Number(*v)),
                _ => unreachable!("should not happen"),
            }
        }
    }
    fn literal(&mut self, _can_assign: bool) {
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

    fn string(&mut self, _can_assign: bool) {
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

    fn binary(&mut self, _can_assign: bool) {
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

    fn unary(&mut self, _can_assign: bool) {
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
            let can_assign: bool = precedence <= Precedence::Assignment;
            prefix_rule(self, can_assign);

            while precedence <= self.rules[self.parser.current.ttype as usize].precedence {
                self.advance();
                if let Some(infix_rule) = &self.rules[self.parser.previous.ttype as usize].infix {
                    infix_rule(self, can_assign);
                }

                if can_assign && self.is_match(TT::Assign) {
                    self.error("Invalid assignment target.");
                }
            }
        } else {
            self.error("Expected expression");
        }
    }

    fn expression(&mut self) {
        self.parse_precendence(Precedence::Assignment)
    }

    fn grouping(&mut self, _can_assign: bool) {
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
        self.function.emit_byte(byte, self.parser.previous.line)
    }

    fn emit_constant(&mut self, val: Value) {
        self.function.emit_constant(val, self.parser.previous.line)
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
