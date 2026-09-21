#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),

    Heading {
        level: u8,
        text: String,
    },
}