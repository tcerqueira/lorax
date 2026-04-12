use std::{borrow::Cow, fmt::Display};

use report::{Span, Spanned};

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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ty.fmt(f)
    }
}

#[macro_export]
macro_rules! tok {
    (EOF, $line:expr) => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Eof,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['(', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LeftParen,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [')', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::RightParen,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['{', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LeftBrace,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    ['}', $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::RightBrace,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [,, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Comma,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [., $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Dot,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [-, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Minus,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [+, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Plus,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [;, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Semicolon,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [*, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Star,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Bang,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [!=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::BangEqual,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Equal,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [==, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::EqualEqual,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Less,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [<=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::LessEqual,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Greater,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [>=, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::GreaterEqual,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [/, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Slash,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [and, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::And,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [class, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Class,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [else, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Else,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [false, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::False,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [for, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::For,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [fun, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Fun,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [if, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::If,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [nil, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Nil,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [or, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Or,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [print, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Print,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [return, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Return,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [super, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Super,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [this, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::This,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [true, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::True,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [var, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Var,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };
    [while, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::While,
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()} ,
        }
    };

    [$any:tt] => {
        tok![$any, 1]
    };

    [s: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::String($lit.into()),
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [n: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Number($lit as f64),
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };
    [id: $lit:expr, $line:expr] => {
        $crate::tokens::Token {
            ty: $crate::tokens::TokenType::Identifier($lit.into()),
            span: report::Span { line_start: $line, line_end: $line, ..Default::default()},
        }
    };

    [$tag:tt: $lit:expr] => {
        tok![$tag: $lit, 1]
    };
}
