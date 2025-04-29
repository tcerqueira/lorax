use std::fmt::Display;

use crate::span::Span;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ty: TokenType,
    pub span: Span,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Minus => write!(f, "-"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Identifier(ident) => write!(f, "{ident}"),
            TokenType::String(s) => write!(f, "\"{s}\""),
            TokenType::Number(num) => write!(f, "{num}"),
            TokenType::And => write!(f, "and"),
            TokenType::Class => write!(f, "class"),
            TokenType::Else => write!(f, "else"),
            TokenType::False => write!(f, "false"),
            TokenType::Fun => write!(f, "fun"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Nil => write!(f, "nil"),
            TokenType::Or => write!(f, "or"),
            TokenType::Print => write!(f, "print"),
            TokenType::Return => write!(f, "return"),
            TokenType::Super => write!(f, "super"),
            TokenType::This => write!(f, "this"),
            TokenType::True => write!(f, "true"),
            TokenType::Var => write!(f, "var"),
            TokenType::While => write!(f, "while"),
            TokenType::Eof => write!(f, "end of file"),
        }
    }
}

#[macro_export]
macro_rules! tok {
    (EOF, $line:expr) => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Eof,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['(', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LeftParen,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [')', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::RightParen,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['{', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LeftBrace,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['}', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::RightBrace,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [,, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Comma,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [., $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Dot,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [-, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Minus,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [+, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Plus,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [;, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Semicolon,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [*, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Star,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Bang,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::BangEqual,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Equal,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [==, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::EqualEqual,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Less,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LessEqual,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Greater,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::GreaterEqual,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [/, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Slash,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [and, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::And,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [class, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Class,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [else, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Else,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [false, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::False,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [for, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::For,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [fun, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Fun,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [if, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::If,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [nil, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Nil,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [or, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Or,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [print, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Print,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [return, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Return,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [super, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Super,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [this, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::This,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [true, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::True,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [var, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Var,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [while, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::While,
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };

    [$any:tt] => {
        tok![$any, 1]
    };

    [s: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::String($lit.into()),
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [n: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Number($lit as f64),
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [id: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Identifier($lit.into()),
            span: $crate::span::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };

    [$tag:tt: $lit:expr] => {
        tok![$tag: $lit, 1]
    };
}
