use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space0},
    combinator::map,
    multi::many0,
    sequence::delimited,
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

operand_token!(plus_operator, "+", TT::Plus);
operand_token!(minus_operator, "-", TT::Minus);
operand_token!(l_paren_operator, "(", TT::Left_Paren);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    EndOfFile,
    Plus,
    Minus,
    Left_Paren,
}

impl Display for TT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TT::EndOfFile => write!(f, "EndOfFile"),
            TT::Plus => write!(f, "Plus"),
            TT::Minus => write!(f, "Minus"),
            TT::Left_Paren => write!(f, "LeftParen"),
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
        let result: IResult<&str, Vec<Token>> = many0(delimited(
            space0,
            alt((plus_operator, minus_operator, l_paren_operator)),
            space0,
        ))(self.source);

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

    use std::iter::Scan;

    use super::*;
    use rstest::*;

    #[rstest]
    #[case::test_empty_line_eof_token("", TT::EndOfFile)]
    #[case::test_plus_token("+", TT::Plus)]
    #[case::test_minus_token("-", TT::Minus)]
    #[case::test_left_paren_token("(", TT::Left_Paren)]
    fn test_single_char_tokens(#[case] line: &str, #[case] expected: TT) {
        let mut scanner = Scanner::new(line);

        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, expected)
    }
    #[rstest]
    #[case::test_ignore_leading_whitespace(" +", TT::Plus)]
    #[case::test_ignore_trailing_whitespace("+   ", TT::Plus)]
    fn test_ignore_whitespace_in_input(#[case] line: &str, #[case] expected: TT) {
        let mut scanner = Scanner::new(line);
        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, expected)
    }

    #[rstest]
    fn test_ignore_whitespace_between_input() {
        let mut scanner = Scanner::new("+ -");
        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, TT::Plus);
        assert_eq!(scanner.tokens[1].ttype, TT::Minus);
        assert_eq!(scanner.tokens[2].ttype, TT::EndOfFile)
    }
}
