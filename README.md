# Kastel Markup

Kastel Markup is a small, deterministic Markdown-like markup language and Rust library.

## Architecture

```text
Source
  ↓
Lexer
  ↓
Tokens
  ↓
Parser
  ↓
AST
  ↓
Renderers
```

The parser and AST are independent from the renderers.

## Current syntax

### Blocks

- Front Matter
- Headings (`#` … `######`)
- Paragraphs
- Unordered lists
- Ordered lists
- Nested lists
- Task checkboxes
- Quotes and nested quotes
- Horizontal rules (`---`)
- Fenced code blocks
- Indented code blocks
- Tables with alignment
- Anchors (`{#id}`)
- Comments (`<!-- ... -->`)
- Custom blocks (`::: note ... :::`), including nesting

### Inline

- Emphasis: `*text*`, `_text_`
- Strong: `**text**`, `__text__`
- Strong + emphasis: `***text***`, `___text___`
- Strikethrough: `~~text~~`
- Inline code: `` `code` ``
- Links: `[text](url)`
- Images: `![alt](url)`
- Backslash escapes

## Rust API

### Parse

```rust
use kastel_markup::parse;

let document = parse("# Hello\n\nThis is *Kastel*.")?;
```

### HTML fragment

```rust
use kastel_markup::render_html;

let html = render_html("# Hello\n\nThis is **Kastel**.")?;
```

### Standalone HTML document

```rust
use kastel_markup::render_html_document;

let html = render_html_document(
    "---\n\
     title: Kastel\n\
     lang: fr\n\
     ---\n\
     # Hello"
)?;
```

Front Matter keys `title`, `lang` and `language` are used by the standalone HTML renderer. Front Matter is not emitted into the HTML body.

### Low-level pipeline

```rust
use kastel_markup::{HtmlRenderer, Lexer, Parser, Renderer};

let tokens = Lexer::new("# Hello").tokenize()?;
let document = Parser::new(tokens).parse()?;
let html = HtmlRenderer::new().render(&document);
```

## CLI

The package also provides the `km` binary.

```bash
km document.km
km document.km -o document.html
km document.km --standalone -o document.html
km - < document.km
```

By default the CLI writes an HTML fragment to stdout. `--standalone` generates a complete HTML5 document.

Only the `html` format is currently implemented.

## Project layout

```text
kastel-markup/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── error.rs
│   ├── ast/
│   │   ├── mod.rs
│   │   └── block.rs
│   ├── lexer/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── lexer.rs
│   │   └── token.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   └── parser.rs
│   └── renderer/
│       ├── mod.rs
│       └── html.rs
└── tests/
    ├── api.rs
    └── html_renderer.rs
```

## Commands

```bash
cargo fmt --all
cargo check
cargo test
cargo run -- --help
cargo run -- --version
```
