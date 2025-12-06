use std::iter::FusedIterator;

use rlox_report::{error::LexingError, *};

use super::tokens::*;

pub struct Scanner<'s> {
    source: &'s str,
    curr: usize,
    global_curr: usize,
    line: u32,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            curr: 0,
            global_curr: 0,
            line: 1,
        }
    }

    // TODO: make it a lazy iterator use std::iter::from_fn
    pub fn scan_tokens(mut self) -> Result<Vec<Token>, Vec<LexingError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        loop {
            match self.scan_token() {
                Ok(Some(token)) => tokens.push(self.make_token(token)),
                Ok(None) => break,
                Err(err) => errors.push(err),
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        tokens.push(Token {
            ty: TokenType::Eof,
            span: Span {
                line_start: self.line,
                line_end: self.line,
                ..Default::default()
            },
        });
        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Option<TokenType>, LexingError> {
        let tok = loop {
            let Some(c) = self.advance_checked() else {
                return Ok(None);
            };
            let tok = match c {
                '(' => Some(TokenType::LeftParen),
                ')' => Some(TokenType::RightParen),
                '{' => Some(TokenType::LeftBrace),
                '}' => Some(TokenType::RightBrace),
                ',' => Some(TokenType::Comma),
                '.' => Some(TokenType::Dot),
                '-' => Some(TokenType::Minus),
                '+' => Some(TokenType::Plus),
                ';' => Some(TokenType::Semicolon),
                '*' => Some(TokenType::Star),
                '!' if self.matches('=') => Some(TokenType::BangEqual),
                '!' => Some(TokenType::Bang),
                '=' if self.matches('=') => Some(TokenType::EqualEqual),
                '=' => Some(TokenType::Equal),
                '<' if self.matches('=') => Some(TokenType::LessEqual),
                '<' => Some(TokenType::Less),
                '>' if self.matches('=') => Some(TokenType::GreaterEqual),
                '>' => Some(TokenType::Greater),
                '/' if self.matches('/') => {
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.advance(); // just consume the comment until end of the line
                    }
                    None
                }
                '/' => Some(TokenType::Slash),
                ' ' | '\r' | '\t' => None,
                '\n' => {
                    self.line += 1;
                    None
                }
                '"' => Some(self.string()?),
                c if c.is_ascii_digit() => Some(self.number()?),
                c if c.is_alphabetic() || c == '_' => Some(self.identifier()?),
                _ => {
                    return Err(LexingError::new(
                        self.make_span(),
                        "Unexpected character.".into(),
                    ));
                }
            };
            // update source to the start of the next token
            self.source = self.rest_span();
            self.curr = 0;

            break match tok {
                Some(tok) => tok,
                None => continue,
            };
        };
        Ok(Some(tok))
    }

    pub fn next_token(&mut self) -> Option<Result<Token, LexingError>> {
        self.scan_token()
            .transpose()
            .map(|t| t.map(|t| self.make_token(t)))
    }

    fn identifier(&mut self) -> Result<TokenType, LexingError> {
        for _ in self
            .rest_span()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
        {
            self.advance();
        }
        let ident = self.curr_span();
        Ok(keyword(ident).unwrap_or(TokenType::Identifier(ident.into())))
    }

    fn string(&mut self) -> Result<TokenType, LexingError> {
        // TODO: escape string
        for c in self.rest_span().chars().take_while(|c| *c != '"') {
            if c == '\n' {
                self.line += 1
            };
            self.advance();
        }

        // Consume closing quote
        self.advance_checked().ok_or(LexingError::new(
            self.make_span(),
            "Unterminated string.".into(),
        ))?;

        const QUOTE_WIDTH: usize = '"'.len_utf8();
        Ok(TokenType::String(
            self.curr_span()[QUOTE_WIDTH..self.curr_span().len() - QUOTE_WIDTH].into(),
        ))
    }

    fn number(&mut self) -> Result<TokenType, LexingError> {
        for _ in self.rest_span().chars().take_while(char::is_ascii_digit) {
            self.advance();
        }

        if self.peek().is_some_and(|c| c == '.')
            && self.peek_nth(1).is_some_and(|c| c.is_ascii_digit())
        {
            self.advance();
            for _ in self.rest_span().chars().take_while(char::is_ascii_digit) {
                self.advance();
            }
        }

        Ok(TokenType::Number(self.curr_span().parse().expect(
            "any number with digits from 0..9, optionally separated by one '.', should always parse",
        )))
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.increment_curr(c.len_utf8());
                true
            }
            _ => false,
        }
    }

    fn advance(&mut self) -> char {
        let c = self
            .rest_span()
            .chars()
            .next()
            .expect("advance assumes there's a character to consume");
        self.increment_curr(c.len_utf8());
        c
    }

    fn advance_checked(&mut self) -> Option<char> {
        let c = self.rest_span().chars().next()?;
        self.increment_curr(c.len_utf8());
        Some(c)
    }

    fn peek(&self) -> Option<char> {
        self.rest_span().chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.rest_span().chars().nth(n)
    }

    fn make_token(&self, token: TokenType) -> Token {
        Token {
            ty: token,
            span: self.make_span(),
        }
    }

    fn curr_span(&self) -> &'s str {
        &self.source[..self.curr]
    }

    fn rest_span(&self) -> &'s str {
        &self.source[self.curr..]
    }

    fn increment_curr(&mut self, inc: usize) {
        self.curr += inc;
        self.global_curr += inc;
    }

    fn make_span(&self) -> Span {
        Span {
            start: self.global_curr - self.curr_span().len(),
            end: self.global_curr,
            line_start: self.line,
            line_end: self.line,
        }
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

impl Iterator for Scanner<'_> {
    type Item = Result<Token, LexingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

impl FusedIterator for Scanner<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tok;

    fn tokens_eq(actual: &[Token], expected: &[Token]) -> bool {
        fn m(t: &Token) -> &TokenType {
            &t.ty
        }
        Iterator::eq(actual.iter().map(m), expected.iter().map(m))
    }

    #[test]
    fn test_single_char() {
        let source = "(){},-+.;*";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok!['('],
                tok![')'],
                tok!['{'],
                tok!['}'],
                tok![,],
                tok![-],
                tok![+],
                tok![.],
                tok![;],
                tok![*],
                tok![EOF],
            ]
        ));
    }

    #[test]
    fn test_one_or_two_char() {
        let source = "=!<>!=>=<===";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok![=],
                tok![!],
                tok![<],
                tok![>],
                tok![!=],
                tok![>=],
                tok![<=],
                tok![==],
                tok![EOF],
            ]
        ));
    }

    #[test]
    fn test_whitespaces_ignored() {
        let source = "! = >\r\n== <\t= \n";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok![!],
                tok![=],
                tok![>],
                tok!(==, 2),
                tok!(<, 2),
                tok!(=, 2),
                tok!(EOF, 3),
            ]
        ));
    }

    #[test]
    fn test_comments_ignored() {
        let source = "///()\n/=() // wtv";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok!(/, 2),
                tok!(=, 2),
                tok!('(', 2),
                tok!(')', 2),
                tok!(EOF, 2),
            ]
        ));
    }

    #[test]
    fn test_string_literals() {
        let source = "\"this string should ignore these // \n!= ()\"";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok![
                    s: "this string should ignore these // \n!= ()",
                    2
                ],
                tok!(EOF, 2)
            ]
        ));
    }

    #[test]
    fn test_numbers() {
        let source = "1234567890 0.123 123.0 0.3";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok![n: 1234567890],
                tok![n: 0.123],
                tok![n: 123.0],
                tok![n: 0.3],
                tok![EOF],
            ]
        ));
    }

    #[test]
    fn test_idents() {
        let source = "_hello123world _and2 or_ var return";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        assert!(tokens_eq(
            &tokens,
            &[
                tok![id: "_hello123world"],
                tok![id: "_and2"],
                tok![id: "or_"],
                tok![var],
                tok![return],
                tok![EOF],
            ]
        ));
    }
}
