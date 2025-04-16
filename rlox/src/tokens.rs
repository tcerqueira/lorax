use std::fmt::Display;

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
    pub span: Box<str>,
    pub line: u32,
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
        Token {
            ty: TokenType::Eof,
            span: "".into(),
            line: $line,
        }
    };
    ['(', $line:expr] => {
        Token {
            ty: TokenType::LeftParen,
            span: "(".into(),
            line: $line,
        }
    };
    [')', $line:expr] => {
        Token {
            ty: TokenType::RightParen,
            span: ")".into(),
            line: $line,
        }
    };
    ['{', $line:expr] => {
        Token {
            ty: TokenType::LeftBrace,
            span: "{".into(),
            line: $line,
        }
    };
    ['}', $line:expr] => {
        Token {
            ty: TokenType::RightBrace,
            span: "}".into(),
            line: $line,
        }
    };
    [,, $line:expr] => {
        Token {
            ty: TokenType::Comma,
            span: ",".into(),
            line: $line,
        }
    };
    [., $line:expr] => {
        Token {
            ty: TokenType::Dot,
            span: ".".into(),
            line: $line,
        }
    };
    [-, $line:expr] => {
        Token {
            ty: TokenType::Minus,
            span: "-".into(),
            line: $line,
        }
    };
    [+, $line:expr] => {
        Token {
            ty: TokenType::Plus,
            span: "+".into(),
            line: $line,
        }
    };
    [;, $line:expr] => {
        Token {
            ty: TokenType::Semicolon,
            span: ";".into(),
            line: $line,
        }
    };
    [*, $line:expr] => {
        Token {
            ty: TokenType::Star,
            span: "*".into(),
            line: $line,
        }
    };
    [!, $line:expr] => {
        Token {
            ty: TokenType::Bang,
            span: "!".into(),
            line: $line,
        }
    };
    [!=, $line:expr] => {
        Token {
            ty: TokenType::BangEqual,
            span: "!=".into(),
            line: $line,
        }
    };
    [=, $line:expr] => {
        Token {
            ty: TokenType::Equal,
            span: "=".into(),
            line: $line,
        }
    };
    [==, $line:expr] => {
        Token {
            ty: TokenType::EqualEqual,
            span: "==".into(),
            line: $line,
        }
    };
    [<, $line:expr] => {
        Token {
            ty: TokenType::Less,
            span: "<".into(),
            line: $line,
        }
    };
    [<=, $line:expr] => {
        Token {
            ty: TokenType::LessEqual,
            span: "<=".into(),
            line: $line,
        }
    };
    [>, $line:expr] => {
        Token {
            ty: TokenType::Greater,
            span: ">".into(),
            line: $line,
        }
    };
    [>=, $line:expr] => {
        Token {
            ty: TokenType::GreaterEqual,
            span: ">=".into(),
            line: $line,
        }
    };
    [/, $line:expr] => {
        Token {
            ty: TokenType::Slash,
            span: "/".into(),
            line: $line,
        }
    };
    [and, $line:expr] => {
        Token {
            ty: TokenType::And,
            span: "and".into(),
            line: $line,
        }
    };
    [class, $line:expr] => {
        Token {
            ty: TokenType::Class,
            span: "class".into(),
            line: $line,
        }
    };
    [else, $line:expr] => {
        Token {
            ty: TokenType::Else,
            span: "else".into(),
            line: $line,
        }
    };
    [false, $line:expr] => {
        Token {
            ty: TokenType::False,
            span: "false".into(),
            line: $line,
        }
    };
    [for, $line:expr] => {
        Token {
            ty: TokenType::For,
            span: "for".into(),
            line: $line,
        }
    };
    [fun, $line:expr] => {
        Token {
            ty: TokenType::Fun,
            span: "fun".into(),
            line: $line,
        }
    };
    [if, $line:expr] => {
        Token {
            ty: TokenType::If,
            span: "if".into(),
            line: $line,
        }
    };
    [nil, $line:expr] => {
        Token {
            ty: TokenType::Nil,
            span: "nil".into(),
            line: $line,
        }
    };
    [or, $line:expr] => {
        Token {
            ty: TokenType::Or,
            span: "or".into(),
            line: $line,
        }
    };
    [print, $line:expr] => {
        Token {
            ty: TokenType::Print,
            span: "print".into(),
            line: $line,
        }
    };
    [return, $line:expr] => {
        Token {
            ty: TokenType::Return,
            span: "return".into(),
            line: $line,
        }
    };
    [super, $line:expr] => {
        Token {
            ty: TokenType::Super,
            span: "super".into(),
            line: $line,
        }
    };
    [this, $line:expr] => {
        Token {
            ty: TokenType::This,
            span: "this".into(),
            line: $line,
        }
    };
    [true, $line:expr] => {
        Token {
            ty: TokenType::True,
            span: "true".into(),
            line: $line,
        }
    };
    [var, $line:expr] => {
        Token {
            ty: TokenType::Var,
            span: "var".into(),
            line: $line,
        }
    };
    [while, $line:expr] => {
        Token {
            ty: TokenType::While,
            span: "while".into(),
            line: $line,
        }
    };

    [$any:tt] => {
        tok![$any, 1]
    };

    [s: $lit:expr, $line:expr] => {
        Token {
            ty: TokenType::String($lit.into()),
            span: format!("\"{}\"", $lit).into(),
            line: $line,
        }
    };
    [n: $lit:expr, $line:expr] => {
        Token {
            ty: TokenType::Number($lit as f64),
            span: stringify!($lit).into(),
            line: $line,
        }
    };
    [id: $lit:expr, $line:expr] => {
        Token {
            ty: TokenType::Identifier($lit.into()),
            span: $lit.into(),
            line: $line,
        }
    };

    [$tag:tt: $lit:expr] => {
        tok![$tag: $lit, 1]
    };
}
