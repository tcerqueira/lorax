use crate::error::CompileError;

#[derive(Debug, PartialEq)]
pub enum TokenType<'s> {
    // Single charecter tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier(&'s str),
    String(&'s str),
    Number(f64),
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Other
    Eof,
}

impl Eq for TokenType<'_> {}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'s> {
    pub token: TokenType<'s>,
    pub span: String,
    pub line: u32,
}

pub struct Scanner<'s> {
    source: &'s str,
    tokens: Vec<Token<'s>>,
    start: usize,
    curr: usize,
    line: u32,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            curr: 0,
            line: 1,
        }
    }

    // TODO: make it a lazy iterator use std::iter::from_fn
    pub fn scan_tokens(mut self) -> Result<Vec<Token<'s>>, Vec<CompileError>> {
        let mut errors = vec![];
        while !self.is_at_end() {
            self.start = self.curr;
            if let Err(err) = self.scan_token() {
                errors.push(err);
            }
        }
        self.tokens.push(Token {
            token: TokenType::Eof,
            span: "".into(),
            line: self.line,
        });
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), CompileError> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => match self.matches('=') {
                true => self.add_token(TokenType::BangEqual),
                false => self.add_token(TokenType::Bang),
            },
            '=' => match self.matches('=') {
                true => self.add_token(TokenType::EqualEqual),
                false => self.add_token(TokenType::Equal),
            },
            '<' => match self.matches('=') {
                true => self.add_token(TokenType::LessEqual),
                false => self.add_token(TokenType::Less),
            },
            '>' => match self.matches('=') {
                true => self.add_token(TokenType::GreaterEqual),
                false => self.add_token(TokenType::Greater),
            },
            '/' => match self.matches('/') {
                true => {
                    while !self.is_at_end() && self.peek().is_some_and(|c| c != '\n') {
                        // just consume the comment until end of the line
                        self.advance();
                    }
                }
                false => self.add_token(TokenType::Slash),
            },
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            _ => {
                return Err(CompileError {
                    line: self.line,
                    span: self.curr_span().into(),
                    message: "Unexpected character.".into(),
                });
            }
        };
        Ok(())
    }

    fn string(&mut self) -> Result<(), CompileError> {
        while let Some(c) = self.peek() {
            match c {
                '"' => break,
                '\n' => self.line += 1,
                _ => {}
            };
            self.advance();
        }

        if self.is_at_end() {
            return Err(CompileError {
                line: self.line,
                span: "".into(),
                message: "Unterminated string.".into(),
            });
        }

        self.advance();
        self.add_token(TokenType::String(
            &self.source[self.start + 1..self.curr - 1],
        ));
        Ok(())
    }

    fn number(&mut self) -> Result<(), CompileError> {
        while let Some('0'..='9') = self.peek() {
            self.advance();
        }

        if self.peek() == Some('.') && matches!(self.peek_nth(1), Some('0'..='9')) {
            self.advance();
            while let Some('0'..='9') = self.peek() {
                self.advance();
            }
        }

        self.add_token(TokenType::Number(self.curr_span().parse().expect(
            "any number with digits from 0..9, optionally separated by one '.', should always parse",
        )));
        Ok(())
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.peek().is_some_and(|c| c == expected) {
            self.curr += 1;
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> char {
        let c = self
            .rest_span()
            .chars()
            .next()
            .expect("should always be called after a 'is_at_end' call");
        self.curr += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        self.rest_span().chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.rest_span().chars().nth(n)
    }

    fn add_token(&mut self, token: TokenType<'s>) {
        self.tokens.push(Token {
            token,
            span: self.curr_span().into(),
            line: self.line,
        });
    }

    fn curr_span(&self) -> &str {
        &self.source[self.start..self.curr]
    }

    fn rest_span(&self) -> &str {
        &self.source[self.curr..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tt {
        ("eof", $line:expr) => {
            Token {
                token: TokenType::Eof,
                span: "".into(),
                line: $line,
            }
        };

        ($str:literal) => {
            Token {
                token: match $str {
                    "(" => TokenType::LeftParen,
                    ")" => TokenType::RightParen,
                    "{" => TokenType::LeftBrace,
                    "}" => TokenType::RightBrace,
                    "," => TokenType::Comma,
                    "." => TokenType::Dot,
                    "-" => TokenType::Minus,
                    "+" => TokenType::Plus,
                    ";" => TokenType::Semicolon,
                    "*" => TokenType::Star,
                    "!" => TokenType::Bang,
                    "!=" => TokenType::BangEqual,
                    "=" => TokenType::Equal,
                    "==" => TokenType::EqualEqual,
                    "<" => TokenType::Less,
                    "<=" => TokenType::LessEqual,
                    ">" => TokenType::Greater,
                    ">=" => TokenType::GreaterEqual,
                    "/" => TokenType::Slash,
                    _ => panic!("Unsupported token: {}", $str),
                },
                span: $str.into(),
                line: 1,
            }
        };

        ($str:literal, $line:expr) => {
            Token {
                token: match $str {
                    "(" => TokenType::LeftParen,
                    ")" => TokenType::RightParen,
                    "{" => TokenType::LeftBrace,
                    "}" => TokenType::RightBrace,
                    "," => TokenType::Comma,
                    "." => TokenType::Dot,
                    "-" => TokenType::Minus,
                    "+" => TokenType::Plus,
                    ";" => TokenType::Semicolon,
                    "*" => TokenType::Star,
                    "!" => TokenType::Bang,
                    "!=" => TokenType::BangEqual,
                    "=" => TokenType::Equal,
                    "==" => TokenType::EqualEqual,
                    "<" => TokenType::Less,
                    "<=" => TokenType::LessEqual,
                    ">" => TokenType::Greater,
                    ">=" => TokenType::GreaterEqual,
                    "/" => TokenType::Slash,
                    _ => panic!("Unsupported token: {}", $str),
                },
                span: $str.into(),
                line: $line,
            }
        };

        (string, $value:expr, $line:expr) => {
            Token {
                token: TokenType::String($value.into()),
                span: format!("\"{}\"", $value),
                line: $line,
            }
        };

        (number, $value:expr, $line:expr) => {
            Token {
                token: TokenType::Number($value as f64),
                span: stringify!($value).into(),
                line: $line,
            }
        };

        (ident, $value:expr, $span:expr, $line:expr) => {
            Token {
                token: TokenType::Identifier($value.to_string()),
                span: $span.into(),
                line: $line,
            }
        };
    }

    #[test]
    fn test_single_char() {
        let source = "(){},-+.;*";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!("("),
                tt!(")"),
                tt!("{"),
                tt!("}"),
                tt!(","),
                tt!("-"),
                tt!("+"),
                tt!("."),
                tt!(";"),
                tt!("*"),
                tt!("eof", 1),
            ]
        )
    }

    #[test]
    fn test_one_or_two_char() {
        let source = "=!<>!=>=<===";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!("="),
                tt!("!"),
                tt!("<"),
                tt!(">"),
                tt!("!="),
                tt!(">="),
                tt!("<="),
                tt!("=="),
                tt!("eof", 1),
            ]
        )
    }

    #[test]
    fn test_whitespaces_ignored() {
        let source = "! = >\r\n== <\t= \n";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!("!"),
                tt!("="),
                tt!(">"),
                tt!("==", 2),
                tt!("<", 2),
                tt!("=", 2),
                tt!("eof", 3),
            ]
        )
    }

    #[test]
    fn test_comments_ignored() {
        let source = "///()\n/=() // wtv";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!("/", 2),
                tt!("=", 2),
                tt!("(", 2),
                tt!(")", 2),
                tt!("eof", 2),
            ]
        )
    }

    #[test]
    fn test_string_literals() {
        let source = "\"this string should ignore these // \n!= ()\"";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!(string, "this string should ignore these // \n!= ()", 2),
                tt!("eof", 2),
            ]
        )
    }

    #[test]
    fn test_numbers() {
        let source = "1234567890 0.123 123.0";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!(number, 1234567890, 1),
                tt!(number, 0.123, 1),
                tt!(number, 123.0, 1),
                tt!("eof", 1),
            ]
        )
    }
}
