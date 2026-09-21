#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    HeadingMarker(u8),
    Text,
    Newline,
    BlankLine,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
}