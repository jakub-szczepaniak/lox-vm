use clap::builder::Str;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, multispace0},
    combinator::map,
    error::Error,
    multi::many0,
    IResult,
};
use std::fmt::Display;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    EndOfFile,
    Plus,
    Minus,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::EndOfFile => write!(f, "EndOfFile"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            _ => write!(f, ""),
        }
    }
}
#[derive(Clone)]
pub struct Token {
    ttype: TokenType,
    line: usize,
    lexeme: String,
}

impl Token {
    fn new(ttype: TokenType, line: usize, lexeme: String) -> Self {
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

pub struct Scanner<'a> {
    pub source: &'a str,
    pub tokens: Vec<Token>,
}

impl Scanner<'_> {
    pub fn tokenize(&mut self) {
        let result: IResult<&str, Token> = alt((
            Scanner::single_char('+', TokenType::Plus),
            Scanner::single_char('-', TokenType::Minus),
        ))(self.source);

        match result {
            Ok((_, token)) => self.tokens.push(token),
            Err(_e) => {}
        }
        self.tokens
            .push(Token::new(TokenType::EndOfFile, 0, "".to_string()))
    }
    fn single_char(single: char, ttype: TokenType) -> impl Fn(&str) -> IResult<&str, Token> {
        move |input| {
            let (input, _) = char(single)(input)?;
            Ok((input, Token::new(ttype, 0, format!("{}", single))))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;

    #[rstest]
    #[case::test_empty_line_eof_token("", TokenType::EndOfFile)]
    #[case::test_plus_token("+", TokenType::Plus)]
    #[case::test_minus_token("-", TokenType::Minus)]
    fn test_single_char_tokens(#[case] line: &str, #[case] expected: TokenType) {
        let mut scanner = Scanner {
            source: line,
            tokens: Vec::new(),
        };

        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, expected)
    }
}
