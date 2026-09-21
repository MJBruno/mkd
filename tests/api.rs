use kastel_markup::{parse, render_html, render_html_document, Block};

#[test]
fn parses_source_directly() {
    let document = parse("# Kastel").expect("parse failed");

    assert_eq!(document.blocks.len(), 1);
    assert!(matches!(document.blocks[0], Block::Heading { level: 1, .. }));
}

#[test]
fn renders_fragment_directly() {
    let html = render_html("# Kastel\n\nHello **world**.").expect("render failed");

    assert_eq!(
        html,
        "<h1>Kastel</h1>\n<p>Hello <strong>world</strong>.</p>\n"
    );
}

#[test]
fn renders_standalone_document_directly() {
    let html = render_html_document(
        "---\n\
         title: Kastel\n\
         lang: fr\n\
         ---\n\
         # Hello\n",
    )
    .expect("render failed");

    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("<html lang=\"fr\">"));
    assert!(html.contains("<title>Kastel</title>"));
    assert!(html.contains("<h1>Hello</h1>"));
}

#[test]
fn propagates_parser_errors() {
    assert!(render_html("*unclosed").is_err());
    assert!(render_html_document("<!-- unclosed").is_err());
}
