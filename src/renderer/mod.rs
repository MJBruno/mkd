use crate::ast::Document;

pub mod html;

pub use html::HtmlRenderer;

pub trait Renderer {
    fn render(&self, document: &Document) -> String;
}

pub fn render_html_fragment(document: &Document) -> String {
    HtmlRenderer::new().render(document)
}
