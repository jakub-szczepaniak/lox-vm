use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace0},
    combinator::map,
    error::Error,
    multi::many0,
    IResult,
};
use std::fmt::Display;

macro_rules! operand_token {
    ($func_name: ident, $lexeme: literal, $output: expr) => {
        fn $func_name(input: &str) -> IResult<&str, Token> {
            map(tag($lexeme), |_| {
                Token::new($output, 1, $lexeme.to_string())
            })(input)
        }
    };
}

operand_token!(plus_operator, "+", TokenType::Plus);
operand_token!(minus_operator, "-", TokenType::Minus);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    EndOfFile,
    Plus,
    Minus,
}

impl Display for TT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TT::EndOfFile => write!(f, "EndOfFile"),
            TT::Plus => write!(f, "Plus"),
            TT::Minus => write!(f, "Minus"),
            _ => write!(f, ""),
        }
    }
}
#[derive(Clone)]
pub struct Token {
    ttype: TT,
    line: usize,
    lexeme: String,
}

impl Token {
    fn new(ttype: TT, line: usize, lexeme: String) -> Self {
        Self {
            ttype,
            line,
            lexeme,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}:{}", self.line, self.ttype, self.lexeme)
    }
}

use TokenType as TT;

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

    pub fn tokenize(&mut self) {
        let result: IResult<&str, Vec<Token>> =
            many0(alt((plus_operator, minus_operator)))(self.source);

        match result {
            Ok((_, token)) => self.tokens = token,
            Err(_e) => {}
        }
        self.tokens
            .push(Token::new(TT::EndOfFile, self.line, "".to_string()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;

    #[rstest]
    #[case::test_empty_line_eof_token("", TT::EndOfFile)]
    #[case::test_plus_token("+", TT::Plus)]
    #[case::test_minus_token("-", TT::Minus)]
    fn test_single_char_tokens(#[case] line: &str, #[case] expected: TT) {
        let mut scanner = Scanner::new(line);

        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, expected)
    }
}
