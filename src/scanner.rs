use crate::token::TT;
use crate::{token::*, InterpretResult};

pub struct Scanner<'a> {
    pub source: &'a str,
    pub tokens: Vec<Token>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        Token::new(TT::EndOfFile, self.line, "".to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;
}
