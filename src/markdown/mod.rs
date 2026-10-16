//! Markdown as Notebook shows it, for every Patina app that shows
//! documents (the `markdown` feature, ADR-0015): no markdown syntax on
//! screen, headings, lists, tasks, quotes, tables, highlighted code and
//! images, in the skin's colours and Ubuntu's variable fonts.
//!
//! - [`model`]: markdown to and from lines of styled spans.
//! - [`doc`]: the document's paragraphs (text, table, code, image).
//! - [`editor`]: the widget, which edits in place (Notebook) or, with
//!   [`Editor::set_read_only`], only shows, selects and follows links
//!   (Reader, help).
//! - [`embeds`]: the table, code block and image widgets, and
//!   [`embeds::code_file`], a whole source file, highlighted.
//! - [`fonts`]: the fonts; [`install`] sets them up with image loading.

pub mod doc;
pub mod editor;
pub mod embeds;
pub mod fonts;
pub mod model;

pub use editor::Editor;

/// Sets up a context for showing markdown: the fonts, and egui's image
/// loaders for the images documents embed. Call once, at start-up.
pub fn install(ctx: &egui::Context) {
    fonts::install(ctx);
    egui_extras::install_image_loaders(ctx);
}
