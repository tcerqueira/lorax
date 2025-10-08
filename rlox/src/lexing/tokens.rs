use std::{borrow::Cow, fmt::Display};

use crate::report::{Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
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
    Identifier(Box<str>),
    String(Box<str>),
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

impl TokenType {
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{{",
            TokenType::RightBrace => "}}",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Minus => "-",
            TokenType::Plus => "+",
            TokenType::Semicolon => ";",
            TokenType::Slash => "/",
            TokenType::Star => "*",
            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::Identifier(ident) => ident.as_ref(),
            TokenType::And => "and",
            TokenType::Class => "class",
            TokenType::Else => "else",
            TokenType::False => "false",
            TokenType::Fun => "fun",
            TokenType::For => "for",
            TokenType::If => "if",
            TokenType::Nil => "nil",
            TokenType::Or => "or",
            TokenType::Print => "print",
            TokenType::Return => "return",
            TokenType::Super => "super",
            TokenType::This => "this",
            TokenType::True => "true",
            TokenType::Var => "var",
            TokenType::While => "while",
            TokenType::Eof => "end of file",
            non_static => {
                return match non_static {
                    TokenType::Number(n) => n.to_string(),
                    TokenType::String(s) => format!("\"{s}\""),
                    _ => panic!("token type not matched"),
                }
                .into();
            }
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ty: TokenType,
    pub span: Span,
}

impl Token {
    pub fn ty(&self) -> &TokenType {
        &self.ty
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        self.ty().as_str()
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[macro_export]
macro_rules! tok {
    (EOF, $line:expr) => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Eof,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['(', $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::LeftParen,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [')', $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::RightParen,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['{', $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::LeftBrace,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['}', $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::RightBrace,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [,, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Comma,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [., $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Dot,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [-, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Minus,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [+, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Plus,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [;, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Semicolon,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [*, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Star,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Bang,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!=, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::BangEqual,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [=, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Equal,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [==, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::EqualEqual,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Less,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<=, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::LessEqual,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Greater,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>=, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::GreaterEqual,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [/, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Slash,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [and, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::And,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [class, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Class,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [else, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Else,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [false, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::False,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [for, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::For,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [fun, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Fun,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [if, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::If,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [nil, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Nil,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [or, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Or,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [print, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Print,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [return, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Return,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [super, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Super,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [this, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::This,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [true, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::True,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [var, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Var,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [while, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::While,
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };

    [$any:tt] => {
        tok![$any, 1]
    };

    [s: $lit:expr, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::String($lit.into()),
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [n: $lit:expr, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Number($lit as f64),
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [id: $lit:expr, $line:expr] => {
        $crate::lexing::tokens::Token {
            ty: $crate::lexing::tokens::TokenType::Identifier($lit.into()),
            span: $crate::report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };

    [$tag:tt: $lit:expr] => {
        tok![$tag: $lit, 1]
    };
}
