use super::token::Position;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    InvalidCharacter {
        character: char,
        position: Position,
    },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter { character, position } => write!(
                f,
                "invalid character {:?} at {}:{}",
                character, position.line, position.column
            ),
        }
    }
}

impl std::error::Error for LexerError {}
