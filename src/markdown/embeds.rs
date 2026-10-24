//! Widgets embedded in the editor in place of a paragraph: tables, whose
//! cells hold their inline markdown (Notebook's ADR-0005), code blocks with a
//! language picker, a copy button and syntax highlighting (Notebook's ADR-0006), and
//! images with their description (Notebook's ADR-0012).

use std::path::Path;

use egui::{Id, RichText};
use egui_extras::syntax_highlighting::{self, CodeTheme};

use super::model::{Align, Image, Table};

/// Images are shown at their own size, scaled down to fit the column and,
/// for tall ones, this height.
const IMAGE_MAX_HEIGHT: f32 = 640.0;

#[derive(Default)]
pub struct EmbedResponse {
    pub changed: bool,
    /// The user asked to remove the whole table or code block.
    pub delete: bool,
}

/// Languages offered for code blocks; any other fence tag is kept and shown.
const LANGUAGES: &[(&str, &str)] = &[
    ("", "Plain text"),
    ("bash", "Bash"),
    ("c", "C"),
    ("cpp", "C++"),
    ("css", "CSS"),
    ("go", "Go"),
    ("html", "HTML"),
    ("java", "Java"),
    ("js", "JavaScript"),
    ("json", "JSON"),
    ("markdown", "Markdown"),
    ("python", "Python"),
    ("rust", "Rust"),
    ("sql", "SQL"),
    ("toml", "TOML"),
    ("ts", "TypeScript"),
    ("yaml", "YAML"),
];

const CELL_WIDTH: f32 = 140.0;

/// A table; with `editable` off, its cells can only be selected and copied.
pub fn table(ui: &mut egui::Ui, table: &mut Table, id: Id, editable: bool) -> EmbedResponse {
    let mut out = EmbedResponse::default();
    let mut command = None;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::Grid::new(id).spacing([4.0, 4.0]).show(ui, |ui| {
            for r in 0..table.rows.len() {
                for c in 0..table.columns() {
                    let header = r == 0;
                    let align = match table.align[c] {
                        Align::Center => egui::Align::Center,
                        Align::Right => egui::Align::Max,
                        _ => egui::Align::Min,
                    };
                    // A &str shows the text, selectable, and edits nothing.
                    let mut shown = table.rows[r][c].as_str();
                    let text: &mut dyn egui::TextBuffer = if editable {
                        &mut table.rows[r][c]
                    } else {
                        &mut shown
                    };
                    let mut edit = egui::TextEdit::singleline(text)
                        .id(id.with((r, c)))
                        .desired_width(CELL_WIDTH)
                        .horizontal_align(align);
                    if header {
                        edit = edit
                            .text_color(ui.visuals().strong_text_color())
                            .hint_text("Heading");
                    }
                    let resp = ui.add_sized([CELL_WIDTH, ui.spacing().interact_size.y], edit);
                    out.changed |= resp.changed();
                    if editable {
                        resp.context_menu(|ui| {
                            if let Some(cmd) = table_menu(ui, r, c, table) {
                                command = Some(cmd);
                                ui.close();
                            }
                        });
                    }
                }
                ui.end_row();
            }
        });
        if !editable {
            return;
        }
        ui.horizontal(|ui| {
            if ui.small_button("+ Row").clicked() {
                command = Some(TableCommand::InsertRow(table.rows.len()));
            }
            if ui.small_button("+ Column").clicked() {
                command = Some(TableCommand::InsertColumn(table.columns()));
            }
            ui.label(RichText::new("Right-click a cell for more").weak().small());
        });
    });
    if let Some(cmd) = command {
        if cmd == TableCommand::DeleteTable {
            out.delete = true;
        } else {
            apply(table, cmd);
            out.changed = true;
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TableCommand {
    InsertRow(usize),
    DeleteRow(usize),
    InsertColumn(usize),
    DeleteColumn(usize),
    Align(usize, Align),
    DeleteTable,
}

fn table_menu(ui: &mut egui::Ui, r: usize, c: usize, table: &Table) -> Option<TableCommand> {
    let mut cmd = None;
    if r > 0 && ui.button("Insert Row Above").clicked() {
        cmd = Some(TableCommand::InsertRow(r));
    }
    if ui.button("Insert Row Below").clicked() {
        cmd = Some(TableCommand::InsertRow(r + 1));
    }
    if r > 0 && table.rows.len() > 2 && ui.button("Delete Row").clicked() {
        cmd = Some(TableCommand::DeleteRow(r));
    }
    ui.separator();
    if ui.button("Insert Column Left").clicked() {
        cmd = Some(TableCommand::InsertColumn(c));
    }
    if ui.button("Insert Column Right").clicked() {
        cmd = Some(TableCommand::InsertColumn(c + 1));
    }
    if table.columns() > 1 && ui.button("Delete Column").clicked() {
        cmd = Some(TableCommand::DeleteColumn(c));
    }
    ui.separator();
    ui.menu_button("Align Column", |ui| {
        for (align, label) in [
            (Align::None, "Default"),
            (Align::Left, "Left"),
            (Align::Center, "Centre"),
            (Align::Right, "Right"),
        ] {
            if ui.radio(table.align[c] == align, label).clicked() {
                cmd = Some(TableCommand::Align(c, align));
            }
        }
    });
    ui.separator();
    if ui.button("Delete Table").clicked() {
        cmd = Some(TableCommand::DeleteTable);
    }
    cmd
}

fn apply(table: &mut Table, cmd: TableCommand) {
    let columns = table.columns();
    match cmd {
        TableCommand::InsertRow(at) => table.rows.insert(at.max(1), vec![String::new(); columns]),
        TableCommand::DeleteRow(at) => {
            table.rows.remove(at);
        }
        TableCommand::InsertColumn(at) => {
            table.align.insert(at, Align::None);
            for row in &mut table.rows {
                row.insert(at, String::new());
            }
        }
        TableCommand::DeleteColumn(at) => {
            table.align.remove(at);
            for row in &mut table.rows {
                row.remove(at);
            }
        }
        TableCommand::Align(c, align) => table.align[c] = align,
        TableCommand::DeleteTable => {}
    }
}

/// The name syntect knows a fence tag's language by: it looks syntaxes up
/// by file extension.
fn syntax_name(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "" => "txt",
        "rust" => "rs",
        "python" => "py",
        "javascript" => "js",
        "typescript" => "ts",
        "bash" | "shell" | "zsh" => "sh",
        "markdown" => "md",
        "yml" => "yaml",
        "c++" => "cpp",
        "golang" => "go",
        other => return other.to_string(),
    }
    .to_string()
}

/// A code block; with `editable` off, its language is a label and its code
/// can only be selected and copied.
pub fn code(
    ui: &mut egui::Ui,
    lang: &mut String,
    code: &mut String,
    id: Id,
    editable: bool,
) -> EmbedResponse {
    let mut out = EmbedResponse::default();
    let fill = ui.visuals().code_bg_color;
    egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
        ui.horizontal(|ui| {
            let shown = LANGUAGES
                .iter()
                .find(|(tag, _)| tag == lang)
                .map_or(lang.as_str(), |(_, name)| name);
            if !editable {
                ui.label(RichText::new(shown).weak().small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("Copy")
                        .on_hover_text("Copy the code")
                        .clicked()
                    {
                        ui.ctx().copy_text(code.clone());
                    }
                });
                return;
            }
            egui::ComboBox::from_id_salt(id.with("lang"))
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    for (tag, name) in LANGUAGES {
                        if ui.selectable_label(lang == tag, *name).clicked() && lang != tag {
                            *lang = tag.to_string();
                            out.changed = true;
                        }
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("Delete")
                    .on_hover_text("Delete this code block")
                    .clicked()
                {
                    out.delete = true;
                }
                if ui
                    .small_button("Copy")
                    .on_hover_text("Copy the code")
                    .clicked()
                {
                    ui.ctx().copy_text(code.clone());
                }
            });
        });
        let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
        let language = syntax_name(lang);
        let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
            let mut job = syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                text.as_str(),
                &language,
            );
            job.wrap.max_width = wrap;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let mut shown = code.as_str();
        let text: &mut dyn egui::TextBuffer = if editable { code } else { &mut shown };
        let resp = ui.add(
            egui::TextEdit::multiline(text)
                .id(id.with("text"))
                .code_editor()
                .frame(egui::Frame::NONE)
                .desired_rows(1)
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter),
        );
        out.changed |= resp.changed();
    });
    out
}

/// An image, with its description (the markdown alt text) editable
/// under it. Relative sources are found from `dir`, the note's folder.
pub fn image(
    ui: &mut egui::Ui,
    image: &mut Image,
    dir: Option<&Path>,
    id: Id,
    editable: bool,
) -> EmbedResponse {
    let mut out = EmbedResponse::default();
    let uri = image_uri(&image.src, dir);
    let width = ui.available_width();
    let missing = uri
        .strip_prefix("file://")
        .is_some_and(|path| !Path::new(path).exists());
    let shown = if missing {
        let text = RichText::new(format!("Image not found: {}", image.src)).weak();
        egui::Frame::group(ui.style())
            .show(ui, |ui| {
                ui.add(egui::Label::new(text).sense(egui::Sense::click()))
            })
            .inner
    } else {
        ui.add(
            egui::Image::new(uri.clone())
                .fit_to_original_size(1.0)
                .max_width(width)
                .max_height(IMAGE_MAX_HEIGHT)
                .corner_radius(4)
                .sense(egui::Sense::click()),
        )
    };
    let hint = if image.title.is_empty() {
        image.src.clone()
    } else {
        format!("{}\n{}", image.title, image.src)
    };
    shown.on_hover_text(hint).context_menu(|ui| {
        if ui.button("Open Image").clicked() {
            // A local file in the image viewer, not the web browser.
            match uri.strip_prefix("file://") {
                Some(path) => {
                    let _ = crate::open::path(Path::new(path));
                }
                None => ui.ctx().open_url(egui::OpenUrl::new_tab(&uri)),
            }
            ui.close();
        }
        if ui.button("Copy Image Path").clicked() {
            ui.ctx().copy_text(image.src.clone());
            ui.close();
        }
        if editable {
            ui.separator();
            if ui.button("Remove Image").clicked() {
                out.delete = true;
                ui.close();
            }
        }
    });
    if !editable {
        if !image.alt.is_empty() {
            ui.label(RichText::new(&image.alt).small().weak());
        }
        return out;
    }
    let alt = ui.add(
        egui::TextEdit::singleline(&mut image.alt)
            .id(id.with("alt"))
            .hint_text("Add a description")
            .font(egui::TextStyle::Small)
            .text_color(ui.visuals().weak_text_color())
            .frame(egui::Frame::NONE)
            .desired_width(width),
    );
    out.changed = alt.changed();
    out
}

/// Gives the parts of `job` in `marks` (byte ranges of its text) the
/// background `colour`, and the `current` one `current_colour`, as find
/// marks what it found. Sections are split where marks start and end.
pub fn mark(
    job: &mut egui::text::LayoutJob,
    marks: &[std::ops::Range<usize>],
    current: Option<usize>,
    colour: egui::Color32,
    current_colour: egui::Color32,
) {
    if marks.is_empty() {
        return;
    }
    let mut cuts: Vec<usize> = marks.iter().flat_map(|m| [m.start, m.end]).collect();
    cuts.sort_unstable();
    cuts.dedup();
    let mut sections = Vec::with_capacity(job.sections.len() + cuts.len());
    for section in std::mem::take(&mut job.sections) {
        let (first, last) = (section.byte_range.start.0, section.byte_range.end.0);
        let mut start = first;
        let inner = cuts.iter().copied().filter(|&c| c > first && c < last);
        for end in inner.chain(std::iter::once(last)) {
            let mut piece = section.clone();
            piece.byte_range = egui::text::ByteIndex(start)..egui::text::ByteIndex(end);
            if start > first {
                piece.leading_space = 0.0;
            }
            if let Some(i) = marks.iter().position(|m| m.start <= start && end <= m.end) {
                piece.format.background = if Some(i) == current {
                    current_colour
                } else {
                    colour
                };
            }
            sections.push(piece);
            start = end;
        }
    }
    job.sections = sections;
}

/// A whole source file, highlighted as a code block is, with line numbers,
/// for reading: the text can be selected and copied, not changed. `lang`
/// is a fence tag or a file extension (`rust`, `rs`, `toml`). `marks` are
/// byte ranges to mark, as found text, the `current` one more strongly;
/// the current one's place on screen is returned, to scroll to.
pub fn code_file(
    ui: &mut egui::Ui,
    text: &str,
    lang: &str,
    id: Id,
    marks: &[std::ops::Range<usize>],
    current: Option<usize>,
) -> Option<egui::Rect> {
    let theme = CodeTheme::from_memory(ui.ctx(), ui.style());
    let language = syntax_name(lang);
    let rows = text.lines().count().max(1);
    let digits = rows.to_string().len();
    let numbers: String = (1..=rows).map(|n| format!("{n:>digits$}\n")).collect();
    let selection = ui.visuals().selection.bg_fill;
    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
        // Code isn't wrapped: lines keep their shape, and scroll sideways.
        let mut job =
            syntax_highlighting::highlight(ui.ctx(), ui.style(), &theme, text.as_str(), &language);
        mark(
            &mut job,
            marks,
            current,
            selection.gamma_multiply(0.35),
            selection,
        );
        ui.fonts_mut(|f| f.layout_job(job))
    };
    let mut found = None;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        ui.add(
            egui::Label::new(
                RichText::new(numbers)
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            )
            .selectable(false),
        );
        let mut shown = text;
        let output = egui::TextEdit::multiline(&mut shown)
            .id(id)
            .code_editor()
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::ZERO)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter)
            .show(ui);
        found = current.and_then(|i| marks.get(i)).map(|m| {
            let chars = text[..m.start].chars().count();
            output
                .galley
                .pos_from_cursor(egui::text::CCursor::new(chars))
                .translate(output.galley_pos.to_vec2())
        });
    });
    found
}

/// Decodes `%20`-style escapes, which markdown image paths often use for
/// spaces. Anything that isn't a valid escape is left as it is.
fn percent_decode(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
}

/// The URI egui loads an image's `src` from: URLs as they are, paths as
/// `file://`, relative ones from `dir` (the document's folder).
pub fn image_uri(src: &str, dir: Option<&Path>) -> String {
    if src.contains("://") {
        return src.to_string();
    }
    let path = std::path::PathBuf::from(percent_decode(src));
    let path = match dir {
        Some(dir) if path.is_relative() => dir.join(path),
        _ => path,
    };
    format!("file://{}", path.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_commands_keep_rows_square() {
        let mut t = Table::new(2, 1);
        apply(&mut t, TableCommand::InsertColumn(1));
        apply(&mut t, TableCommand::InsertRow(1));
        assert_eq!(t.columns(), 3);
        assert!(t.rows.iter().all(|r| r.len() == 3));
        assert_eq!(t.rows.len(), 3);
        apply(&mut t, TableCommand::DeleteColumn(0));
        apply(&mut t, TableCommand::DeleteRow(2));
        assert_eq!((t.columns(), t.rows.len()), (2, 2));
        // The header row stays first.
        apply(&mut t, TableCommand::InsertRow(0));
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn marks_split_sections_where_they_start_and_end() {
        let mut job = egui::text::LayoutJob::default();
        job.append("hello world", 0.0, egui::TextFormat::default());
        let (weak, strong) = (egui::Color32::GRAY, egui::Color32::RED);
        mark(&mut job, &[0..2, 6..11], Some(1), weak, strong);
        let pieces: Vec<(std::ops::Range<usize>, egui::Color32)> = job
            .sections
            .iter()
            .map(|s| {
                (
                    s.byte_range.start.0..s.byte_range.end.0,
                    s.format.background,
                )
            })
            .collect();
        assert_eq!(
            pieces,
            [
                (0..2, weak),
                (2..6, egui::Color32::TRANSPARENT),
                (6..11, strong)
            ]
        );
    }

    #[test]
    fn image_sources_become_uris() {
        let dir = Path::new("/notes");
        assert_eq!(image_uri("a%20b.png", Some(dir)), "file:///notes/a b.png");
        assert_eq!(image_uri("/x.png", Some(dir)), "file:///x.png");
        assert_eq!(
            image_uri("https://e.com/i.png", Some(dir)),
            "https://e.com/i.png"
        );
        assert_eq!(
            image_uri("50%.png", Some(dir)),
            "file:///notes/50%.png",
            "a bad escape is kept"
        );
    }
}
