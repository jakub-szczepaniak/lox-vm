use crate::token::*;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::IResult;

macro_rules! operand {
    ($func_name: ident, $lexeme: literal, $output: expr) => {
        fn $func_name(&mut self, input: &'a str) -> IResult<&str, Token> {
            map(tag($lexeme), move |_| {
                Token::new($output, self.line, $lexeme.to_string())
            })(input)
        }
    };
}
pub struct Tokenizer<'a> {
    source: &'a str,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, line: 1 }
    }

    pub fn tokenize(&mut self) -> IResult<&str, Vec<Token>> {
        let mut tokens = Vec::new();
        let source = self.source;
        let res = self.plus_operator(source).unwrap();
        tokens.push(res.1);

        let line = 1;
        tokens.push(Token::new(TT::EndOfFile, line, "".to_string())); // Use the temporary variable

        Ok((res.0, tokens.clone()))
    }

    operand!(plus_operator, "+", TT::Plus);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_tokenize_result_check() {
        let mut sut: Tokenizer = Tokenizer::new("");
        let result = sut.tokenize();

        assert!(result.is_ok())
    }
    #[rstest]
    fn test_tokenize_result_empty_input() {
        let mut sut: Tokenizer = Tokenizer::new("");
        let result = sut.tokenize().unwrap();

        assert_eq!(
            result.1.last().unwrap(),
            &Token::new(TT::EndOfFile, 1, "".to_string())
        )
    }
    #[rstest]
    fn test_tokenize_result_single_token() {
        let mut sut: Tokenizer = Tokenizer::new("+");
        let result = sut.tokenize().unwrap();
        assert_eq!(
            result.1.first().unwrap(),
            &Token::new(TT::Plus, 1, "+".to_string())
        )
    }
}
