pub mod block;

pub use block::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}