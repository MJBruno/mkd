#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Hash,

    Star,
    DoubleStar,

    Tilde,
    DoubleTilde,

    Underscore,
    DoubleUnderscore,

    Backtick,

    Backslash,

    LeftBracket,
    RightBracket,

    LeftParen,
    RightParen,

    LeftBrace,
    RightBrace,

    Pipe,
    Colon,

    Exclamation,

    Minus,
    Plus,
    Dot,

    GreaterThan,
    LessThan,

    Indent(usize),

    Text,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, position: Position) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            position,
        }
    }
}
