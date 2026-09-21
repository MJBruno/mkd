pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod renderer;

pub use ast::{
    Block,
    Document,
    FrontMatter,
    FrontMatterEntry,
    Inline,
    ListItem,
    Table,
    TableAlignment,
};
pub use error::Error;
pub use lexer::{Lexer, LexerError, Position, Token, TokenKind};
pub use parser::{Parser, ParserError};
pub use renderer::{HtmlRenderer, Renderer};

pub fn parse(source: &str) -> Result<Document, Error> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    Parser::new(tokens).parse().map_err(Error::from)
}

pub fn render_html(source: &str) -> Result<String, Error> {
    let document = parse(source)?;
    Ok(renderer::render_html_fragment(&document))
}

pub fn render_html_document(source: &str) -> Result<String, Error> {
    let document = parse(source)?;
    Ok(HtmlRenderer::new().render_document(&document))
}
