pub enum TokenType {
    EOF,
}

pub struct Token {
    ttype: TokenType,
    line: usize,
    lexeme: String,
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        Vec::new()
    }
}
