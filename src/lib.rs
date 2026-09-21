pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{Block, Document};
pub use lexer::{Lexer, LexerError};
pub use parser::Parser;

pub fn parse(
    source: &str,
) -> Result<Document, LexerError> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);

    Ok(parser.parse())
}