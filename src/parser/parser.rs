use crate::ast::{Block, Document};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Document {
        let mut blocks = Vec::new();

        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::HeadingMarker(level) => {
                    blocks.push(self.parse_heading(level));
                }

                TokenKind::Text => {
                    blocks.push(self.parse_paragraph());
                }

                TokenKind::BlankLine => {
                    self.advance();
                }

                TokenKind::Eof => break,
            }
        }

        Document { blocks }
    }

    fn parse_heading(&mut self, level: u8) -> Block {
        self.advance();

        let text = self.advance().lexeme;

        Block::Heading { level, text }
    }

    fn parse_paragraph(&mut self) -> Block {
        let mut lines = Vec::new();

        while !self.is_at_end() {
            if !matches!(self.peek().kind, TokenKind::Text) {
                break;
            }

            lines.push(self.advance().lexeme);
        }

        Block::Paragraph(lines.join("\n"))
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();

        if !self.is_at_end() {
            self.current += 1;
        }

        token
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
}