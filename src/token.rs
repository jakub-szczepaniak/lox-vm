use std::fmt::Display;

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(a) => write!(f, "{}", a),
            Self::Boolean(a) => write!(f, "{}", a),
            Self::Nil => write!(f, "None"),
            Self::Number(x) => write!(f, "{}", x),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    //special
    EndOfFile,
    Error,
    Undefined,
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
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Break,
    If,
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
            TT::Constant => write!(f, "Constant"),
            TT::Identifier => write!(f, "Identifier"),
            TT::And => write!(f, "And"),
            TT::If => write!(f, "If"),
            TT::Break => write!(f, "Break"),
            _ => write!(f, ""),
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub ttype: TT,
    pub line: usize,
    pub lexeme: String,
    pub literal: Option<Literal>,
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
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ttype {
            TT::String | TT::Number | TT::Constant => {
                write!(f, "{}::{}:{:?}", self.line, self.ttype, self.literal)
            }

            _ => write!(f, "{}::{}:{}", self.line, self.ttype, self.lexeme),
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TT::Undefined,
            line: 0,
            lexeme: "".to_string(),
            literal: None,
        }
    }
}

pub use TokenType as TT;
