use std::fmt::Write;

use crate::ast::{
    Block,
    Document,
    Inline,
    ListItem,
    Table,
    TableAlignment,
};

use super::Renderer;

#[derive(Debug, Default, Clone, Copy)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub const fn new() -> Self {
        Self
    }

    pub fn render_document(&self, document: &Document) -> String {
        let title = self.document_title(document);
        let language = self.document_language(document);

        let mut output = String::new();

        output.push_str("<!doctype html>\n");
        writeln!(output, r#"<html lang="{}">"#, escape_html_attribute(&language)).unwrap();
        output.push_str("<head>\n");
        output.push_str("    <meta charset=\"utf-8\">\n");
        output.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
        );
        writeln!(output, "    <title>{}</title>", escape_html(&title)).unwrap();
        output.push_str("</head>\n");
        output.push_str("<body>\n");

        let mut body = String::new();
        self.render_blocks(&document.blocks, &mut body);

        for line in body.lines() {
            output.push_str("    ");
            output.push_str(line);
            output.push('\n');
        }

        output.push_str("</body>\n");
        output.push_str("</html>\n");

        output
    }

    fn render_blocks(&self, blocks: &[Block], output: &mut String) {
        for block in blocks {
            self.render_block(block, output);
        }
    }

    fn render_block(&self, block: &Block, output: &mut String) {
        match block {
            Block::Paragraph(content) => {
                output.push_str("<p>");
                self.render_inlines(content, output);
                output.push_str("</p>\n");
            }
            Block::Heading {
                level,
                content,
                anchor,
            } => {
                let level = (*level).clamp(1, 6);
                write!(output, "<h{level}").unwrap();

                if let Some(anchor) = anchor {
                    write!(output, r#" id="{}""#, escape_html_attribute(anchor)).unwrap();
                }

                write!(output, ">").unwrap();
                self.render_inlines(content, output);
                writeln!(output, "</h{level}>").unwrap();
            }
            Block::Comment(content) => {
                output.push_str("<!--");
                output.push_str(&escape_html_comment(content));
                output.push_str("-->\n");
            }
            Block::FrontMatter(_) => {}
            Block::UnorderedList { items } => {
                output.push_str("<ul>\n");
                for item in items {
                    self.render_list_item(item, output);
                }
                output.push_str("</ul>\n");
            }
            Block::OrderedList { items } => {
                output.push_str("<ol>\n");
                for item in items {
                    self.render_list_item(item, output);
                }
                output.push_str("</ol>\n");
            }
            Block::Quote { blocks } => {
                output.push_str("<blockquote>\n");
                self.render_blocks(blocks, output);
                output.push_str("</blockquote>\n");
            }
            Block::HorizontalRule => {
                output.push_str("<hr>\n");
            }
            Block::CodeBlock { language, code } => {
                output.push_str("<pre><code");

                if let Some(language) = language {
                    let language = language.trim();
                    if !language.is_empty() {
                        write!(output, r#" class="language-{}""#, escape_html_attribute(language))
                            .unwrap();
                    }
                }

                output.push('>');
                output.push_str(&escape_html(code));
                output.push_str("</code></pre>\n");
            }
            Block::Table(table) => self.render_table(table, output),
            Block::Custom { kind, blocks } => {
                self.render_custom_block(kind, blocks, output);
            }
        }
    }

    fn render_list_item(&self, item: &ListItem, output: &mut String) {
        output.push_str("<li");

        if item.checked.is_some() {
            output.push_str(r#" class="task-list-item""#);
        }

        output.push('>');

        if item.blocks.is_empty() {
            if let Some(checked) = item.checked {
                self.render_checkbox(checked, output);
            }
            output.push_str("</li>\n");
            return;
        }

        if let Block::Paragraph(content) = &item.blocks[0] {
            if let Some(checked) = item.checked {
                self.render_checkbox(checked, output);
                output.push(' ');
            }

            self.render_inlines(content, output);

            for block in &item.blocks[1..] {
                self.render_block(block, output);
            }
        } else {
            if let Some(checked) = item.checked {
                self.render_checkbox(checked, output);
                output.push(' ');
            }

            self.render_blocks(&item.blocks, output);
        }

        output.push_str("</li>\n");
    }

    fn render_checkbox(&self, checked: bool, output: &mut String) {
        if checked {
            output.push_str(r#"<input type="checkbox" disabled checked>"#);
        } else {
            output.push_str(r#"<input type="checkbox" disabled>"#);
        }
    }

    fn render_table(&self, table: &Table, output: &mut String) {
        let column_count = table
            .headers
            .len()
            .max(table.alignments.len())
            .max(table.rows.iter().map(Vec::len).max().unwrap_or(0));

        if column_count == 0 {
            return;
        }

        output.push_str("<table>\n<thead>\n<tr>\n");

        for index in 0..column_count {
            output.push_str("<th");

            if let Some(alignment) = table.alignments.get(index) {
                self.render_alignment_attribute(*alignment, output);
            }

            output.push('>');
            if let Some(header) = table.headers.get(index) {
                self.render_inlines(header, output);
            }
            output.push_str("</th>\n");
        }

        output.push_str("</tr>\n</thead>\n");

        if !table.rows.is_empty() {
            output.push_str("<tbody>\n");

            for row in &table.rows {
                output.push_str("<tr>\n");

                for index in 0..column_count {
                    output.push_str("<td");

                    if let Some(alignment) = table.alignments.get(index) {
                        self.render_alignment_attribute(*alignment, output);
                    }

                    output.push('>');
                    if let Some(cell) = row.get(index) {
                        self.render_inlines(cell, output);
                    }
                    output.push_str("</td>\n");
                }

                output.push_str("</tr>\n");
            }

            output.push_str("</tbody>\n");
        }

        output.push_str("</table>\n");
    }

    fn render_alignment_attribute(
        &self,
        alignment: TableAlignment,
        output: &mut String,
    ) {
        let value = match alignment {
            TableAlignment::Left => "left",
            TableAlignment::Center => "center",
            TableAlignment::Right => "right",
            TableAlignment::None => return,
        };

        write!(output, r#" style="text-align:{}""#, value).unwrap();
    }

    fn render_custom_block(
        &self,
        kind: &str,
        blocks: &[Block],
        output: &mut String,
    ) {
        let class_name = custom_class_name(kind);

        writeln!(
            output,
            r#"<div class="custom-block custom-{}">"#,
            escape_html_attribute(&class_name)
        )
        .unwrap();

        self.render_blocks(blocks, output);
        output.push_str("</div>\n");
    }

    fn render_inlines(&self, inlines: &[Inline], output: &mut String) {
        for inline in inlines {
            self.render_inline(inline, output);
        }
    }

    fn render_inline(&self, inline: &Inline, output: &mut String) {
        match inline {
            Inline::Text(text) => output.push_str(&escape_html(text)),
            Inline::Emphasis(content) => {
                output.push_str("<em>");
                self.render_inlines(content, output);
                output.push_str("</em>");
            }
            Inline::Strong(content) => {
                output.push_str("<strong>");
                self.render_inlines(content, output);
                output.push_str("</strong>");
            }
            Inline::Strikethrough(content) => {
                output.push_str("<del>");
                self.render_inlines(content, output);
                output.push_str("</del>");
            }
            Inline::Code(code) => {
                output.push_str("<code>");
                output.push_str(&escape_html(code));
                output.push_str("</code>");
            }
            Inline::Link { text, url } => {
                write!(output, r#"<a href="{}">"#, safe_url(url)).unwrap();
                self.render_inlines(text, output);
                output.push_str("</a>");
            }
            Inline::Image { alt, url } => {
                write!(
                    output,
                    r#"<img src="{}" alt="{}">"#,
                    safe_url(url),
                    escape_html_attribute(alt)
                )
                .unwrap();
            }
        }
    }

    fn document_title(&self, document: &Document) -> String {
        for block in &document.blocks {
            let Block::FrontMatter(front_matter) = block else {
                continue;
            };

            if let Some(value) = front_matter.entries.iter().find_map(|entry| {
                (entry.key == "title" && !entry.value.trim().is_empty())
                    .then(|| entry.value.trim().to_string())
            }) {
                return value;
            }
        }

        "Kastel Markup".into()
    }

    fn document_language(&self, document: &Document) -> String {
        for block in &document.blocks {
            let Block::FrontMatter(front_matter) = block else {
                continue;
            };

            for entry in &front_matter.entries {
                if (entry.key == "lang" || entry.key == "language")
                    && is_valid_language_tag(entry.value.trim())
                {
                    return entry.value.trim().to_string();
                }
            }
        }

        "en".into()
    }
}

impl Renderer for HtmlRenderer {
    fn render(&self, document: &Document) -> String {
        let mut output = String::new();
        self.render_blocks(&document.blocks, &mut output);
        output
    }
}

fn escape_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }

    output
}

fn escape_html_attribute(value: &str) -> String {
    escape_html(value)
}

fn escape_html_comment(value: &str) -> String {
    value.replace("--", "- -")
}

fn safe_url(value: &str) -> String {
    let trimmed = value.trim();
    let normalized = trimmed
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();

    if normalized.starts_with("javascript:")
        || normalized.starts_with("vbscript:")
        || normalized.starts_with("data:")
    {
        return "#".into();
    }

    escape_html_attribute(trimmed)
}

fn custom_class_name(value: &str) -> String {
    let mut result = String::new();

    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push('-');
        }
    }

    if result.is_empty() {
        "custom".into()
    } else {
        result
    }
}

fn is_valid_language_tag(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Block, Document, FrontMatter, FrontMatterEntry, Inline, ListItem, Table, TableAlignment,
    };

    #[test]
    fn renders_paragraph() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("Hello".into())])],
        };

        assert_eq!(HtmlRenderer::new().render(&document), "<p>Hello</p>\n");
    }

    #[test]
    fn renders_heading_with_anchor() {
        let document = Document {
            blocks: vec![Block::Heading {
                level: 2,
                content: vec![Inline::Text("Introduction".into())],
                anchor: Some("intro".into()),
            }],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<h2 id=\"intro\">Introduction</h2>\n"
        );
    }

    #[test]
    fn renders_inline_markup() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Text("Hello ".into()),
                Inline::Strong(vec![Inline::Text("world".into())]),
                Inline::Text("!".into()),
            ])],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<p>Hello <strong>world</strong>!</p>\n"
        );
    }

    #[test]
    fn escapes_html() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text(
                "<script>alert(\"x\")</script>".into(),
            )])],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<p>&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;</p>\n"
        );
    }

    #[test]
    fn renders_link_and_image() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![
                Inline::Link {
                    text: vec![Inline::Text("Kastel".into())],
                    url: "https://example.com".into(),
                },
                Inline::Text(" ".into()),
                Inline::Image {
                    alt: "logo".into(),
                    url: "kastel.png".into(),
                },
            ])],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<p><a href=\"https://example.com\">Kastel</a> <img src=\"kastel.png\" alt=\"logo\"></p>\n"
        );
    }

    #[test]
    fn blocks_dangerous_urls() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![Inline::Link {
                text: vec![Inline::Text("click".into())],
                url: "javascript:alert(1)".into(),
            }])],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<p><a href=\"#\">click</a></p>\n"
        );
    }

    #[test]
    fn renders_simple_lists() {
        let document = Document {
            blocks: vec![Block::UnorderedList {
                items: vec![
                    ListItem {
                        checked: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("First".into())])],
                    },
                    ListItem {
                        checked: None,
                        blocks: vec![Block::Paragraph(vec![Inline::Text("Second".into())])],
                    },
                ],
            }],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<ul>\n<li>First</li>\n<li>Second</li>\n</ul>\n"
        );
    }

    #[test]
    fn renders_nested_lists() {
        let document = Document {
            blocks: vec![Block::UnorderedList {
                items: vec![ListItem {
                    checked: None,
                    blocks: vec![
                        Block::Paragraph(vec![Inline::Text("Parent".into())]),
                        Block::UnorderedList {
                            items: vec![ListItem {
                                checked: None,
                                blocks: vec![Block::Paragraph(vec![Inline::Text(
                                    "Child".into(),
                                )])],
                            }],
                        },
                    ],
                }],
            }],
        };

        let html = HtmlRenderer::new().render(&document);
        assert!(html.contains("<li>Parent<ul>"));
        assert!(html.contains("<li>Child</li>"));
    }

    #[test]
    fn renders_checkbox() {
        let document = Document {
            blocks: vec![Block::UnorderedList {
                items: vec![ListItem {
                    checked: Some(true),
                    blocks: vec![Block::Paragraph(vec![Inline::Text("Done".into())])],
                }],
            }],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<ul>\n<li class=\"task-list-item\"><input type=\"checkbox\" disabled checked> Done</li>\n</ul>\n"
        );
    }

    #[test]
    fn renders_table() {
        let document = Document {
            blocks: vec![Block::Table(Table {
                headers: vec![
                    vec![Inline::Text("Name".into())],
                    vec![Inline::Text("Age".into())],
                ],
                alignments: vec![TableAlignment::Left, TableAlignment::Right],
                rows: vec![vec![
                    vec![Inline::Text("Bruno".into())],
                    vec![Inline::Text("26".into())],
                ]],
            })],
        };

        let html = HtmlRenderer::new().render(&document);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th style=\"text-align:left\">Name</th>"));
        assert!(html.contains("<th style=\"text-align:right\">Age</th>"));
        assert!(html.contains("<td style=\"text-align:left\">Bruno</td>"));
        assert!(html.contains("<td style=\"text-align:right\">26</td>"));
    }

    #[test]
    fn renders_irregular_table() {
        let document = Document {
            blocks: vec![Block::Table(Table {
                headers: vec![
                    vec![Inline::Text("A".into())],
                    vec![Inline::Text("B".into())],
                    vec![Inline::Text("C".into())],
                ],
                alignments: vec![
                    TableAlignment::None,
                    TableAlignment::None,
                    TableAlignment::None,
                ],
                rows: vec![vec![
                    vec![Inline::Text("1".into())],
                    vec![Inline::Text("2".into())],
                ]],
            })],
        };

        let html = HtmlRenderer::new().render(&document);
        assert!(html.contains("<td>1</td>"));
        assert!(html.contains("<td>2</td>"));
        assert!(html.contains("<td></td>"));
    }

    #[test]
    fn renders_custom_block() {
        let document = Document {
            blocks: vec![Block::Custom {
                kind: "warning".into(),
                blocks: vec![Block::Paragraph(vec![Inline::Text("Attention".into())])],
            }],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<div class=\"custom-block custom-warning\">\n<p>Attention</p>\n</div>\n"
        );
    }

    #[test]
    fn renders_comment_safely() {
        let document = Document {
            blocks: vec![Block::Comment("danger --> content".into())],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<!--danger - -> content-->\n"
        );
    }

    #[test]
    fn skips_front_matter_in_fragment() {
        let document = Document {
            blocks: vec![
                Block::FrontMatter(FrontMatter {
                    entries: vec![FrontMatterEntry {
                        key: "title".into(),
                        value: "Kastel".into(),
                    }],
                }),
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Hello".into())],
                    anchor: None,
                },
            ],
        };

        assert_eq!(
            HtmlRenderer::new().render(&document),
            "<h1>Hello</h1>\n"
        );
    }

    #[test]
    fn renders_complete_html_document() {
        let document = Document {
            blocks: vec![
                Block::FrontMatter(FrontMatter {
                    entries: vec![
                        FrontMatterEntry {
                            key: "title".into(),
                            value: "Kastel".into(),
                        },
                        FrontMatterEntry {
                            key: "lang".into(),
                            value: "fr".into(),
                        },
                    ],
                }),
                Block::Heading {
                    level: 1,
                    content: vec![Inline::Text("Bienvenue".into())],
                    anchor: None,
                },
            ],
        };

        assert_eq!(
            HtmlRenderer::new().render_document(&document),
            "<!doctype html>\n<html lang=\"fr\">\n<head>\n    <meta charset=\"utf-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n    <title>Kastel</title>\n</head>\n<body>\n    <h1>Bienvenue</h1>\n</body>\n</html>\n"
        );
    }

    #[test]
    fn defaults_title_and_language() {
        let document = Document {
            blocks: vec![Block::Paragraph(vec![Inline::Text("Hello".into())])],
        };

        let html = HtmlRenderer::new().render_document(&document);
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<title>Kastel Markup</title>"));
    }
}
