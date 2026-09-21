use super::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();

        for line in self.source.lines() {
            if line.trim().is_empty() {
                tokens.push(Token {
                    kind: TokenKind::BlankLine,
                    lexeme: String::new(),
                });

                continue;
            }

            if let Some(level) = heading_level(line) {
                let marker = "#".repeat(level as usize);

                tokens.push(Token {
                    kind: TokenKind::HeadingMarker(level),
                    lexeme: marker,
                });

                let text = &line[level as usize + 1..];

                tokens.push(Token {
                    kind: TokenKind::Text,
                    lexeme: text.to_string(),
                });

                continue;
            }

            tokens.push(Token {
                kind: TokenKind::Text,
                lexeme: line.to_string(),
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
        });

        tokens
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
    fn lexes_heading() {
        let lexer = Lexer::new("# Hello");

        let tokens = lexer.tokenize();

        assert_eq!(
            tokens[0].kind,
            TokenKind::HeadingMarker(1)
        );

        assert_eq!(
            tokens[1].kind,
            TokenKind::Text
        );

        assert_eq!(
            tokens[1].lexeme,
            "Hello"
        );
    }

    #[test]
    fn lexes_blank_line() {
        let lexer = Lexer::new("Hello\n\nWorld");

        let tokens = lexer.tokenize();

        assert_eq!(tokens[1].kind, TokenKind::BlankLine);
    }

    #[test]
    fn hash_without_space_is_text() {
        let lexer = Lexer::new("#Hello");

        let tokens = lexer.tokenize();

        assert_eq!(tokens[0].kind, TokenKind::Text);
        assert_eq!(tokens[0].lexeme, "#Hello");
    }
}