use crate::value::Value;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{alpha1, alphanumeric1, multispace0},
    combinator::{map, map_res, recognize},
    multi::many0,
    number::complete::double,
    sequence::{delimited, pair},
    IResult,
};
use std::fmt::Display;
use std::str::Utf8Error;

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

pub fn tokenize(input: &str) -> IResult<&str, Vec<Token>> {
    many0(delimited(
        multispace0,
        alt((
            operand_tokens,
            equal_tokens,
            parsed_string,
            parsed_digit,
            parsed_symbol,
        )),
        multispace0,
    ))(input)
}

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

fn parsed_symbol(input: &str) -> IResult<&str, Token> {
    let bytes = input.as_bytes();

    let result = map_res(
        recognize(pair(
            alt((alpha1::<_, nom::error::Error<_>>, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |keyword| {
            let c = std::str::from_utf8(keyword);
            c.map(|syntax| match syntax {
                "class" => Token::new(TokenType::Class, 1, syntax.to_string()),
                "else" => Token::new(TokenType::Else, 1, syntax.to_string()),
                "fun" => Token::new(TokenType::Fun, 1, syntax.to_string()),
                "for" => Token::new(TokenType::For, 1, syntax.to_string()),
                "or" => Token::new(TokenType::Or, 1, syntax.to_string()),
                "print" => Token::new(TokenType::Print, 1, syntax.to_string()),
                "return" => Token::new(TokenType::Return, 1, syntax.to_string()),
                "super" => Token::new(TokenType::Super, 1, syntax.to_string()),
                "this" => Token::new(TokenType::This, 1, syntax.to_string()),
                "var" => Token::new(TokenType::Var, 1, syntax.to_string()),
                "while" => Token::new(TokenType::While, 1, syntax.to_string()),
                "break" => Token::new(TokenType::Break, 1, syntax.to_string()),
                "if" => Token::new(TokenType::If, 1, syntax.to_string()),
                "true" => Token {
                    ttype: TT::Constant,
                    line: 1,
                    lexeme: syntax.to_string(),
                    literal: Some(Literal::Boolean(true)),
                },
                "false" => Token {
                    ttype: TT::Constant,
                    line: 1,
                    lexeme: syntax.to_string(),
                    literal: Some(Literal::Boolean(false)),
                },
                "nil" => Token {
                    ttype: TT::Constant,
                    line: 1,
                    lexeme: syntax.to_string(),
                    literal: Some(Literal::Nil),
                },
                "and" => Token::new(TokenType::And, 1, syntax.to_string()),
                _ => Token::new(TokenType::Identifier, 1, syntax.to_string()),
            })
        },
    )(bytes);

    match result {
        Ok((out, token)) => {
            // handle the Ok case
            Ok((std::str::from_utf8(out).unwrap(), token))
        }
        Err(e) => {
            // handle the Err case
            let str_error = e.map(|inner_error| {
                let str_slice = std::str::from_utf8(inner_error.input).unwrap();
                nom::error::Error::new(str_slice, inner_error.code)
            });
            Err(str_error)
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    String(String),
    Number(Value),
    Boolean(bool),
    Nil,
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
    //literals
    String,
    Number,
    Identifier,
    Constant,

    //keywords
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Break,
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
    pub ttype: TT,
    line: usize,
    lexeme: String,
    literal: Option<Literal>,
}

impl Token {
    pub fn new(ttype: TT, line: usize, lexeme: String) -> Self {
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

pub use TokenType as TT;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
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
