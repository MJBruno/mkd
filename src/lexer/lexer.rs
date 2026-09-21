use super::error::LexerError;
use super::token::{Position, Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokenize(&self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        for (line_index, line) in self.source.lines().enumerate() {
            let line_number = line_index + 1;

            if line.trim().is_empty() {
                tokens.push(Token {
                    kind: TokenKind::BlankLine,
                    lexeme: String::new(),
                    position: Position::new(line_number, 1),
                });

                continue;
            }

            if let Some(level) = heading_level(line) {
                let marker = "#".repeat(level as usize);

                tokens.push(Token {
                    kind: TokenKind::HeadingMarker(level),
                    lexeme: marker,
                    position: Position::new(line_number, 1),
                });

                let text_start = level as usize + 1;
                let text = &line[text_start..];

                tokens.push(Token {
                    kind: TokenKind::Text,
                    lexeme: text.to_string(),
                    position: Position::new(
                        line_number,
                        text_start + 1,
                    ),
                });

                continue;
            }

            tokens.push(Token {
                kind: TokenKind::Text,
                lexeme: line.to_string(),
                position: Position::new(line_number, 1),
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            position: Position::new(
                self.source.lines().count() + 1,
                1,
            ),
        });

        Ok(tokens)
    }
}

fn heading_level(line: &str) -> Option<u8> {
    let bytes = line.as_bytes();

    let mut level = 0;

    while level < 6 && bytes.get(level) == Some(&b'#') {
        level += 1;
    }

    if level == 0 {
        return None;
    }

    if bytes.get(level) != Some(&b' ') {
        return None;
    }

    Some(level as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind;

    #[test]
    fn heading_has_position() {
        let lexer = Lexer::new("# Hello");

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[0].position,
            Position::new(1, 1)
        );

        assert_eq!(
            tokens[1].position,
            Position::new(1, 3)
        );
    }

    #[test]
    fn second_line_has_correct_position() {
        let lexer = Lexer::new(
            "# Title\n\
             Hello"
        );

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[2].position.line,
            2
        );

        assert_eq!(
            tokens[2].position.column,
            1
        );
    }

    #[test]
    fn blank_line_has_position() {
        let lexer = Lexer::new(
            "Hello\n\nWorld"
        );

        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[1].kind,
            TokenKind::BlankLine
        );

        assert_eq!(
            tokens[1].position,
            Position::new(2, 1)
        );
    }
}