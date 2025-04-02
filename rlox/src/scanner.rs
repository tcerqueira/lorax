use crate::error::CompileError;
use crate::tokens::*;

pub struct Scanner<'s> {
    tokens: Vec<Token>,
    source: &'s str,
    curr: usize,
    line: u32,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            tokens: vec![],
            source,
            curr: 0,
            line: 1,
        }
    }

    // TODO: make it a lazy iterator use std::iter::from_fn
    pub fn scan_tokens(mut self) -> Result<Vec<Token>, Vec<CompileError>> {
        let mut errors = vec![];
        while !self.source.is_empty() {
            if let Err(err) = self.scan_token() {
                errors.push(err);
            }
            // update source to the start of the next token
            self.source = self.rest_span();
            self.curr = 0;
        }
        self.tokens.push(Token {
            ty: TokenType::Eof,
            span: "".into(),
            line: self.line,
        });
        Ok(self.tokens)
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
                    while self.peek().is_some_and(|c| c != '\n') {
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
            'a'..='z' | 'A'..='Z' | '_' => self.identifier()?,
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

    fn identifier(&mut self) -> Result<(), CompileError> {
        for _ in self
            .rest_span()
            .chars()
            .take_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
        {
            self.advance();
        }
        let ident = self.curr_span();
        self.add_token(keyword(ident).unwrap_or(TokenType::Identifier(ident.into())));
        Ok(())
    }

    fn string(&mut self) -> Result<(), CompileError> {
        for c in self.rest_span().chars().take_while(|c| *c != '"') {
            if c == '\n' {
                self.line += 1
            };
            self.advance();
        }

        // Consume closing quote
        self.advance_checked().ok_or(CompileError {
            line: self.line,
            span: "".into(),
            message: "Unterminated string.".into(),
        })?;

        self.add_token(TokenType::String(
            self.curr_span()[1..self.curr_span().len() - 1].into(),
        ));
        Ok(())
    }

    fn number(&mut self) -> Result<(), CompileError> {
        for _ in self.rest_span().chars().take_while(char::is_ascii_digit) {
            self.advance();
        }

        if self.peek() == Some('.') && matches!(self.peek_nth(1), Some('0'..='9')) {
            self.advance();
            for _ in self.rest_span().chars().take_while(char::is_ascii_digit) {
                self.advance();
            }
        }

        self.add_token(TokenType::Number(self.curr_span().parse().expect(
            "any number with digits from 0..9, optionally separated by one '.', should always parse",
        )));
        Ok(())
    }

    fn matches(&mut self, expected: char) -> bool {
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
            .expect("advance assumes there's a character to consume");
        self.curr += 1;
        c
    }

    fn advance_checked(&mut self) -> Option<char> {
        let c = self.rest_span().chars().next()?;
        self.curr += 1;
        Some(c)
    }

    fn peek(&self) -> Option<char> {
        self.rest_span().chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.rest_span().chars().nth(n)
    }

    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token {
            ty: token,
            span: self.curr_span().into(),
            line: self.line,
        });
    }

    fn curr_span(&self) -> &'s str {
        &self.source[..self.curr]
    }

    fn rest_span(&self) -> &'s str {
        &self.source[self.curr..]
    }
}

fn keyword(s: &str) -> Option<TokenType> {
    Some(match s {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tt {
        ("eof") => {
            tt!("eof", 1)
        };

        ("eof", $line:expr) => {
            Token {
                ty: TokenType::Eof,
                span: "".into(),
                line: $line,
            }
        };

        ($str:literal) => {
            tt!($str, 1)
        };

        ($str:literal, $line:expr) => {
            Token {
                ty: match $str {
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
                    "and" => TokenType::And,
                    "class" => TokenType::Class,
                    "else" => TokenType::Else,
                    "false" => TokenType::False,
                    "for" => TokenType::For,
                    "fun" => TokenType::Fun,
                    "if" => TokenType::If,
                    "nil" => TokenType::Nil,
                    "or" => TokenType::Or,
                    "print" => TokenType::Print,
                    "return" => TokenType::Return,
                    "super" => TokenType::Super,
                    "this" => TokenType::This,
                    "true" => TokenType::True,
                    "var" => TokenType::Var,
                    "while" => TokenType::While,
                    _ => TokenType::Identifier($str.into()),
                },
                span: $str.into(),
                line: $line,
            }
        };

        (string, $value:expr, $span:expr) => {
            tt!(string, $value, $span, 1)
        };

        (string, $value:expr, $span:expr, $line:expr) => {
            Token {
                ty: TokenType::String($value.into()),
                span: $span.into(),
                line: $line,
            }
        };

        (number, $value:expr) => {
            tt!(number, $value, 1)
        };

        (number, $value:expr, $line:expr) => {
            Token {
                ty: TokenType::Number($value as f64),
                span: stringify!($value).into(),
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
                tt!("eof"),
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
                tt!("eof"),
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
                tt!(
                    string,
                    "this string should ignore these // \n!= ()",
                    "\"this string should ignore these // \n!= ()\"",
                    2
                ),
                tt!("eof", 2),
            ]
        )
    }

    #[test]
    fn test_numbers() {
        let source = "1234567890 0.123 123.0 0.3";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!(number, 1234567890),
                tt!(number, 0.123),
                tt!(number, 123.0),
                tt!(number, 0.3),
                tt!("eof"),
            ]
        )
    }

    #[test]
    fn test_idents() {
        let source = "_hello123world _and2 or_ var return";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert_eq!(
            tokens,
            vec![
                tt!("_hello123world"),
                tt!("_and2"),
                tt!("or_"),
                tt!("var"),
                tt!("return"),
                tt!("eof"),
            ]
        )
    }

    #[test]
    fn test_wtv() {
        let s = "==";
        assert_eq!(&s[2..], "");
    }
}
