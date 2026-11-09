//! Widgets embedded in the editor in place of a paragraph: tables, whose
//! cells hold their inline markdown (Notebook's ADR-0005), code blocks with a
//! language picker, a copy button and syntax highlighting (Notebook's ADR-0006), and
//! images with their description (Notebook's ADR-0012).

use std::path::Path;

use egui::{Id, RichText};

use super::model::{Align, CellPart, Image, Table, cell_parts};

/// Images are shown at their own size, scaled down to fit the column and,
/// for tall ones, this height.
const IMAGE_MAX_HEIGHT: f32 = 1600.0;

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

/// Space inside a cell, around its content.
const CELL_PADDING: egui::Vec2 = egui::vec2(8.0, 5.0);
/// A column is never narrower than this, so an empty cell can still be clicked.
const MIN_COLUMN: f32 = 48.0;
/// What an image that isn't loaded yet takes.
const PENDING_IMAGE: egui::Vec2 = egui::vec2(64.0, 64.0);

/// A table, laid out as HTML lays out an auto-width one: it is as wide as its
/// content wants, up to the width available (the text's); beyond that, text wraps
/// and images shrink, each column getting room for what it holds. Cells are joined,
/// with borders between them. A cell holds its inline markdown, and the images in it
/// (`![alt](src)`, `<img>`) are shown; with `editable` off, its text can only be
/// selected and copied.
pub fn table(
    ui: &mut egui::Ui,
    table: &mut Table,
    dir: Option<&Path>,
    id: Id,
    editable: bool,
) -> EmbedResponse {
    let mut out = EmbedResponse::default();
    let mut command = None;
    let columns = table.columns();
    if table.rows.is_empty() || columns == 0 {
        return out;
    }
    let available = ui.available_width();
    let cells: Vec<Vec<Vec<CellPart>>> = table
        .rows
        .iter()
        .map(|row| row.iter().map(|cell| cell_parts(cell)).collect())
        .collect();
    let widths = column_widths(ui, &table.rows, &cells, dir, available);
    let width: f32 = widths.iter().sum();

    let visuals = ui.visuals().clone();
    let stroke = visuals.widgets.noninteractive.bg_stroke;
    // Fills and borders go under the cells' content: their place is kept now.
    let backdrop = ui.painter().add(egui::Shape::Noop);
    let mut shapes = Vec::new();
    let origin = ui.available_rect_before_wrap().min;
    let mut y = origin.y;
    for (r, row_cells) in cells.iter().enumerate() {
        let mut row_height: f32 = 0.0;
        let mut x = origin.x;
        let mut rects = Vec::new();
        for c in 0..columns {
            let align = match table.align[c] {
                Align::Center => egui::Align::Center,
                Align::Right => egui::Align::Max,
                _ => egui::Align::Min,
            };
            let inner = egui::Rect::from_min_size(
                egui::pos2(x, y) + CELL_PADDING,
                egui::vec2(widths[c] - 2.0 * CELL_PADDING.x, 1.0e6),
            );
            let mut cell_ui = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(id.with(("cell", r, c)))
                    .max_rect(inner)
                    .layout(egui::Layout::top_down(align)),
            );
            let cell = CellView {
                id: id.with((r, c)),
                header: r == 0,
                align,
                editable,
                dir,
            };
            let changed_by = show_cell(&mut cell_ui, &row_cells[c], &mut table.rows[r][c], &cell);
            out.changed |= changed_by.changed;
            if let Some(cmd) = changed_by.menu_command(r, c, table) {
                command = Some(cmd);
            }
            row_height = row_height.max(cell_ui.min_rect().height() + 2.0 * CELL_PADDING.y);
            rects.push(x);
            x += widths[c];
        }
        let band = egui::Rect::from_min_size(egui::pos2(origin.x, y), egui::vec2(width, row_height));
        if r == 0 {
            shapes.push(egui::Shape::rect_filled(band, 0.0, visuals.faint_bg_color));
        }
        // The line under each row, and the ones between its cells.
        shapes.push(egui::Shape::hline(band.x_range(), band.bottom(), stroke));
        for &left in rects.iter().skip(1) {
            shapes.push(egui::Shape::vline(left, band.y_range(), stroke));
        }
        y += row_height;
    }
    let table_rect = egui::Rect::from_min_max(origin, egui::pos2(origin.x + width, y));
    shapes.push(egui::Shape::hline(table_rect.x_range(), table_rect.top(), stroke));
    shapes.push(egui::Shape::vline(table_rect.left(), table_rect.y_range(), stroke));
    shapes.push(egui::Shape::vline(table_rect.right(), table_rect.y_range(), stroke));
    ui.painter().set(backdrop, egui::Shape::Vec(shapes));
    ui.allocate_rect(table_rect, egui::Sense::hover());

    if editable {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if ui.small_button("+ Row").clicked() {
                command = Some(TableCommand::InsertRow(table.rows.len()));
            }
            if ui.small_button("+ Column").clicked() {
                command = Some(TableCommand::InsertColumn(table.columns()));
            }
            ui.label(RichText::new("Right-click a cell for more").weak().small());
        });
    }
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

/// What a cell is shown with.
struct CellView<'a> {
    id: Id,
    header: bool,
    align: egui::Align,
    editable: bool,
    dir: Option<&'a Path>,
}

/// What came of showing a cell.
#[derive(Default)]
struct CellResult {
    changed: bool,
    /// The cell's response, for its context menu.
    response: Option<egui::Response>,
    command: Option<TableCommand>,
}

impl CellResult {
    fn menu_command(mut self, r: usize, c: usize, table: &Table) -> Option<TableCommand> {
        if let Some(response) = self.response.take() {
            response.context_menu(|ui| {
                if let Some(cmd) = table_menu(ui, r, c, table) {
                    self.command = Some(cmd);
                    ui.close();
                }
            });
        }
        self.command
    }
}

/// The text of a cell as it's measured and shown, `<br>` as a break.
fn cell_text_font(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Body.resolve(ui.style())
}

/// How wide a cell's content wants to be, `(least, most)`: its longest word, and its text
/// on one line. An image wants its own width either way, so it's the text that wraps.
fn content_width(
    ui: &egui::Ui,
    parts: &[CellPart],
    source: &str,
    dir: Option<&Path>,
) -> (f32, f32) {
    let font = cell_text_font(ui);
    let color = ui.visuals().text_color();
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_string(), font.clone(), color)
            .size()
            .x
    };
    let text_width = |text: &str| -> (f32, f32) {
        let most = text.lines().map(&measure).fold(0.0, f32::max);
        let least = text
            .split_whitespace()
            .map(&measure)
            .fold(0.0, f32::max);
        (least, most)
    };
    if !parts.iter().any(|p| matches!(p, CellPart::Image(_))) {
        // What shows is the source itself.
        return text_width(source);
    }
    parts.iter().fold((0.0, 0.0), |(least, most), part| {
        let (l, m) = match part {
            CellPart::Text(text) => text_width(text),
            CellPart::Image(image) => {
                let natural = natural_size(ui.ctx(), &image_uri(&image.src, dir))
                    .unwrap_or(PENDING_IMAGE);
                (natural.x, natural.x)
            }
        };
        (f32::max(least, l), f32::max(most, m))
    })
}

/// Each column's width: as wide as its widest cell wants if the table fits in
/// `available` that way; else every column gets its least (images their own width,
/// text its longest word), and what's left is shared by how much more each wants.
fn column_widths(
    ui: &egui::Ui,
    sources: &[Vec<String>],
    cells: &[Vec<Vec<CellPart>>],
    dir: Option<&Path>,
    available: f32,
) -> Vec<f32> {
    let columns = cells[0].len();
    let pad = 2.0 * CELL_PADDING.x;
    let mut least = vec![MIN_COLUMN; columns];
    let mut most = vec![MIN_COLUMN; columns];
    for (row, sources) in cells.iter().zip(sources) {
        for (c, parts) in row.iter().enumerate() {
            let (l, m) = content_width(ui, parts, &sources[c], dir);
            least[c] = least[c].max(l + pad);
            most[c] = most[c].max(m + pad);
        }
    }
    distribute(&least, &most, available)
}

/// The widths for columns that want `least..=most`, in `available`.
fn distribute(least: &[f32], most: &[f32], available: f32) -> Vec<f32> {
    let (min_sum, max_sum): (f32, f32) = (least.iter().sum(), most.iter().sum());
    if max_sum <= available {
        return most.to_vec();
    }
    if min_sum >= available {
        // Not even the least fits: all shrink together, down to a floor.
        let scale = available / min_sum;
        return least.iter().map(|l| (l * scale).max(24.0)).collect();
    }
    let room = available - min_sum;
    let want = max_sum - min_sum;
    least
        .iter()
        .zip(most)
        .map(|(l, m)| l + room * (m - l) / want)
        .collect()
}

/// An image's own size, once egui has loaded it.
fn natural_size(ctx: &egui::Context, uri: &str) -> Option<egui::Vec2> {
    refresh_if_changed(ctx, uri);
    match ctx.try_load_texture(
        uri,
        egui::TextureOptions::default(),
        egui::load::SizeHint::Scale(1.0.into()),
    ) {
        Ok(poll) => poll.size(),
        Err(_) => None,
    }
}

/// One cell: its text, editable as the markdown it is, or, when it holds
/// images and isn't being edited, those and its text as they look.
fn show_cell(
    ui: &mut egui::Ui,
    parts: &[CellPart],
    source: &mut String,
    cell: &CellView,
) -> CellResult {
    let mut result = CellResult::default();
    let width = ui.available_width();
    let has_images = parts.iter().any(|p| matches!(p, CellPart::Image(_)));
    let editing_key = cell.id.with("editing");
    let mut editing: bool = ui.data(|d| d.get_temp(editing_key)).unwrap_or(false);
    if has_images && !editing {
        show_parts(ui, parts, cell, width);
        let area = ui.min_rect().expand2(CELL_PADDING);
        let click = ui.interact(area, cell.id.with("area"), egui::Sense::click());
        if cell.editable && click.clicked() {
            ui.data_mut(|d| {
                d.insert_temp(editing_key, true);
                d.insert_temp(cell.id.with("focus"), true);
            });
            ui.ctx().request_repaint();
        }
        result.response = Some(click);
        return result;
    }
    // A &str shows the text, selectable, and edits nothing.
    let mut shown = source.as_str();
    let text: &mut dyn egui::TextBuffer = if cell.editable {
        source
    } else {
        &mut shown
    };
    let mut edit = egui::TextEdit::multiline(text)
        .id(cell.id)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::ZERO)
        .desired_rows(1)
        .desired_width(width)
        .return_key(None)
        .horizontal_align(cell.align);
    if cell.header {
        edit = edit
            .text_color(ui.visuals().strong_text_color())
            .hint_text("Heading");
    }
    let response = ui.add(edit);
    result.changed = response.changed();
    if has_images && cell.editable {
        let focus_wanted: bool = ui.data(|d| d.get_temp(cell.id.with("focus"))).unwrap_or(false);
        if focus_wanted {
            response.request_focus();
            ui.data_mut(|d| d.insert_temp(cell.id.with("focus"), false));
        } else if !response.has_focus() {
            editing = false;
            ui.data_mut(|d| d.insert_temp(editing_key, editing));
        }
    }
    if cell.editable {
        result.response = Some(response);
    }
    result
}

/// A cell's text and images, one under another.
fn show_parts(ui: &mut egui::Ui, parts: &[CellPart], cell: &CellView, width: f32) {
    for part in parts {
        match part {
            CellPart::Text(text) => {
                let mut rich = RichText::new(text);
                if cell.header {
                    rich = rich.color(ui.visuals().strong_text_color());
                }
                ui.add(egui::Label::new(rich).wrap());
            }
            CellPart::Image(image) => {
                let uri = image_uri(&image.src, cell.dir);
                let missing = uri
                    .strip_prefix("file://")
                    .is_some_and(|path| !Path::new(path).exists());
                if missing {
                    let note = if image.alt.is_empty() { &image.src } else { &image.alt };
                    ui.add(egui::Label::new(RichText::new(format!("🖼 {note}")).weak()).wrap());
                    continue;
                }
                let natural = natural_size(ui.ctx(), &uri).unwrap_or(PENDING_IMAGE);
                let scale = (width / natural.x)
                    .min(IMAGE_MAX_HEIGHT / natural.y)
                    .min(1.0);
                let response = ui.add(
                    egui::Image::new(uri.clone())
                        .fit_to_exact_size(natural * scale)
                        .corner_radius(2),
                );
                let hint = if image.title.is_empty() {
                    image.alt.clone()
                } else {
                    image.title.clone()
                };
                if !hint.is_empty() {
                    response.on_hover_text(hint);
                }
            }
        }
    }
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
    // The text-field background, as editors use (ADR-0026).
    let fill = ui.visuals().extreme_bg_color;
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
        let language = crate::highlight::language_for(lang);
        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
            let mut job = crate::highlight::layout_job(
                ui,
                id.with("text"),
                text.as_str(),
                language.as_deref(),
                &font,
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
    refresh_if_changed(ui.ctx(), &uri);
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
    let shown_width = shown.rect.width();
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
            // Wrapped to the image's width, however the layout is aligned.
            let wrap = shown_width.max(240.0).min(width);
            ui.allocate_ui(egui::vec2(wrap, 0.0), |ui| {
                ui.set_width(wrap);
                ui.add(egui::Label::new(RichText::new(&image.alt).small().weak()).wrap());
            });
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
            .desired_width(shown_width.max(240.0).min(width)),
    );
    if alt.changed() {
        // Edited: written as markdown from now on.
        image.html = None;
    }
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
    let language = crate::highlight::language_for(lang);
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let rows = text.lines().count().max(1);
    let digits = rows.to_string().len();
    let numbers: String = (1..=rows).map(|n| format!("{n:>digits$}\n")).collect();
    let selection = ui.visuals().selection.bg_fill;
    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
        // Code isn't wrapped: lines keep their shape, and scroll sideways.
        let mut job =
            crate::highlight::layout_job(ui, id, text.as_str(), language.as_deref(), &font);
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

/// How often a shown local image's file is looked at for a change, in seconds.
const IMAGE_POLL: f64 = 0.5;

/// Makes egui load a local image again when its file has changed since it was
/// shown, which its loaders would never do (they keep it by URI). Looks at most
/// every `IMAGE_POLL` seconds, and asks for a frame then so a change is seen
/// without any input.
fn refresh_if_changed(ctx: &egui::Context, uri: &str) {
    let Some(path) = uri.strip_prefix("file://") else {
        return;
    };
    let now = ctx.input(|i| i.time);
    let key = Id::new(("image-seen", uri));
    let (checked, seen): (f64, Option<std::time::SystemTime>) = ctx
        .data(|d| d.get_temp(key))
        .unwrap_or((f64::NEG_INFINITY, None));
    if now - checked < IMAGE_POLL {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            IMAGE_POLL - (now - checked),
        ));
        return;
    }
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    if seen.is_some() && modified != seen {
        ctx.forget_image(uri);
    }
    ctx.data_mut(|d| d.insert_temp(key, (now, modified)));
    ctx.request_repaint_after(std::time::Duration::from_secs_f64(IMAGE_POLL));
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
    fn a_table_that_fits_takes_only_what_it_wants() {
        // Wants 100 and 60 in 500: nothing is stretched.
        assert_eq!(distribute(&[40.0, 30.0], &[100.0, 60.0], 500.0), [100.0, 60.0]);
    }

    #[test]
    fn a_table_that_doesn_t_fit_fills_the_width_and_wraps_the_text() {
        // An image column that won't shrink (128) beside text that will (60..600).
        let widths = distribute(&[128.0, 60.0], &[128.0, 600.0], 400.0);
        assert_eq!(widths[0], 128.0, "the image keeps its width");
        assert!((widths.iter().sum::<f32>() - 400.0).abs() < 0.01, "the text takes the rest");
        // Even the least doesn't fit: everything shrinks, to a floor.
        let tight = distribute(&[300.0, 300.0], &[300.0, 300.0], 300.0);
        assert!(tight.iter().all(|w| *w == 150.0));
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
