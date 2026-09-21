use crate::ast::{
    Block,
    Document,
    FrontMatter,
    FrontMatterEntry,
    Inline,
    ListItem,
    Table,
    TableAlignment,
};
use crate::lexer::{Position, Token, TokenKind};

use super::error::ParserError;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Document, ParserError> {
        let mut blocks = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();

            if self.is_at_end() {
                break;
            }

            if self.is_front_matter_start() {
                blocks.push(self.parse_front_matter()?);
                continue;
            }

            blocks.push(self.parse_single_block()?);
        }

        Ok(Document { blocks })
    }

    fn parse_single_block(&mut self) -> Result<Block, ParserError> {
        if self.is_custom_block_start() {
            return self.parse_custom_block();
        }

        if self.is_comment_start() {
            return self.parse_comment();
        }

        if self.is_table_start() {
            return self.parse_table();
        }

        if self.is_horizontal_rule() {
            return Ok(self.parse_horizontal_rule());
        }

        if self.is_code_block_start() {
            return self.parse_code_block();
        }

        if self.is_heading_start() {
            return self.parse_heading();
        }

        if self.is_ordered_list_start() {
            return self.parse_ordered_list(0);
        }

        match self.peek().kind {
            TokenKind::Minus => self.parse_unordered_list(0),
            TokenKind::GreaterThan => self.parse_quote(),
            TokenKind::Indent(_) => {
                if self.is_indented_code_block() {
                    Ok(self.parse_indented_code_block())
                } else {
                    self.parse_paragraph()
                }
            }
            _ => self.parse_paragraph(),
        }
    }

    fn parse_paragraph(&mut self) -> Result<Block, ParserError> {
        let mut content = Vec::new();

        while !self.is_at_end() && !self.check(TokenKind::Newline) {
            let inline = self.parse_inline()?;
            Self::push_inline(&mut content, inline);
        }

        Self::trim_inline_whitespace(&mut content);

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        Ok(Block::Paragraph(content))
    }

    fn parse_heading(&mut self) -> Result<Block, ParserError> {
        let mut level = 0;

        while self.check(TokenKind::Hash) && level < 6 {
            self.advance();
            level += 1;
        }

        if !self.check(TokenKind::Text) || !self.peek().lexeme.starts_with(' ') {
            return Err(ParserError::UnexpectedToken {
                expected: "a space after heading marker".into(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            });
        }

        let mut content = self.parse_inline_until_newline(true)?;
        Self::trim_inline_whitespace(&mut content);

        let anchor = if self.is_anchor_start() {
            Some(self.parse_anchor()?)
        } else {
            None
        };

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        Ok(Block::Heading {
            level,
            content,
            anchor,
        })
    }

    fn parse_anchor(&mut self) -> Result<String, ParserError> {
        let position = self.peek().position;

        self.expect(TokenKind::LeftBrace, "{ token")?;
        self.expect(TokenKind::Hash, "# token")?;

        let mut value = String::new();

        while !self.is_at_end()
            && !self.check(TokenKind::RightBrace)
            && !self.check(TokenKind::Newline)
        {
            value.push_str(&self.advance().lexeme);
        }

        if !self.check(TokenKind::RightBrace) {
            return Err(ParserError::UnexpectedToken {
                expected: "} token".into(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            });
        }

        self.advance();

        if value.is_empty() || !Self::is_valid_anchor(&value) {
            return Err(ParserError::InvalidAnchor { value, position });
        }

        Ok(value)
    }

    fn is_valid_anchor(value: &str) -> bool {
        !value.is_empty()
            && value.chars().all(|ch| {
                ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
            })
    }

    fn parse_inline_until_newline(
        &mut self,
        stop_at_anchor: bool,
    ) -> Result<Vec<Inline>, ParserError> {
        let mut content = Vec::new();

        while !self.is_at_end()
            && !self.check(TokenKind::Newline)
            && (!stop_at_anchor || !self.is_anchor_start())
        {
            let inline = self.parse_inline()?;
            Self::push_inline(&mut content, inline);
        }

        Ok(content)
    }

    fn parse_inline(&mut self) -> Result<Inline, ParserError> {
        match self.peek().kind {
            TokenKind::Text => Ok(Inline::Text(self.advance().lexeme)),
            TokenKind::Star => self.parse_emphasis(),
            TokenKind::DoubleStar => self.parse_strong(),
            TokenKind::Underscore => self.parse_emphasis(),
            TokenKind::DoubleUnderscore => self.parse_strong(),
            TokenKind::DoubleTilde => self.parse_strikethrough(),
            TokenKind::Backtick => self.parse_code(),
            TokenKind::LeftBracket => self.parse_link(),
            TokenKind::Exclamation => self.parse_image(),
            TokenKind::Backslash => Ok(self.parse_escape()),
            TokenKind::Hash
            | TokenKind::Tilde
            | TokenKind::LeftParen
            | TokenKind::RightParen
            | TokenKind::RightBracket
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Dot
            | TokenKind::GreaterThan
            | TokenKind::LessThan
            | TokenKind::LeftBrace
            | TokenKind::RightBrace
            | TokenKind::Pipe
            | TokenKind::Colon => Ok(Inline::Text(self.advance().lexeme)),
            TokenKind::Indent(_) => Ok(Inline::Text(self.advance().lexeme)),
            TokenKind::Newline | TokenKind::Eof => Err(ParserError::UnexpectedToken {
                expected: "inline content".into(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            }),
        }
    }

    fn parse_emphasis(&mut self) -> Result<Inline, ParserError> {
        let delimiter = self.advance();
        let kind = delimiter.kind;
        let content = self.parse_until(kind)?;
        Ok(Inline::Emphasis(content))
    }

    fn parse_strong(&mut self) -> Result<Inline, ParserError> {
        let delimiter = self.advance();

        let triple_emphasis =
            (delimiter.kind == TokenKind::DoubleStar && self.check(TokenKind::Star))
                || (delimiter.kind == TokenKind::DoubleUnderscore
                    && self.check(TokenKind::Underscore));

        if triple_emphasis {
            let nested_delimiter = self.advance();
            let content = self.parse_until(delimiter.kind)?;

            if !self.check(nested_delimiter.kind) {
                return Err(ParserError::UnclosedDelimiter {
                    delimiter: nested_delimiter.lexeme,
                    position: nested_delimiter.position,
                });
            }

            self.advance();

            return Ok(Inline::Strong(vec![Inline::Emphasis(content)]));
        }

        let content = self.parse_until(delimiter.kind)?;
        Ok(Inline::Strong(content))
    }

    fn parse_strikethrough(&mut self) -> Result<Inline, ParserError> {
        let delimiter = self.advance();
        let content = self.parse_until(delimiter.kind)?;
        Ok(Inline::Strikethrough(content))
    }

    fn parse_code(&mut self) -> Result<Inline, ParserError> {
        let delimiter = self.advance();
        let mut code = String::new();

        while !self.is_at_end() && !self.check(TokenKind::Backtick) {
            if self.check(TokenKind::Newline) {
                return Err(ParserError::UnclosedDelimiter {
                    delimiter: "`".into(),
                    position: delimiter.position,
                });
            }

            code.push_str(&self.advance().lexeme);
        }

        if !self.check(TokenKind::Backtick) {
            return Err(ParserError::UnclosedDelimiter {
                delimiter: "`".into(),
                position: delimiter.position,
            });
        }

        self.advance();
        Ok(Inline::Code(code))
    }

    fn parse_link(&mut self) -> Result<Inline, ParserError> {
        let start_position = self.peek().position;
        self.advance();

        let text = self.parse_until(TokenKind::RightBracket)?;

        self.expect(TokenKind::LeftParen, "( token after link text")?;

        let url = self.parse_raw_until(TokenKind::RightParen, "link URL", start_position)?;

        Ok(Inline::Link { text, url })
    }

    fn parse_image(&mut self) -> Result<Inline, ParserError> {
        let start_position = self.peek().position;
        self.advance();
        self.expect(TokenKind::LeftBracket, "[ token after !")?;

        let alt = self.parse_until(TokenKind::RightBracket)?;
        self.expect(TokenKind::LeftParen, "( token after image alt")?;
        let url = self.parse_raw_until(TokenKind::RightParen, "image URL", start_position)?;

        Ok(Inline::Image {
            alt: Self::inline_to_text(&alt),
            url,
        })
    }

    fn parse_escape(&mut self) -> Inline {
        self.advance();

        if self.is_at_end() || self.check(TokenKind::Newline) {
            return Inline::Text("\\".into());
        }

        Inline::Text(self.advance().lexeme)
    }

    fn parse_until(&mut self, closing: TokenKind) -> Result<Vec<Inline>, ParserError> {
        let opening = if self.current > 0 {
            self.tokens[self.current - 1].clone()
        } else {
            self.peek().clone()
        };

        let mut content = Vec::new();

        while !self.is_at_end()
            && !self.check(closing)
            && !self.check(TokenKind::Newline)
        {
            let inline = self.parse_inline()?;
            Self::push_inline(&mut content, inline);
        }

        if self.check(closing) {
            self.advance();
            return Ok(content);
        }

        Err(ParserError::UnclosedDelimiter {
            delimiter: opening.lexeme,
            position: opening.position,
        })
    }

    fn parse_raw_until(
        &mut self,
        closing: TokenKind,
        expected: &str,
        position: Position,
    ) -> Result<String, ParserError> {
        let mut value = String::new();

        while !self.is_at_end()
            && !self.check(closing)
            && !self.check(TokenKind::Newline)
        {
            value.push_str(&self.advance().lexeme);
        }

        if !self.check(closing) {
            return Err(ParserError::UnexpectedToken {
                expected: format!("{expected} followed by a closing delimiter"),
                found: self.peek().lexeme.clone(),
                position,
            });
        }

        self.advance();
        Ok(value.trim().to_string())
    }

    fn parse_unordered_list(&mut self, indent: usize) -> Result<Block, ParserError> {
        let mut items = Vec::new();

        while self.is_unordered_list_at(indent) {
            self.consume_exact_indent(indent);
            self.advance();

            let checked = self.parse_checkbox();
            let mut content = self.parse_inline_until_newline(false)?;

            Self::trim_inline_whitespace(&mut content);

            let mut blocks = Vec::new();
            if !content.is_empty() {
                blocks.push(Block::Paragraph(content));
            }

            if self.check(TokenKind::Newline) {
                self.advance();
            }

            self.parse_nested_blocks(indent, &mut blocks)?;

            items.push(ListItem { checked, blocks });
        }

        Ok(Block::UnorderedList { items })
    }

    fn parse_ordered_list(&mut self, indent: usize) -> Result<Block, ParserError> {
        let mut items = Vec::new();

        while self.is_ordered_list_at(indent) {
            self.consume_exact_indent(indent);
            self.advance();
            self.advance();

            let mut content = self.parse_inline_until_newline(false)?;
            Self::trim_inline_whitespace(&mut content);

            let mut blocks = Vec::new();
            if !content.is_empty() {
                blocks.push(Block::Paragraph(content));
            }

            if self.check(TokenKind::Newline) {
                self.advance();
            }

            self.parse_nested_blocks(indent, &mut blocks)?;

            items.push(ListItem {
                checked: None,
                blocks,
            });
        }

        Ok(Block::OrderedList { items })
    }

    fn parse_checkbox(&mut self) -> Option<bool> {
        if !self.check(TokenKind::Text) {
            return None;
        }

        let space = &self.peek().lexeme;
        if !space.chars().all(char::is_whitespace) || space.is_empty() {
            return None;
        }

        if self.current + 3 >= self.tokens.len() {
            return None;
        }

        if self.tokens[self.current + 1].kind != TokenKind::LeftBracket {
            return None;
        }

        if self.tokens[self.current + 2].kind != TokenKind::Text {
            return None;
        }

        let marker = self.tokens[self.current + 2].lexeme.trim();

        let checked = match marker {
            "" => false,
            "x" | "X" => true,
            _ => return None,
        };

        if self.tokens[self.current + 3].kind != TokenKind::RightBracket {
            return None;
        }

        self.advance();
        self.advance();
        self.advance();
        self.advance();

        Some(checked)
    }

    fn parse_nested_blocks(
        &mut self,
        parent_indent: usize,
        blocks: &mut Vec<Block>,
    ) -> Result<(), ParserError> {
        let nested_indent = match self.peek().kind {
            TokenKind::Indent(value) if value > parent_indent => value,
            _ => return Ok(()),
        };

        if self.is_unordered_list_at(nested_indent) {
            blocks.push(self.parse_unordered_list(nested_indent)?);
        } else if self.is_ordered_list_at(nested_indent) {
            blocks.push(self.parse_ordered_list(nested_indent)?);
        } else if nested_indent >= 4 {
            blocks.push(self.parse_indented_code_block());
        }

        Ok(())
    }

    fn parse_quote(&mut self) -> Result<Block, ParserError> {
        let depth = self.quote_depth();
        let mut paragraph = Vec::new();

        loop {
            if self.quote_depth() != depth {
                break;
            }

            for _ in 0..depth {
                self.advance();
            }

            let mut content = self.parse_inline_until_newline(false)?;
            Self::trim_inline_whitespace(&mut content);

            if !content.is_empty() {
                if !paragraph.is_empty() {
                    Self::push_inline(&mut paragraph, Inline::Text(" ".into()));
                }
                for inline in content {
                    Self::push_inline(&mut paragraph, inline);
                }
            }

            if self.check(TokenKind::Newline) {
                self.advance();
            } else {
                break;
            }

            if !self.is_line_start() || !matches!(self.peek().kind, TokenKind::GreaterThan) {
                break;
            }
        }

        let inner = if paragraph.is_empty() {
            Vec::new()
        } else {
            vec![Block::Paragraph(paragraph)]
        };

        let mut block = Block::Quote { blocks: inner };
        for _ in 1..depth {
            block = Block::Quote { blocks: vec![block] };
        }

        Ok(block)
    }

    fn quote_depth(&self) -> usize {
        let mut index = self.current;
        let mut depth = 0;

        while index < self.tokens.len()
            && self.tokens[index].kind == TokenKind::GreaterThan
        {
            depth += 1;
            index += 1;
        }

        depth
    }

    fn parse_horizontal_rule(&mut self) -> Block {
        self.advance();
        self.advance();
        self.advance();
        Block::HorizontalRule
    }

    fn parse_code_block(&mut self) -> Result<Block, ParserError> {
        let start_position = self.peek().position;

        self.advance();
        self.advance();
        self.advance();

        let language = if self.check(TokenKind::Text) {
            let value = self.advance().lexeme.trim().to_string();
            (!value.is_empty()).then_some(value)
        } else {
            None
        };

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        let mut code = String::new();

        while !self.is_at_end() {
            if self.is_fenced_code_end() {
                self.advance();
                self.advance();
                self.advance();

                if self.check(TokenKind::Newline) {
                    self.advance();
                }

                return Ok(Block::CodeBlock { language, code });
            }

            if self.check(TokenKind::Newline) {
                code.push('\n');
                self.advance();
                continue;
            }

            code.push_str(&self.advance().lexeme);
        }

        Err(ParserError::UnclosedBlock {
            kind: "fenced code block".into(),
            position: start_position,
        })
    }

    fn parse_indented_code_block(&mut self) -> Block {
        let mut code = String::new();

        while !self.is_at_end() {
            let indent = match self.peek().kind {
                TokenKind::Indent(value) if value >= 4 => value,
                _ => break,
            };

            self.advance();

            if indent > 4 {
                code.push_str(&" ".repeat(indent - 4));
            }

            while !self.is_at_end() && !self.check(TokenKind::Newline) {
                code.push_str(&self.advance().lexeme);
            }

            if self.check(TokenKind::Newline) {
                code.push('\n');
                self.advance();
            }
        }

        if code.ends_with('\n') {
            code.pop();
        }

        Block::CodeBlock {
            language: None,
            code,
        }
    }

    fn is_table_start(&self) -> bool {
        if !self.is_line_start() || !self.check(TokenKind::Pipe) {
            return false;
        }

        let mut index = self.current;
        while index < self.tokens.len() && self.tokens[index].kind != TokenKind::Newline {
            index += 1;
        }

        if index >= self.tokens.len() {
            return false;
        }

        index += 1;
        self.is_table_separator_at(index)
    }

    fn is_table_separator_at(&self, mut index: usize) -> bool {
        if index >= self.tokens.len() || self.tokens[index].kind != TokenKind::Pipe {
            return false;
        }

        index += 1;
        let mut cells = 0;

        loop {
            while index < self.tokens.len()
                && self.tokens[index].kind == TokenKind::Text
                && self.tokens[index].lexeme.chars().all(char::is_whitespace)
            {
                index += 1;
            }

            if index < self.tokens.len() && self.tokens[index].kind == TokenKind::Colon {
                index += 1;
            }

            let mut dashes = 0;
            while index < self.tokens.len() && self.tokens[index].kind == TokenKind::Minus {
                dashes += 1;
                index += 1;
            }

            if index < self.tokens.len() && self.tokens[index].kind == TokenKind::Colon {
                index += 1;
            }

            if dashes < 3 {
                return false;
            }

            cells += 1;

            while index < self.tokens.len()
                && self.tokens[index].kind == TokenKind::Text
                && self.tokens[index].lexeme.chars().all(char::is_whitespace)
            {
                index += 1;
            }

            if index >= self.tokens.len() {
                return true;
            }

            if self.tokens[index].kind == TokenKind::Newline {
                return cells > 0;
            }

            if self.tokens[index].kind != TokenKind::Pipe {
                return false;
            }

            index += 1;

            if index < self.tokens.len() && self.tokens[index].kind == TokenKind::Newline {
                return cells > 0;
            }
        }
    }

    fn parse_table(&mut self) -> Result<Block, ParserError> {
        let headers = self.parse_table_row()?;

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        let alignments = self.parse_table_alignment_row();

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        let mut rows = Vec::new();

        while self.check(TokenKind::Pipe) {
            rows.push(self.parse_table_row()?);

            if self.check(TokenKind::Newline) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Block::Table(Table {
            headers,
            alignments,
            rows,
        }))
    }

    fn parse_table_row(&mut self) -> Result<Vec<Vec<Inline>>, ParserError> {
        let mut cells = Vec::new();

        if self.check(TokenKind::Pipe) {
            self.advance();
        }

        loop {
            let mut content = Vec::new();

            while !self.is_at_end()
                && !self.check(TokenKind::Pipe)
                && !self.check(TokenKind::Newline)
            {
                let inline = self.parse_inline()?;
                Self::push_inline(&mut content, inline);
            }

            Self::trim_inline_whitespace(&mut content);
            cells.push(content);

            if self.check(TokenKind::Pipe) {
                self.advance();
                if self.check(TokenKind::Newline) {
                    break;
                }
                continue;
            }

            break;
        }

        Ok(cells)
    }

    fn parse_table_alignment_row(&mut self) -> Vec<TableAlignment> {
        let mut alignments = Vec::new();

        if self.check(TokenKind::Pipe) {
            self.advance();
        }

        loop {
            while self.check_whitespace_text() {
                self.advance();
            }

            let mut left = false;
            let mut right = false;
            let mut dashes = 0;

            if self.check(TokenKind::Colon) {
                left = true;
                self.advance();
            }

            while self.check(TokenKind::Minus) {
                dashes += 1;
                self.advance();
            }

            if self.check(TokenKind::Colon) {
                right = true;
                self.advance();
            }

            alignments.push(if dashes >= 3 {
                match (left, right) {
                    (true, true) => TableAlignment::Center,
                    (true, false) => TableAlignment::Left,
                    (false, true) => TableAlignment::Right,
                    (false, false) => TableAlignment::None,
                }
            } else {
                TableAlignment::None
            });

            while self.check_whitespace_text() {
                self.advance();
            }

            if self.check(TokenKind::Pipe) {
                self.advance();
                if self.check(TokenKind::Newline) {
                    break;
                }
                continue;
            }

            break;
        }

        alignments
    }

    fn parse_comment(&mut self) -> Result<Block, ParserError> {
        let start_position = self.peek().position;

        self.advance();
        self.advance();
        self.advance();
        self.advance();

        let mut content = String::new();

        while !self.is_at_end() && !self.is_comment_end() {
            content.push_str(&self.advance().lexeme);
        }

        if !self.is_comment_end() {
            return Err(ParserError::UnclosedBlock {
                kind: "comment".into(),
                position: start_position,
            });
        }

        self.advance();
        self.advance();
        self.advance();

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        Ok(Block::Comment(content))
    }

    fn parse_front_matter(&mut self) -> Result<Block, ParserError> {
        let start_position = self.peek().position;

        self.advance();
        self.advance();
        self.advance();

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        let mut entries = Vec::new();

        while !self.is_at_end() && !self.is_horizontal_rule() {
            let mut line = String::new();

            while !self.is_at_end() && !self.check(TokenKind::Newline) {
                line.push_str(&self.advance().lexeme);
            }

            if self.check(TokenKind::Newline) {
                self.advance();
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                entries.push(FrontMatterEntry {
                    key: key.trim().to_string(),
                    value: value.trim().to_string(),
                });
            }
        }

        if !self.is_horizontal_rule() {
            return Err(ParserError::UnclosedBlock {
                kind: "front matter".into(),
                position: start_position,
            });
        }

        self.advance();
        self.advance();
        self.advance();

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        Ok(Block::FrontMatter(FrontMatter { entries }))
    }

    fn parse_custom_block(&mut self) -> Result<Block, ParserError> {
        let start_position = self.peek().position;

        self.advance();
        self.advance();
        self.advance();

        let kind = if self.check(TokenKind::Text) {
            self.advance().lexeme.trim().to_string()
        } else {
            String::new()
        };

        if kind.is_empty() {
            return Err(ParserError::UnexpectedToken {
                expected: "custom block name".into(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            });
        }

        if self.check(TokenKind::Newline) {
            self.advance();
        } else if !self.is_at_end() {
            return Err(ParserError::UnexpectedToken {
                expected: "newline after custom block name".into(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            });
        }

        let mut blocks = Vec::new();

        while !self.is_at_end() && !self.is_custom_block_end() {
            self.skip_newlines();

            if self.is_at_end() || self.is_custom_block_end() {
                break;
            }

            blocks.push(self.parse_single_block()?);
        }

        if !self.is_custom_block_end() {
            return Err(ParserError::UnclosedBlock {
                kind,
                position: start_position,
            });
        }

        self.advance();
        self.advance();
        self.advance();

        if self.check(TokenKind::Newline) {
            self.advance();
        }

        Ok(Block::Custom { kind, blocks })
    }

    fn is_front_matter_start(&self) -> bool {
        self.current == 0
            && self.current + 3 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Minus
            && self.tokens[self.current + 1].kind == TokenKind::Minus
            && self.tokens[self.current + 2].kind == TokenKind::Minus
            && self.tokens[self.current + 3].kind == TokenKind::Newline
    }

    fn is_heading_start(&self) -> bool {
        let mut index = self.current;
        let mut level = 0;

        while index < self.tokens.len()
            && self.tokens[index].kind == TokenKind::Hash
        {
            level += 1;
            index += 1;
        }

        if !(1..=6).contains(&level) {
            return false;
        }

        index < self.tokens.len()
            && self.tokens[index].kind == TokenKind::Text
            && self.tokens[index].lexeme.starts_with(' ')
    }

    fn is_anchor_start(&self) -> bool {
        self.check(TokenKind::LeftBrace)
            && self.current + 1 < self.tokens.len()
            && self.tokens[self.current + 1].kind == TokenKind::Hash
    }

    fn is_ordered_list_start(&self) -> bool {
        self.is_ordered_list_at(0)
    }

    fn is_ordered_list_at(&self, indent: usize) -> bool {
        let Some(index) = self.index_after_indent(indent) else {
            return false;
        };

        if index + 1 >= self.tokens.len()
            || self.tokens[index].kind != TokenKind::Text
            || self.tokens[index + 1].kind != TokenKind::Dot
        {
            return false;
        }

        let text = &self.tokens[index].lexeme;
        !text.is_empty() && text.chars().all(|ch| ch.is_ascii_digit())
    }

    fn is_unordered_list_at(&self, indent: usize) -> bool {
        let Some(index) = self.index_after_indent(indent) else {
            return false;
        };
        index < self.tokens.len() && self.tokens[index].kind == TokenKind::Minus
    }

    fn index_after_indent(&self, indent: usize) -> Option<usize> {
        if indent == 0 {
            if self.check(TokenKind::Indent(0)) {
                Some(self.current + 1)
            } else {
                Some(self.current)
            }
        } else if matches!(self.peek().kind, TokenKind::Indent(value) if value == indent) {
            Some(self.current + 1)
        } else {
            None
        }
    }

    fn consume_exact_indent(&mut self, indent: usize) {
        if indent > 0 && matches!(self.peek().kind, TokenKind::Indent(value) if value == indent) {
            self.advance();
        }
    }

    fn is_indented_code_block(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Indent(value) if value >= 4)
    }

    fn is_horizontal_rule(&self) -> bool {
        if !self.is_line_start() || self.current + 2 >= self.tokens.len() {
            return false;
        }

        if self.tokens[self.current].kind != TokenKind::Minus
            || self.tokens[self.current + 1].kind != TokenKind::Minus
            || self.tokens[self.current + 2].kind != TokenKind::Minus
        {
            return false;
        }

        matches!(
            self.tokens.get(self.current + 3).map(|token| token.kind),
            Some(TokenKind::Newline) | Some(TokenKind::Eof)
        )
    }

    fn is_code_block_start(&self) -> bool {
        self.is_line_start()
            && self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Backtick
            && self.tokens[self.current + 1].kind == TokenKind::Backtick
            && self.tokens[self.current + 2].kind == TokenKind::Backtick
    }

    fn is_fenced_code_end(&self) -> bool {
        self.is_line_start()
            && self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Backtick
            && self.tokens[self.current + 1].kind == TokenKind::Backtick
            && self.tokens[self.current + 2].kind == TokenKind::Backtick
            && matches!(
                self.tokens.get(self.current + 3).map(|token| token.kind),
                Some(TokenKind::Newline) | Some(TokenKind::Eof)
            )
    }

    fn is_comment_start(&self) -> bool {
        self.is_line_start()
            && self.current + 3 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::LessThan
            && self.tokens[self.current + 1].kind == TokenKind::Exclamation
            && self.tokens[self.current + 2].kind == TokenKind::Minus
            && self.tokens[self.current + 3].kind == TokenKind::Minus
    }

    fn is_comment_end(&self) -> bool {
        self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Minus
            && self.tokens[self.current + 1].kind == TokenKind::Minus
            && self.tokens[self.current + 2].kind == TokenKind::GreaterThan
    }

    fn is_custom_block_start(&self) -> bool {
        self.is_line_start()
            && self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Colon
            && self.tokens[self.current + 1].kind == TokenKind::Colon
            && self.tokens[self.current + 2].kind == TokenKind::Colon
            && self.current + 3 < self.tokens.len()
            && self.tokens[self.current + 3].kind == TokenKind::Text
            && !self.tokens[self.current + 3].lexeme.trim().is_empty()
    }

    fn is_custom_block_end(&self) -> bool {
        self.is_line_start()
            && self.current + 2 < self.tokens.len()
            && self.tokens[self.current].kind == TokenKind::Colon
            && self.tokens[self.current + 1].kind == TokenKind::Colon
            && self.tokens[self.current + 2].kind == TokenKind::Colon
            && matches!(
                self.tokens.get(self.current + 3).map(|token| token.kind),
                Some(TokenKind::Newline) | Some(TokenKind::Eof)
            )
    }

    fn check_whitespace_text(&self) -> bool {
        self.check(TokenKind::Text)
            && self.peek().lexeme.chars().all(char::is_whitespace)
    }

    fn is_line_start(&self) -> bool {
        self.current == 0 || self.tokens[self.current - 1].kind == TokenKind::Newline
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token, ParserError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(ParserError::UnexpectedToken {
                expected: expected.to_string(),
                found: self.peek().lexeme.clone(),
                position: self.peek().position,
            })
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
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
        self.peek().kind == TokenKind::Eof
    }

    fn inline_to_text(content: &[Inline]) -> String {
        let mut output = String::new();

        for inline in content {
            match inline {
                Inline::Text(text) | Inline::Code(text) => output.push_str(text),
                Inline::Emphasis(inner)
                | Inline::Strong(inner)
                | Inline::Strikethrough(inner) => {
                    output.push_str(&Self::inline_to_text(inner));
                }
                Inline::Link { text, .. } => output.push_str(&Self::inline_to_text(text)),
                Inline::Image { alt, .. } => output.push_str(alt),
            }
        }

        output
    }

    fn trim_inline_whitespace(content: &mut Vec<Inline>) {
        Self::trim_inline_start(content);
        Self::trim_inline_end(content);
    }

    fn trim_inline_start(content: &mut Vec<Inline>) {
        while let Some(first) = content.first_mut() {
            match first {
                Inline::Text(text) => {
                    let trimmed = text.trim_start().to_string();
                    if trimmed.is_empty() {
                        content.remove(0);
                    } else {
                        *text = trimmed;
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn trim_inline_end(content: &mut Vec<Inline>) {
        while let Some(last) = content.last_mut() {
            match last {
                Inline::Text(text) => {
                    let trimmed = text.trim_end().to_string();
                    if trimmed.is_empty() {
                        content.pop();
                    } else {
                        *text = trimmed;
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn push_inline(content: &mut Vec<Inline>, inline: Inline) {
        match (content.last_mut(), inline) {
            (Some(Inline::Text(existing)), Inline::Text(text)) => existing.push_str(&text),
            (_, inline) => content.push(inline),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Document, ParserError> {
        let tokens = Lexer::new(source).tokenize().unwrap();
        Parser::new(tokens).parse()
    }

    #[test]
    fn parses_paragraph() {
        assert_eq!(
            parse("Hello world").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![Inline::Text("Hello world".into())])]
            }
        );
    }

    #[test]
    fn parses_heading() {
        assert_eq!(
            parse("### Hello").unwrap(),
            Document {
                blocks: vec![Block::Heading {
                    level: 3,
                    content: vec![Inline::Text("Hello".into())],
                    anchor: None,
                }]
            }
        );
    }

    #[test]
    fn rejects_invalid_heading_without_space() {
        assert_eq!(
            parse("#Hello").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![Inline::Text("#Hello".into())])]
            }
        );
    }

    #[test]
    fn parses_emphasis_and_strong() {
        assert_eq!(
            parse("*hello* **world**").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![
                    Inline::Emphasis(vec![Inline::Text("hello".into())]),
                    Inline::Text(" ".into()),
                    Inline::Strong(vec![Inline::Text("world".into())]),
                ])]
            }
        );
    }

    #[test]
    fn parses_strong_emphasis() {
        assert_eq!(
            parse("***hello***").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![Inline::Strong(vec![
                    Inline::Emphasis(vec![Inline::Text("hello".into())]),
                ])])]
            }
        );

        assert_eq!(
            parse("___hello___").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![Inline::Strong(vec![
                    Inline::Emphasis(vec![Inline::Text("hello".into())]),
                ])])]
            }
        );
    }

    #[test]
    fn parses_strikethrough_and_code() {
        assert_eq!(
            parse("~~old~~ `code`").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![
                    Inline::Strikethrough(vec![Inline::Text("old".into())]),
                    Inline::Text(" ".into()),
                    Inline::Code("code".into()),
                ])]
            }
        );
    }

    #[test]
    fn parses_link_and_image() {
        assert_eq!(
            parse("[site](https://example.com) ![logo](kastel.png)").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![
                    Inline::Link {
                        text: vec![Inline::Text("site".into())],
                        url: "https://example.com".into(),
                    },
                    Inline::Text(" ".into()),
                    Inline::Image {
                        alt: "logo".into(),
                        url: "kastel.png".into(),
                    },
                ])]
            }
        );
    }

    #[test]
    fn parses_escape() {
        assert_eq!(
            parse(r"\*hello\*").unwrap(),
            Document {
                blocks: vec![Block::Paragraph(vec![Inline::Text("*hello*".into())])]
            }
        );
    }

    #[test]
    fn parses_unordered_list() {
        let document = parse("- one\n- two").unwrap();
        assert_eq!(document.blocks.len(), 1);
        assert!(matches!(document.blocks[0], Block::UnorderedList { .. }));
    }

    #[test]
    fn parses_checkbox() {
        let document = parse("- [ ] todo\n- [x] done").unwrap();
        match &document.blocks[0] {
            Block::UnorderedList { items } => {
                assert_eq!(items[0].checked, Some(false));
                assert_eq!(items[1].checked, Some(true));
            }
            _ => panic!("expected unordered list"),
        }
    }

    #[test]
    fn parses_nested_list() {
        let document = parse("- parent\n  - child").unwrap();
        match &document.blocks[0] {
            Block::UnorderedList { items } => {
                assert!(matches!(items[0].blocks[1], Block::UnorderedList { .. }));
            }
            _ => panic!("expected unordered list"),
        }
    }

    #[test]
    fn parses_ordered_list() {
        let document = parse("1. one\n2. two").unwrap();
        assert!(matches!(document.blocks[0], Block::OrderedList { .. }));
    }

    #[test]
    fn parses_quote() {
        let document = parse("> hello\n> world").unwrap();
        match &document.blocks[0] {
            Block::Quote { blocks } => assert_eq!(blocks.len(), 1),
            _ => panic!("expected quote"),
        }
    }

    #[test]
    fn parses_nested_quote() {
        let document = parse(">> nested").unwrap();
        match &document.blocks[0] {
            Block::Quote { blocks } => {
                assert!(matches!(blocks[0], Block::Quote { .. }));
            }
            _ => panic!("expected quote"),
        }
    }

    #[test]
    fn parses_horizontal_rule() {
        assert_eq!(
            parse("---").unwrap(),
            Document {
                blocks: vec![Block::HorizontalRule]
            }
        );
    }

    #[test]
    fn parses_fenced_code() {
        assert_eq!(
            parse("```rust\nlet x = 1;\n```").unwrap(),
            Document {
                blocks: vec![Block::CodeBlock {
                    language: Some("rust".into()),
                    code: "let x = 1;\n".into(),
                }]
            }
        );
    }

    #[test]
    fn rejects_unclosed_fenced_code() {
        assert!(matches!(
            parse("```rust\nlet x = 1;").unwrap_err(),
            ParserError::UnclosedBlock { .. }
        ));
    }

    #[test]
    fn parses_indented_code() {
        let document = parse("    let x = 1;").unwrap();
        assert_eq!(
            document.blocks,
            vec![Block::CodeBlock {
                language: None,
                code: "let x = 1;".into(),
            }]
        );
    }

    #[test]
    fn parses_anchor() {
        assert_eq!(
            parse("# Hello {#intro}").unwrap(),
            Document {
                blocks: vec![Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Hello".into())],
                    anchor: Some("intro".into()),
                }]
            }
        );
    }

    #[test]
    fn rejects_invalid_anchor() {
        assert!(matches!(
            parse("# Hello {#bad.id}").unwrap_err(),
            ParserError::InvalidAnchor { .. }
        ));
    }

    #[test]
    fn parses_comment() {
        assert_eq!(
            parse("<!-- Hello -->").unwrap(),
            Document {
                blocks: vec![Block::Comment(" Hello ".into())]
            }
        );
    }

    #[test]
    fn rejects_unclosed_comment() {
        assert!(matches!(
            parse("<!-- Hello").unwrap_err(),
            ParserError::UnclosedBlock { .. }
        ));
    }

    #[test]
    fn parses_multiline_comment() {
        assert_eq!(
            parse("<!--\nHello\nWorld\n-->").unwrap(),
            Document {
                blocks: vec![Block::Comment("\nHello\nWorld\n".into())]
            }
        );
    }

    #[test]
    fn parses_front_matter() {
        assert_eq!(
            parse("---\ntitle: Kastel\nauthor: Bruno\n---\n# Intro").unwrap(),
            Document {
                blocks: vec![
                    Block::FrontMatter(FrontMatter {
                        entries: vec![
                            FrontMatterEntry {
                                key: "title".into(),
                                value: "Kastel".into(),
                            },
                            FrontMatterEntry {
                                key: "author".into(),
                                value: "Bruno".into(),
                            },
                        ],
                    }),
                    Block::Heading {
                        level: 1,
                        content: vec![Inline::Text("Intro".into())],
                        anchor: None,
                    },
                ]
            }
        );
    }

    #[test]
    fn rejects_unclosed_front_matter() {
        assert!(matches!(
            parse("---\ntitle: Kastel").unwrap_err(),
            ParserError::UnclosedBlock { .. }
        ));
    }

    #[test]
    fn parses_custom_block() {
        assert_eq!(
            parse("::: note\nThis is a note.\n:::").unwrap(),
            Document {
                blocks: vec![Block::Custom {
                    kind: "note".into(),
                    blocks: vec![Block::Paragraph(vec![Inline::Text(
                        "This is a note.".into(),
                    )])],
                }]
            }
        );
    }

    #[test]
    fn parses_nested_custom_blocks() {
        let document = parse(
            "::: warning\nAttention.\n::: note\nAdditional information.\n:::\nEnd.\n:::",
        )
        .unwrap();

        match &document.blocks[0] {
            Block::Custom { kind, blocks } => {
                assert_eq!(kind, "warning");
                assert!(matches!(blocks[1], Block::Custom { .. }));
            }
            _ => panic!("expected custom block"),
        }
    }

    #[test]
    fn rejects_unclosed_custom_block() {
        assert!(matches!(
            parse("::: note\nHello").unwrap_err(),
            ParserError::UnclosedBlock { .. }
        ));
    }

    #[test]
    fn parses_table() {
        let document = parse("| Name | Age |\n| --- | --- |\n| Bruno | 30 |").unwrap();
        assert!(matches!(document.blocks[0], Block::Table(_)));
    }

    #[test]
    fn parses_empty_front_matter() {
        assert_eq!(
            parse("---\n---").unwrap(),
            Document {
                blocks: vec![Block::FrontMatter(FrontMatter { entries: vec![] })]
            }
        );
    }

    #[test]
    fn parses_horizontal_rule_at_eof() {
        assert_eq!(
            parse("---").unwrap(),
            Document {
                blocks: vec![Block::HorizontalRule]
            }
        );
    }

    #[test]
    fn parses_fenced_code_closing_at_eof() {
        assert_eq!(
            parse("```rust\nlet x = 1;\n```").unwrap(),
            Document {
                blocks: vec![Block::CodeBlock {
                    language: Some("rust".into()),
                    code: "let x = 1;\n".into(),
                }]
            }
        );
    }

    #[test]
    fn parses_custom_block_closing_at_eof() {
        assert_eq!(
            parse("::: note\nHello\n:::").unwrap(),
            Document {
                blocks: vec![Block::Custom {
                    kind: "note".into(),
                    blocks: vec![Block::Paragraph(vec![Inline::Text(
                        "Hello".into(),
                    )])],
                }]
            }
        );
    }
}
