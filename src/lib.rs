pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::{Block, Document};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;

pub fn parse(source: &str) -> Document {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);

    parser.parse()
}