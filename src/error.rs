use std::fmt;

use crate::lexer::LexerError;
use crate::parser::ParserError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParserError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexer(error) => write!(f, "lexer error: {error}"),
            Self::Parser(error) => write!(f, "parser error: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<LexerError> for Error {
    fn from(error: LexerError) -> Self {
        Self::Lexer(error)
    }
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}
