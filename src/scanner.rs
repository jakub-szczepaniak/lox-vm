use aho_corasick::AhoCorasick;

use crate::token::TT;
use crate::{token::*, value::Value, InterpretResult};

pub struct Scanner {
    pub source: Vec<char>,
    pub line: usize,
    start: usize,
    current: usize,
    length: usize,
    ac: AhoCorasick,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect::<Vec<char>>(),
            line: 1,
            current: 0,
            start: 0,
            length: source.len(),
            ac: AhoCorasick::new([
                "and", "class", "else", "false", "for", "fun", "if", "nil", "or", "print",
                "return", "super", "this", "true", "var", "while",
            ])
            .unwrap(),
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

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
            '"' => self.string(),
            _ if c.is_alphabetic() || c == '_' => self.identifier(),
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

    fn peek_is_alphanumeric(&self) -> bool {
        let c = self.peek();
        c.is_alphanumeric() || c == '_'
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if let Some('/') = self.peek_next() {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn identifier(&mut self) -> Token {
        while self.peek_is_alphanumeric() {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }
    fn identifier_type(&self) -> TokenType {
        let haystack = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let mut matches = Vec::new();
        for mat in self.ac.find_iter(&haystack) {
            matches.push(mat)
        }
        if matches.is_empty() {
            return TT::Identifier;
        }
        match haystack.as_str() {
            "and" => TT::And,
            "false" => TT::Constant,
            "true" => TT::Constant,
            "nil" => TT::Constant,
            "while" => TT::While,
            "for" => TT::For,
            "fun" => TT::Fun,
            "class" => TT::Class,
            "super" => TT::Super,
            "this" => TT::This,
            "or" => TT::Or,
            "else" => TT::Else,
            "print" => TT::Print,
            "return" => TT::Return,
            "var" => TT::Var,

            _ => TT::Error,
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

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error_token("Unterminated string")
        } else {
            self.advance();
            self.make_token(TT::String)
        }
    }

    fn make_token(&self, ttype: TT) -> Token {
        let mut literal: Option<Literal> = None;
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        match ttype {
            TT::Number => {
                let value: Value = lexeme.parse().unwrap();
                literal = Some(Literal::Number(value));
            }
            TT::String => {
                let work = lexeme.clone();
                let value = work.trim_matches('"');
                literal = Some(Literal::String(value.to_string()));
            }
            TT::Constant => {
                let value = lexeme.clone();
                match value.as_str() {
                    "nil" => literal = Some(Literal::Nil),
                    "true" => literal = Some(Literal::Boolean(true)),
                    "false" => literal = Some(Literal::Boolean(false)),
                    &_ => {
                        return self.error_token("Unexpected value: ");
                    }
                }
            }
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
