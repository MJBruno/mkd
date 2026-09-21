#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(String),

    Heading { level: u8, text: String },
}

pub fn parse(source: &str) -> Document {
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();

    for line in source.lines() {
        if line.trim().is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph);
            continue;
        }

        if let Some((level, text)) = parse_heading(line) {
            flush_paragraph(&mut blocks, &mut paragraph);

            blocks.push(Block::Heading {
                level,
                text: text.to_string(),
            });

            continue;
        }

        paragraph.push(line);
    }

    flush_paragraph(&mut blocks, &mut paragraph);

    Document { blocks }
}

fn flush_paragraph(blocks: &mut Vec<Block>, paragraph: &mut Vec<&str>) {
    if paragraph.is_empty() {
        return;
    }

    blocks.push(Block::Paragraph(paragraph.join("\n")));
    paragraph.clear();
}

fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let bytes = line.as_bytes();

    let mut level = 0;

    while level < 6 && bytes.get(level) == Some(&b'#') {
        level += 1;
    }

    if level == 0 {
        return None;
    }

    let rest = &line[level..];

    if !rest.starts_with(' ') {
        return None;
    }

    let text = rest.trim_start();

    if text.is_empty() {
        return None;
    }

    Some((level as u8, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_paragraph() {
        let document = parse("Bonjour le monde.");

        assert_eq!(
            document.blocks,
            vec![Block::Paragraph("Bonjour le monde.".to_string())]
        );
    }

    #[test]
    fn parses_multiline_paragraph() {
        let document = parse(
            "Bonjour.\n\
             Ceci est la deuxième ligne.",
        );

        assert_eq!(
            document.blocks,
            vec![Block::Paragraph(
                "Bonjour.\nCeci est la deuxième ligne.".to_string()
            )]
        );
    }

    #[test]
    fn separates_paragraphs() {
        let document = parse(
            "Premier paragraphe.\n\n\
             Deuxième paragraphe.",
        );

        assert_eq!(
            document.blocks,
            vec![
                Block::Paragraph("Premier paragraphe.".to_string()),
                Block::Paragraph("Deuxième paragraphe.".to_string()),
            ]
        );
    }

    #[test]
    fn parses_heading() {
        let document = parse("# Kastel Markup");

        assert_eq!(
            document.blocks,
            vec![Block::Heading {
                level: 1,
                text: "Kastel Markup".to_string(),
            }]
        );
    }

    #[test]
    fn parses_all_heading_levels() {
        let document = parse(
            "# H1\n\
             ## H2\n\
             ### H3\n\
             #### H4\n\
             ##### H5\n\
             ###### H6",
        );

        assert_eq!(document.blocks.len(), 6);

        for (index, block) in document.blocks.iter().enumerate() {
            assert_eq!(
                block,
                &Block::Heading {
                    level: (index + 1) as u8,
                    text: format!("H{}", index + 1),
                }
            );
        }
    }

    #[test]
    fn hash_without_space_is_paragraph() {
        let document = parse("#Not a heading");

        assert_eq!(
            document.blocks,
            vec![Block::Paragraph("#Not a heading".to_string())]
        );
    }

    #[test]
    fn more_than_six_hashes_is_paragraph() {
        let document = parse("####### Not a heading");

        assert_eq!(
            document.blocks,
            vec![Block::Paragraph("####### Not a heading".to_string())]
        );
    }
}
