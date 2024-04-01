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
operand_token!(l_paren_operator, "(", TT::LeftParen);
operand_token!(r_paren_operator, ")", TT::RightParen);
operand_token!(l_bracket_operator, "{", TT::LeftBracket);
operand_token!(r_bracket_operator, "}", TT::RightBracket);
operand_token!(star_operator, "*", TT::Star);
operand_token!(slash_operator, "/", TT::Slash);
operand_token!(semicolon_operator, ";", TT::Semicolon);
operand_token!(dot_operator, ".", TT::Dot);
operand_token!(comma_operator, ",", TT::Comma);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    EndOfFile,
    Plus,
    Minus,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Dot,
    Slash,
    Star,
}

impl Display for TT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TT::EndOfFile => write!(f, "EndOfFile"),
            TT::Plus => write!(f, "Plus"),
            TT::Minus => write!(f, "Minus"),
            TT::LeftParen => write!(f, "LeftParen"),
            TT::RightParen => write!(f, "RightParen"),
            TT::LeftBracket => write!(f, "LeftBracket"),
            TT::RightBracket => write!(f, "RightBracket"),
            TT::Comma => write!(f, ","),
            TT::Dot => write!(f, "."),
            TT::Semicolon => write!(f, ";"),
            TT::Slash => write!(f, "/"),
            TT::Star => write!(f, "*"),
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
            alt((
                plus_operator,
                minus_operator,
                l_paren_operator,
                r_paren_operator,
                l_bracket_operator,
                r_bracket_operator,
                comma_operator,
                star_operator,
                slash_operator,
                dot_operator,
                semicolon_operator,
            )),
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

    use super::*;
    use rstest::*;

    #[rstest]
    #[case::test_empty_line_eof_token("", TT::EndOfFile)]
    #[case::test_plus_token("+", TT::Plus)]
    #[case::test_minus_token("-", TT::Minus)]
    #[case::test_left_paren_token("(", TT::LeftParen)]
    #[case::test_right_paren_token(")", TT::RightParen)]
    #[case::test_left_bracket_token("{", TT::LeftBracket)]
    #[case::test_right_bracket_token("}", TT::RightBracket)]
    #[case::test_star_token("*", TT::Star)]
    #[case::test_dot_token(".", TT::Dot)]
    #[case::test_slash_token("/", TT::Slash)]
    #[case::test_comma_token(",", TT::Comma)]
    #[case::test_semicolon_token(";", TT::Semicolon)]

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
