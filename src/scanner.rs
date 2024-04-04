use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take, take_while},
    character::complete::{digit1, line_ending, space0},
    character::is_digit,
    combinator::{map, map_res},
    multi::many0,
    number::complete::double,
    sequence::delimited,
    IResult,
};
use std::{fmt::Display, str::Utf8Error};

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

fn operand_tokens(input: &str) -> IResult<&str, Token> {
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
    ))(input)
}

fn equal_tokens(input: &str) -> IResult<&str, Token> {
    use nom::character::complete::char;
    alt((
        map(tag("!="), |_| {
            Token::new(TT::BangEquals, 1, "!=".to_string())
        }),
        map(char('!'), |_| Token::new(TT::Bang, 1, "!".to_string())),
        map(tag("=="), |_| Token::new(TT::Equals, 1, "==".to_string())),
        map(char('='), |_| Token::new(TT::Assign, 1, "=".to_string())),
        map(tag(">="), |_| {
            Token::new(TT::GreaterEquals, 1, ">=".to_string())
        }),
        map(char('>'), |_| Token::new(TT::Greater, 1, ">".to_string())),
        map(tag("<="), |_| {
            Token::new(TT::LessEquals, 1, "<=".to_string())
        }),
        map(char('<'), |_| Token::new(TT::Less, 1, "<".to_string())),
    ))(input)
}

fn parsed_string(input: &str) -> IResult<&str, Token> {
    map(string, Token::new_string)(input)
}

fn string(input: &str) -> IResult<&str, String> {
    delimited(tag("\""), map_res(utf_points, convert_vec_utf8), tag("\""))(input)
}

fn utf_points(input: &str) -> IResult<&str, Vec<u8>> {
    let (input1, char1) = take(1usize)(input)?;
    match char1.as_bytes() {
        b"\"" => Ok((input, vec![])),
        b"\\" => {
            let (input2, char2) = take(1usize)(input1)?;
            utf_points(input2)
                .map(|(slice, done)| (slice, concat_slice_with_vec(char2.as_bytes(), done)))
        }
        c => utf_points(input1).map(|(slice, done)| (slice, concat_slice_with_vec(c, done))),
    }
}

fn concat_slice_with_vec(c: &[u8], done: Vec<u8>) -> Vec<u8> {
    let mut new_vec = c.to_vec();
    new_vec.extend(done);
    new_vec
}

fn convert_vec_utf8(v: Vec<u8>) -> Result<String, Utf8Error> {
    let slice = v.as_slice();
    std::str::from_utf8(slice).map(|s| s.to_owned())
}

fn parsed_digit(input: &str) -> IResult<&str, Token> {
    let (input, lexeme) = double(input)?;

    Ok((input, Token::new_digit(format!("{}", lexeme), lexeme)))
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    String(String),
    Number(Value),
}

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
    //double letter tokens
    Bang,
    BangEquals,
    Equals,
    Assign,
    Greater,
    GreaterEquals,
    Less,
    LessEquals,
    //
    String,
    Number,
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
            TT::Bang => write!(f, "Bang"),
            TT::BangEquals => write!(f, "BangEquals"),
            TT::Equals => write!(f, "Equals"),
            TT::Assign => write!(f, "Assign"),
            TT::Greater => write!(f, "Greater"),
            TT::String => write!(f, "String"),
            TT::Number => write!(f, "Number"),
            _ => write!(f, ""),
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    ttype: TT,
    line: usize,
    lexeme: String,
    literal: Option<Literal>,
}

impl Token {
    fn new(ttype: TT, line: usize, lexeme: String) -> Self {
        Self {
            ttype,
            line,
            lexeme,
            literal: None,
        }
    }
    fn new_string(lexeme: String) -> Self {
        Self {
            ttype: TT::String,
            line: 1,
            lexeme: lexeme.clone(),
            literal: Some(Literal::String(lexeme)),
        }
    }

    fn new_digit(lexeme: String, value: f64) -> Self {
        Self {
            ttype: TT::Number,
            line: 1,
            lexeme: lexeme.clone(),
            literal: Some(Literal::Number(value)),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}:{}", self.line, self.ttype, self.lexeme)
    }
}

use TokenType as TT;

use crate::value::Value;

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
            alt((operand_tokens, equal_tokens, parsed_string, parsed_digit)),
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
    #[rstest]
    #[case::bang_equals("!=", TT::BangEquals)]
    #[case::bang("!", TT::Bang)]
    #[case::equal_equals("==", TT::Equals)]
    #[case::assign("=", TT::Assign)]
    #[case::greater(">", TT::Greater)]
    #[case::greater_equals(">=", TT::GreaterEquals)]
    #[case::less("<", TT::Less)]
    #[case::less_equals("<=", TT::LessEquals)]

    fn test_double_char_tokens(#[case] input: &str, #[case] expected: TT) {
        let mut scanner = Scanner::new(input);
        scanner.tokenize();

        assert_eq!(scanner.tokens[0].ttype, expected)
    }

    #[rstest]
    #[case::simple_string("\"ab\"", Token::new_string("ab".to_string()))]

    fn test_string_parser(#[case] input: &str, #[case] output: Token) {
        assert_eq!(parsed_string(input), Ok(("", output)))
    }

    #[rstest]
    #[case::single_digit("1", Token::new_digit("1".to_string(), 1.0))]
    #[case::digit_with_fraction("1.2", Token::new_digit("1.2".to_string(), 1.2))]
    fn test_digit_scanner(#[case] input: &str, #[case] output: Token) {
        assert_eq!(parsed_digit(input), Ok(("", output)))
    }
}
