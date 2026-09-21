#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),

    Heading {
        level: u8,
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_document() {
        let document = Document {
            blocks: vec![
                Block::Heading {
                    level: 1,
                    text: "Kastel Markup".to_string(),
                },
                Block::Paragraph(
                    "Un langage de balisage simple.".to_string(),
                ),
            ],
        };

        assert_eq!(document.blocks.len(), 2);

        assert_eq!(
            document.blocks[0],
            Block::Heading {
                level: 1,
                text: "Kastel Markup".to_string(),
            }
        );

        assert_eq!(
            document.blocks[1],
            Block::Paragraph(
                "Un langage de balisage simple.".to_string(),
            )
        );
    }
}