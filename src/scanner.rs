use clap::builder::Str;
use nom::{character::complete::char, multi::many0};
use std::{fmt::Display, net::ToSocketAddrs};

#[derive(PartialEq, Debug)]
pub enum TokenType {
    EndOfFile,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
pub struct Token {
    ttype: TokenType,
    line: usize,
    lexeme: String,
}

impl Token {
    fn new(ttype: TokenType, line: usize, lexeme: &str) -> Self {
        Self {
            ttype,
            line,
            lexeme: lexeme.to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}:{}", self.line, self.ttype, self.lexeme)
    }
}

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) {
        self.tokens.push(Token::new(TokenType::EndOfFile, 0, ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_empty_line_eof_token() {
        let mut scanner = Scanner::new("".to_string());

        scanner.tokenize();
        assert_eq!(scanner.tokens[0].ttype, TokenType::EndOfFile)
    }
}
