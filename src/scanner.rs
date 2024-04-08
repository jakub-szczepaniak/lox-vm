use crate::token::TT;
use crate::token::*;

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
        let result = tokenize(self.source);

        match result {
            Ok((_, token)) => {
                self.tokens = token;
            }
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
}
