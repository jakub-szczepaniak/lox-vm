use clap::builder::NonEmptyStringValueParser;

use crate::token::TT;
use crate::{token::*, value::Value, InterpretResult};

pub struct Scanner {
    pub source: Vec<char>,
    pub line: usize,
    start: usize,
    current: usize,
    length: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect::<Vec<char>>(),
            line: 1,
            current: 0,
            start: 0,
            length: source.len(),
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TT::EndOfFile);
        };
        let c = self.advance();
        match c {
            '(' => self.make_token(TT::LeftParen),
            ')' => self.make_token(TT::RightParen),
            '{' => self.make_token(TT::LeftBracket),
            '}' => self.make_token(TT::RightBracket),
            ';' => self.make_token(TT::Semicolon),
            ',' => self.make_token(TT::Comma),
            '.' => self.make_token(TT::Dot),
            '-' => self.make_token(TT::Minus),
            '+' => self.make_token(TT::Plus),

            '/' => self.make_token(TT::Slash),
            '*' => self.make_token(TT::Star),
            '!' => {
                let eq = self.matching('=');
                self.make_token(if eq { TT::BangEquals } else { TT::Bang })
            }
            '=' => {
                let eq = self.matching('=');
                self.make_token(if eq { TT::Equals } else { TT::Assign })
            }
            '<' => {
                let eq = self.matching('=');
                self.make_token(if eq { TT::LessEquals } else { TT::Less })
            }
            '>' => {
                let eq = self.matching('=');
                self.make_token(if eq {
                    TT::GreaterEquals
                } else {
                    {
                        TT::Greater
                    }
                })
            }
            '0'..='9' => self.number(),

            _ => self.error_token("Unexpected token"),
        }
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn matching(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().unwrap().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.make_token(TT::Number)
    }

    fn make_token(&self, ttype: TT) -> Token {
        let mut literal: Option<Literal> = None;
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        match ttype {
            TT::Number => {
                let value: Value = lexeme.parse().unwrap();
                literal = Some(Literal::Number(value));
            }
            TT::String => {}
            _ => {}
        };
        Token {
            ttype,
            line: self.line,
            lexeme,
            literal,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token::new(
            TT::Error,
            self.line,
            format!(
                "{}: {:?}",
                message,
                self.source[self.start..self.current]
                    .iter()
                    .collect::<Vec<&char>>()
            ),
        )
    }
    fn is_at_end(&self) -> bool {
        self.current == self.length
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;
}
