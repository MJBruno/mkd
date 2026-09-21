use kastel_markup::{render_html, Lexer, Parser, Renderer};

fn render(source: &str) -> String {
    render_html(source).expect("render failed")
}

#[test]
fn renders_complete_fragment() {
    let html = render(
        "# Kastel {#kastel}\n\
         \n\
         This is **Kastel** with *markup*.\n\
         \n\
         - First\n\
         - Second\n\
         \n\
         > A quote\n\
         \n\
         ---\n\
         \n\
         | Name | Age |\n\
         | --- | ---: |\n\
         | Bruno | 26 |\n\
         \n\
         ```rust\n\
         fn main() {}\n\
         ```\n\
         \n\
         ::: warning\n\
         Be careful.\n\
         :::\n",
    );

    assert!(html.contains("<h1 id=\"kastel\">Kastel</h1>"));
    assert!(html.contains("<strong>Kastel</strong>"));
    assert!(html.contains("<em>markup</em>"));
    assert!(html.contains("<li>First</li>"));
    assert!(html.contains("<li>Second</li>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<hr>"));
    assert!(html.contains("<table>"));
    assert!(html.contains("class=\"language-rust\""));
    assert!(html.contains("class=\"custom-block custom-warning\""));
}

#[test]
fn lexer_parser_renderer_pipeline_is_available() {
    let tokens = Lexer::new("# Kastel").tokenize().unwrap();
    let document = Parser::new(tokens).parse().unwrap();
    let html = kastel_markup::HtmlRenderer::new().render(&document);

    assert_eq!(html, "<h1>Kastel</h1>\n");
}
