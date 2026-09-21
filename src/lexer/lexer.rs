use super::error::LexerError;
use super::token::{Position, Token, TokenKind};

pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    line: usize,
    column: usize,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            line: 1,
            column: 1,
            at_line_start: true,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            if self.at_line_start && (ch == ' ' || ch == '\t') {
                tokens.push(self.read_indent());
                self.at_line_start = false;
                continue;
            }

            match ch {
                '#' => tokens.push(self.single(TokenKind::Hash, '#')),
                '*' => tokens.push(self.lex_star()),
                '~' => tokens.push(self.lex_tilde()),
                '_' => tokens.push(self.lex_underscore()),
                '`' => tokens.push(self.single(TokenKind::Backtick, '`')),
                '\\' => tokens.push(self.single(TokenKind::Backslash, '\\')),
                '[' => tokens.push(self.single(TokenKind::LeftBracket, '[')),
                ']' => tokens.push(self.single(TokenKind::RightBracket, ']')),
                '(' => tokens.push(self.single(TokenKind::LeftParen, '(')),
                ')' => tokens.push(self.single(TokenKind::RightParen, ')')),
                '{' => tokens.push(self.single(TokenKind::LeftBrace, '{')),
                '}' => tokens.push(self.single(TokenKind::RightBrace, '}')),
                '|' => tokens.push(self.single(TokenKind::Pipe, '|')),
                ':' => tokens.push(self.single(TokenKind::Colon, ':')),
                '!' => tokens.push(self.single(TokenKind::Exclamation, '!')),
                '-' => tokens.push(self.single(TokenKind::Minus, '-')),
                '+' => tokens.push(self.single(TokenKind::Plus, '+')),
                '.' => tokens.push(self.single(TokenKind::Dot, '.')),
                '>' => tokens.push(self.single(TokenKind::GreaterThan, '>')),
                '<' => tokens.push(self.single(TokenKind::LessThan, '<')),
                '\n' => {
                    tokens.push(self.single(TokenKind::Newline, '\n'));
                    self.at_line_start = true;
                }
                '\r' => {
                    let position = self.position();
                    self.chars.next();

                    if self.peek() == Some('\n') {
                        self.chars.next();
                    }

                    self.line += 1;
                    self.column = 1;
                    self.at_line_start = true;
                    tokens.push(Token::new(TokenKind::Newline, "\n", position));
                }
                _ => tokens.push(self.read_text()),
            }
        }

        tokens.push(Token::new(TokenKind::Eof, "", self.position()));
        Ok(tokens)
    }

    fn lex_star(&mut self) -> Token {
        let position = self.position();
        self.advance();
        if self.peek() == Some('*') {
            self.advance();
            return Token::new(TokenKind::DoubleStar, "**", position);
        }
        Token::new(TokenKind::Star, "*", position)
    }

    fn lex_tilde(&mut self) -> Token {
        let position = self.position();
        self.advance();
        if self.peek() == Some('~') {
            self.advance();
            return Token::new(TokenKind::DoubleTilde, "~~", position);
        }
        Token::new(TokenKind::Tilde, "~", position)
    }

    fn lex_underscore(&mut self) -> Token {
        let position = self.position();
        self.advance();
        if self.peek() == Some('_') {
            self.advance();
            return Token::new(TokenKind::DoubleUnderscore, "__", position);
        }
        Token::new(TokenKind::Underscore, "_", position)
    }

    fn single(&mut self, kind: TokenKind, ch: char) -> Token {
        let position = self.position();
        self.advance();
        Token::new(kind, ch.to_string(), position)
    }

    fn read_indent(&mut self) -> Token {
        let position = self.position();
        let mut count = 0;

        while let Some(ch) = self.peek() {
            match ch {
                ' ' => {
                    count += 1;
                    self.advance();
                }
                '\t' => {
                    count += 4;
                    self.advance();
                }
                _ => break,
            }
        }

        Token::new(TokenKind::Indent(count), " ".repeat(count), position)
    }

    fn read_text(&mut self) -> Token {
        let position = self.position();
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '#' | '*' | '~' | '`' | '_' | '\\'
                | '[' | ']' | '(' | ')' | '!' | '-'
                | '+' | '.' | '>' | '<' | '|' | ':'
                | '{' | '}' | '\n' | '\r' => break,
                _ => {
                    text.push(ch);
                    self.advance();
                }
            }
        }

        Token::new(TokenKind::Text, text, position)
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
            self.at_line_start = true;
        } else {
            self.column += 1;
            self.at_line_start = false;
        }

        Some(ch)
    }

    fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn lexes_emphasis() {
        assert_eq!(
            kinds("*hello*"),
            vec![TokenKind::Star, TokenKind::Text, TokenKind::Star, TokenKind::Eof]
        );
    }

    #[test]
    fn lexes_strong() {
        assert_eq!(
            kinds("**hello**"),
            vec![
                TokenKind::DoubleStar,
                TokenKind::Text,
                TokenKind::DoubleStar,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_strikethrough() {
        assert_eq!(
            kinds("~~hello~~"),
            vec![
                TokenKind::DoubleTilde,
                TokenKind::Text,
                TokenKind::DoubleTilde,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_inline_code() {
        assert_eq!(
            kinds("`hello`"),
            vec![
                TokenKind::Backtick,
                TokenKind::Text,
                TokenKind::Backtick,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_heading() {
        assert_eq!(
            kinds("### Hello"),
            vec![
                TokenKind::Hash,
                TokenKind::Hash,
                TokenKind::Hash,
                TokenKind::Text,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_combined_emphasis() {
        assert_eq!(
            kinds("***hello***"),
            vec![
                TokenKind::DoubleStar,
                TokenKind::Star,
                TokenKind::Text,
                TokenKind::DoubleStar,
                TokenKind::Star,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_link() {
        assert_eq!(
            kinds("[hello](https://example.com)"),
            vec![
                TokenKind::LeftBracket,
                TokenKind::Text,
                TokenKind::RightBracket,
                TokenKind::LeftParen,
                TokenKind::Text,
                TokenKind::Colon,
                TokenKind::Text,
                TokenKind::Dot,
                TokenKind::Text,
                TokenKind::RightParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_image() {
        assert_eq!(
            kinds("![Kastel](kastel.png)"),
            vec![
                TokenKind::Exclamation,
                TokenKind::LeftBracket,
                TokenKind::Text,
                TokenKind::RightBracket,
                TokenKind::LeftParen,
                TokenKind::Text,
                TokenKind::Dot,
                TokenKind::Text,
                TokenKind::RightParen,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_unordered_list() {
        assert_eq!(
            kinds("- hello"),
            vec![TokenKind::Minus, TokenKind::Text, TokenKind::Eof]
        );
    }

    #[test]
    fn lexes_ordered_list() {
        let tokens = Lexer::new("1. hello").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Text);
        assert_eq!(tokens[0].lexeme, "1");
        assert_eq!(tokens[1].kind, TokenKind::Dot);
        assert_eq!(tokens[2].lexeme, " hello");
    }

    #[test]
    fn lexes_quote() {
        assert_eq!(
            kinds("> hello"),
            vec![TokenKind::GreaterThan, TokenKind::Text, TokenKind::Eof]
        );
    }

    #[test]
    fn lexes_horizontal_rule() {
        assert_eq!(
            kinds("---"),
            vec![TokenKind::Minus, TokenKind::Minus, TokenKind::Minus, TokenKind::Eof]
        );
    }

    #[test]
    fn lexes_underscore_emphasis() {
        assert_eq!(
            kinds("_hello_"),
            vec![
                TokenKind::Underscore,
                TokenKind::Text,
                TokenKind::Underscore,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_escaped_markers() {
        assert_eq!(
            kinds(r"\*hello\*"),
            vec![
                TokenKind::Backslash,
                TokenKind::Star,
                TokenKind::Text,
                TokenKind::Backslash,
                TokenKind::Star,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_checkbox() {
        let tokens = Lexer::new("- [x] task").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Minus);
        assert_eq!(tokens[1].lexeme, " ");
        assert_eq!(tokens[2].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[3].lexeme, "x");
        assert_eq!(tokens[4].kind, TokenKind::RightBracket);
    }

    #[test]
    fn lexes_indentation() {
        let tokens = Lexer::new("  - child").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Indent(2));
        assert_eq!(tokens[1].kind, TokenKind::Minus);
    }

    #[test]
    fn lexes_nested_list_indentation() {
        let tokens = Lexer::new("- parent\n  - child").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Minus);
        assert_eq!(tokens[1].lexeme, " parent");
        assert_eq!(tokens[2].kind, TokenKind::Newline);
        assert_eq!(tokens[3].kind, TokenKind::Indent(2));
        assert_eq!(tokens[4].kind, TokenKind::Minus);
    }

    #[test]
    fn lexes_comment() {
        assert_eq!(
            kinds("<!-- hello -->"),
            vec![
                TokenKind::LessThan,
                TokenKind::Exclamation,
                TokenKind::Minus,
                TokenKind::Minus,
                TokenKind::Text,
                TokenKind::Minus,
                TokenKind::Minus,
                TokenKind::GreaterThan,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_custom_block() {
        assert_eq!(
            kinds("::: note"),
            vec![
                TokenKind::Colon,
                TokenKind::Colon,
                TokenKind::Colon,
                TokenKind::Text,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn handles_crlf() {
        let tokens = Lexer::new("hello\r\nworld").tokenize().unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].lexeme, "world");
    }

    #[test]
    fn handles_unicode() {
        let tokens = Lexer::new("héllo 世界").tokenize().unwrap();
        assert_eq!(tokens[0].lexeme, "héllo 世界");
    }
}
