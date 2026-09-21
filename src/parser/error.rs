use crate::lexer::{LexerError, Position};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    Lexer(LexerError),

    UnexpectedToken {
        expected: String,
        found: String,
        position: Position,
    },

    UnclosedDelimiter {
        delimiter: String,
        position: Position,
    },

    UnclosedBlock {
        kind: String,
        position: Position,
    },

    InvalidAnchor {
        value: String,
        position: Position,
    },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexer(error) => write!(f, "lexer error: {error}"),
            Self::UnexpectedToken {
                expected,
                found,
                position,
            } => write!(
                f,
                "unexpected token {:?} at {}:{}; expected {}",
                found, position.line, position.column, expected
            ),
            Self::UnclosedDelimiter {
                delimiter,
                position,
            } => write!(
                f,
                "unclosed delimiter {:?} at {}:{}",
                delimiter, position.line, position.column
            ),
            Self::UnclosedBlock { kind, position } => write!(
                f,
                "unclosed block {:?} at {}:{}",
                kind, position.line, position.column
            ),
            Self::InvalidAnchor { value, position } => write!(
                f,
                "invalid anchor {:?} at {}:{}",
                value, position.line, position.column
            ),
        }
    }
}

impl std::error::Error for ParserError {}
