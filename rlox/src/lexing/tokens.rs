use std::fmt::Display;

use crate::report::Span;

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
    pub fn ident(&self) -> &str {
        let TokenType::Identifier(ident) = self else {
            panic!("token is not an Identifier")
        };
        ident
    }
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
