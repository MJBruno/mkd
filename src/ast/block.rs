#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(Vec<Inline>),

    Heading {
        level: u8,
        content: Vec<Inline>,
        anchor: Option<String>,
    },

    Comment(String),

    FrontMatter(FrontMatter),

    UnorderedList {
        items: Vec<ListItem>,
    },

    OrderedList {
        items: Vec<ListItem>,
    },

    Quote {
        blocks: Vec<Block>,
    },

    HorizontalRule,

    CodeBlock {
        language: Option<String>,
        code: String,
    },

    Table(Table),

    Custom {
        kind: String,
        blocks: Vec<Block>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontMatter {
    pub entries: Vec<FrontMatterEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontMatterEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub headers: Vec<Vec<Inline>>,
    pub alignments: Vec<TableAlignment>,
    pub rows: Vec<Vec<Vec<Inline>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link {
        text: Vec<Inline>,
        url: String,
    },
    Image {
        alt: String,
        url: String,
    },
}
