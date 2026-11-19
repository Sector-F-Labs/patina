//! Colour emoji (feature `emoji`, ADR-0031). egui draws every glyph from its
//! outline in one colour and never reads a colour font, so emoji in egui text
//! come out as monochrome shapes, or as boxes. This module paints emoji as
//! images from Noto Color Emoji (its CBDT build, SIL OFL), which the app
//! installs (`install`, `install_file`): Patina doesn't look for one. The
//! font's cmap and GSUB ligatures turn a sequence (a ZWJ family, a skin
//! tone, a flag, a keycap) into its glyph, and the glyph's 136 px PNG
//! becomes a texture, made once per glyph.
//!
//! - [`label`] shows text with its emoji in colour, wrapping like a label.
//! - [`runs`] splits text into plain and emoji runs, for a widget that lays
//!   text out itself (a terminal, a text editor).
//! - [`glyph`] and [`paint`] are the pieces under both: a sequence's glyph,
//!   and an image of it in a rectangle.
//!
//! Teleprinter solved this first (its ADR-0013); this is that code, shared.

use std::collections::HashMap;
use std::sync::OnceLock;

use egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui, pos2, vec2};
use ttf_parser::gsub::SubstitutionSubtable;
use ttf_parser::{Face, GlyphId, RasterImageFormat};

/// The font, handed over by the app with [`install`].
static FONT: OnceLock<&'static [u8]> = OnceLock::new();

/// Gives this module a colour emoji font: the bytes of a Noto Color Emoji
/// (its CBDT build, with PNG images). Call it once at start-up (ADR-0031).
/// The font isn't part of this crate, for it is 10 MB, over crates.io's
/// limit, and where one lives differs by platform: the app brings its own
/// (bundled, found, or chosen by the user), through this or [`install_file`].
/// Sector F's desktop uses the `patina-emoji-font` git crate.
/// Until a font is installed, emoji stay in the plain text, as monochrome
/// shapes or boxes. The first font installed is the one used; a font that
/// has no emoji images is refused, and `false` says so.
pub fn install(font: &'static [u8]) -> bool {
    if FONT.get().is_some() {
        return true;
    }
    if !usable(font) {
        return false;
    }
    let _ = FONT.set(font);
    true
}

/// Installs the font at `path` (read once and kept for the life of the
/// program). `false` if it can't be read or isn't a CBDT colour emoji font.
pub fn install_file(path: &std::path::Path) -> bool {
    if FONT.get().is_some() {
        return true;
    }
    match std::fs::read(path) {
        Ok(bytes) if usable(&bytes) => install(Box::leak(bytes.into_boxed_slice())),
        _ => false,
    }
}

/// Whether `font` is one this module can draw from: it parses, and has a
/// PNG image for 😀.
fn usable(font: &[u8]) -> bool {
    Face::parse(font, 0).is_ok_and(|face| {
        face.glyph_index('\u{1f600}').is_some_and(|g| {
            face.glyph_raster_image(g, u16::MAX)
                .is_some_and(|r| r.format == RasterImageFormat::PNG)
        })
    })
}

const ZWJ: char = '\u{200d}';
const VS16: char = '\u{fe0f}';
const KEYCAP: char = '\u{20e3}';

fn face() -> Option<&'static Face<'static>> {
    static FACE: OnceLock<Option<Face<'static>>> = OnceLock::new();
    #[cfg(test)]
    let _ = install(patina_emoji_font::FONT);
    // Not cached until the font is in, so an install after a first look still works.
    let font = FONT.get()?;
    FACE.get_or_init(|| Face::parse(font, 0).ok()).as_ref()
}

/// An emoji's glyph in the bundled font.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Glyph(u16);

/// The one glyph `text` becomes, if the font has it: a single emoji or a
/// whole sequence.
pub fn glyph(text: &str) -> Option<Glyph> {
    let face = face()?;
    // Noto's sequences mostly leave VS16 out; try as written first.
    let stripped: String = text.chars().filter(|&c| c != VS16).collect();
    [text, stripped.as_str()].into_iter().find_map(|text| {
        let mut glyphs = text
            .chars()
            .map(|c| face.glyph_index(c))
            .collect::<Option<Vec<_>>>()?;
        ligate(face, &mut glyphs);
        match glyphs[..] {
            [one] => Some(Glyph(one.0)),
            _ => None,
        }
    })
}

/// Applies the font's ligature substitutions, which is all Noto's GSUB holds.
fn ligate(face: &Face, glyphs: &mut Vec<GlyphId>) {
    let Some(gsub) = face.tables().gsub else {
        return;
    };
    for lookup in gsub.lookups {
        let mut i = 0;
        while i < glyphs.len() {
            let found = lookup
                .subtables
                .into_iter::<SubstitutionSubtable>()
                .find_map(|subtable| {
                    let SubstitutionSubtable::Ligature(ligatures) = subtable else {
                        return None;
                    };
                    let set = ligatures
                        .ligature_sets
                        .get(ligatures.coverage.get(glyphs[i])?)?;
                    set.into_iter().find(|l| {
                        let rest = &glyphs[i + 1..];
                        l.components.len() as usize <= rest.len()
                            && l.components.into_iter().zip(rest).all(|(a, &b)| a == b)
                    })
                });
            if let Some(ligature) = found {
                glyphs.splice(i..=i + ligature.components.len() as usize, [ligature.glyph]);
            }
            i += 1;
        }
    }
}

/// Textures made so far, per glyph; `None` for a glyph with no image.
#[derive(Clone, Default)]
struct Cache(HashMap<Glyph, Option<TextureHandle>>);

/// The glyph's image as a texture, made the first time and kept.
pub fn texture(ctx: &egui::Context, glyph: Glyph) -> Option<TextureHandle> {
    let id = egui::Id::new("patina-emoji");
    let known = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Cache>(id)
            .0
            .get(&glyph)
            .cloned()
    });
    known.unwrap_or_else(|| {
        let made = load(ctx, glyph);
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<Cache>(id)
                .0
                .insert(glyph, made.clone())
        });
        made
    })
}

fn load(ctx: &egui::Context, glyph: Glyph) -> Option<TextureHandle> {
    let raster = face()?.glyph_raster_image(GlyphId(glyph.0), u16::MAX)?;
    if raster.format != RasterImageFormat::PNG {
        return None;
    }
    let rgba = image::load_from_memory_with_format(raster.data, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    let image =
        ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], &rgba);
    // Drawn at a fraction of the 136 px strike: mipmaps keep it from aliasing.
    let options = TextureOptions {
        mipmap_mode: Some(egui::TextureFilter::Linear),
        ..TextureOptions::LINEAR
    };
    Some(ctx.load_texture(format!("emoji-{}", glyph.0), image, options))
}

/// Paints `glyph`'s image centred in `rect`, as large as fits. Nothing if
/// the font has no image for it.
pub fn paint(painter: &egui::Painter, glyph: Glyph, rect: Rect) {
    let Some(texture) = texture(painter.ctx(), glyph) else {
        return;
    };
    let size = texture.size_vec2();
    let scale = (rect.width() / size.x).min(rect.height() / size.y);
    let target = Rect::from_center_size(rect.center(), size * scale);
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    painter.image(texture.id(), target, uv, Color32::WHITE);
}

/// A stretch of text: plain, or one emoji (a sequence) with its glyph.
#[derive(Clone, Debug, PartialEq)]
pub enum Run<'a> {
    Text(&'a str),
    Emoji(&'a str, Glyph),
}

/// Splits `text` into plain runs and emoji runs. What counts as an emoji is
/// a character that is emoji by default (pictographs, flags), or any with
/// VS16 or a keycap mark after it, plus what carries a sequence on: ZWJ and
/// what it joins, skin tones, a flag's second letter. So `1` and `#` stay
/// text unless keycapped, and `❤` is text, `❤️` an emoji. What the font has
/// no glyph for stays text.
pub fn runs(text: &str) -> Vec<Run<'_>> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        let next = chars.peek().map(|&(_, n)| n);
        if !starts_emoji(c, next) {
            continue;
        }
        // Take the sequence.
        let mut end = start + c.len_utf8();
        let mut last = c;
        while let Some(&(at, n)) = chars.peek() {
            let joins = n == VS16
                || n == KEYCAP
                || is_skin_tone(n)
                || last == ZWJ
                || n == ZWJ
                || (is_regional(last) && is_regional(n) && !is_pair(&text[start..end]));
            if !joins {
                break;
            }
            chars.next();
            end = at + n.len_utf8();
            last = n;
        }
        // One the font has no glyph for stays in the plain text.
        if let Some(g) = glyph(&text[start..end]) {
            if plain < start {
                out.push(Run::Text(&text[plain..start]));
            }
            out.push(Run::Emoji(&text[start..end], g));
            plain = end;
        }
    }
    if plain < text.len() {
        out.push(Run::Text(&text[plain..]));
    }
    out
}

fn starts_emoji(c: char, next: Option<char>) -> bool {
    if c.is_ascii() && !c.is_ascii_digit() && c != '#' && c != '*' {
        return false;
    }
    let marked = matches!(next, Some(VS16 | KEYCAP));
    marked
        || matches!(c as u32,
            0x1F300..=0x1F64F | 0x1F680..=0x1F6FF | 0x1F900..=0x1FAFF | 0x1F1E6..=0x1F1FF
            | 0x2705 | 0x274C | 0x2B50 | 0x2B55 | 0x231A | 0x231B | 0x23E9..=0x23EC | 0x23F0
            | 0x23F3 | 0x25FD | 0x25FE | 0x2614 | 0x2615 | 0x2648..=0x2653 | 0x267F | 0x2693
            | 0x26A1 | 0x26AA | 0x26AB | 0x26BD | 0x26BE | 0x26C4 | 0x26C5 | 0x26CE | 0x26D4
            | 0x26EA | 0x26F2 | 0x26F3 | 0x26F5 | 0x26FA | 0x26FD | 0x270A | 0x270B | 0x2728
            | 0x274E | 0x2753..=0x2755 | 0x2757 | 0x2795..=0x2797 | 0x27B0 | 0x27BF | 0x2B1B
            | 0x2B1C | 0x1F004 | 0x1F0CF | 0x1F18E | 0x1F191..=0x1F19A | 0x1F201 | 0x1F21A
            | 0x1F22F | 0x1F232..=0x1F236 | 0x1F238..=0x1F23A | 0x1F250 | 0x1F251)
}

fn is_regional(c: char) -> bool {
    ('\u{1f1e6}'..='\u{1f1ff}').contains(&c)
}

fn is_skin_tone(c: char) -> bool {
    ('\u{1f3fb}'..='\u{1f3ff}').contains(&c)
}

/// Whether `s` is already two regional indicators: a flag is a pair.
fn is_pair(s: &str) -> bool {
    s.chars().filter(|&c| is_regional(c)).count() >= 2
}

/// Text with its emoji in colour: plain stretches as labels, emoji as
/// images the height of a line, wrapping at the width available.
pub fn label(ui: &mut Ui, text: &str) {
    let size = egui::TextStyle::Body.resolve(ui.style()).size;
    label_with(ui, text, size, |t| t);
}

/// [`label`] with the text's size and style chosen: `style` turns each plain
/// stretch (already `size` points) into the rich text to show, to make it
/// strong, weak, coloured. Emoji are the height of a line at `size`.
pub fn label_with(
    ui: &mut Ui,
    text: &str,
    size: f32,
    style: impl Fn(egui::RichText) -> egui::RichText,
) {
    let height = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(size)));
    let parts = runs(text);
    if parts.iter().all(|r| matches!(r, Run::Text(_))) {
        ui.label(style(egui::RichText::new(text).size(size)));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for run in parts {
            match run {
                Run::Text(t) => {
                    ui.label(style(egui::RichText::new(t).size(size)));
                }
                Run::Emoji(_, g) => {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(height * 1.15, height), egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        paint(ui.painter(), g, rect);
                    }
                }
            }
        }
    });
}

/// An emoji [`hide`] laid out invisibly in a job: where it starts and how
/// many characters it takes (in characters of the job's text), and its glyph.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    start: usize,
    len: usize,
    glyph: Glyph,
}

/// Makes the emoji in `job` invisible, so they take room, and the cursor and
/// selection of a text edit find them, and says where they are for
/// [`paint_placed`] to draw in colour over them (egui draws no colour). Call
/// it on the job a `TextEdit` layouter makes, and paint after the edit.
/// Nothing is changed, and nothing found, for text without emoji or before
/// a font is installed.
pub fn hide(job: &mut egui::text::LayoutJob) -> Vec<Placed> {
    let mut placed = Vec::new();
    if job.text.is_ascii() || FONT.get().is_none() {
        return placed;
    }
    let sections = std::mem::take(&mut job.sections);
    let (mut chars, mut at) = (0, 0);
    for section in sections {
        let range = section.byte_range.start.0..section.byte_range.end.0;
        chars += job.text[at..range.start].chars().count();
        at = range.end;
        let parts = runs(&job.text[range.clone()]);
        if parts.iter().all(|r| matches!(r, Run::Text(_))) {
            chars += job.text[range].chars().count();
            job.sections.push(section);
            continue;
        }
        let mut from = range.start;
        for (i, run) in parts.into_iter().enumerate() {
            let (text, glyph) = match run {
                Run::Text(t) => (t, None),
                Run::Emoji(t, g) => (t, Some(g)),
            };
            let mut format = section.format.clone();
            let len = text.chars().count();
            if let Some(glyph) = glyph {
                placed.push(Placed {
                    start: chars,
                    len,
                    glyph,
                });
                format.color = Color32::TRANSPARENT;
            }
            job.sections.push(egui::text::LayoutSection {
                leading_space: if i == 0 { section.leading_space } else { 0.0 },
                byte_range: egui::text::ByteIndex(from)..egui::text::ByteIndex(from + text.len()),
                format,
            });
            from += text.len();
            chars += len;
        }
    }
    placed
}

/// Draws the emoji [`hide`] found, in colour, over their invisible places in
/// `galley`, whose top left is at `origin` (a `TextEdit`'s `galley_pos`).
pub fn paint_placed(
    painter: &egui::Painter,
    galley: &egui::Galley,
    origin: egui::Pos2,
    placed: &[Placed],
) {
    let clip = painter.clip_rect();
    for p in placed {
        let mut row_start = 0;
        for row in &galley.rows {
            let count = row.char_count_including_newline().0;
            let row_end = row_start + row.row.char_count_excluding_newline().0;
            if p.start >= row_start && p.start < row_end.max(row_start + 1) {
                let band = row.rect();
                if origin.y + band.max.y >= clip.min.y && origin.y + band.min.y <= clip.max.y {
                    let end = (p.start + p.len).min(row_end);
                    let x0 =
                        row.pos.x + row.row.x_offset(egui::text::CharIndex(p.start - row_start));
                    let x1 = row.pos.x + row.row.x_offset(egui::text::CharIndex(end - row_start));
                    let w = (x1 - x0).max(band.height());
                    let rect = Rect::from_min_size(
                        pos2(origin.x + x0, origin.y + band.min.y),
                        vec2(w, band.height()),
                    );
                    paint(painter, p.glyph, rect);
                }
                break;
            }
            row_start += count;
        }
    }
}

/// One line of text laid out for painting with its emoji in colour: what a
/// row drawn with a [`egui::Painter`] (a file's name, a notification's
/// title) uses where [`label`] doesn't fit, because it can't be cut at a
/// width or placed. Made with [`line`], painted with [`Line::paint`].
pub struct Line {
    parts: Vec<(f32, Part)>,
    size: egui::Vec2,
}

enum Part {
    Text(std::sync::Arc<egui::Galley>),
    Emoji(Glyph, f32),
}

impl Line {
    /// The line's width and height.
    pub fn size(&self) -> egui::Vec2 {
        self.size
    }

    /// Paints the line with its top left at `pos`; plain parts take their
    /// colour from [`line`].
    pub fn paint(&self, painter: &egui::Painter, pos: egui::Pos2) {
        for (x, part) in &self.parts {
            match part {
                Part::Text(galley) => {
                    let y = (self.size.y - galley.size().y) / 2.0;
                    painter.galley(pos + vec2(*x, y), galley.clone(), Color32::PLACEHOLDER);
                }
                Part::Emoji(glyph, width) => {
                    let rect = Rect::from_min_size(pos + vec2(*x, 0.0), vec2(*width, self.size.y));
                    paint(painter, *glyph, rect);
                }
            }
        }
    }
}

/// Lays `text` out on one line in `font` and `colour`, its emoji as images
/// the height of the line, cut with an ellipsis where it passes `max_width`
/// (`f32::INFINITY` for no limit). Without a font installed, or without
/// emoji in the text, it is one plain galley.
pub fn line(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    colour: Color32,
    max_width: f32,
) -> Line {
    let height = painter.fonts_mut(|f| f.row_height(&font));
    let mut parts = Vec::new();
    let mut x = 0.0_f32;
    let mut cut = false;
    for run in runs(text) {
        if cut {
            break;
        }
        match run {
            Run::Text(t) => {
                let mut job =
                    egui::text::LayoutJob::simple_singleline(t.to_owned(), font.clone(), colour);
                if max_width.is_finite() {
                    job.wrap =
                        egui::text::TextWrapping::truncate_at_width((max_width - x).max(0.0));
                }
                let galley = painter.layout_job(job);
                cut = galley.elided;
                let width = galley.size().x;
                parts.push((x, Part::Text(galley)));
                x += width;
            }
            Run::Emoji(_, glyph) => {
                let width = height * 1.15;
                if x + width > max_width {
                    let job = egui::text::LayoutJob::simple_singleline(
                        "…".to_owned(),
                        font.clone(),
                        colour,
                    );
                    let galley = painter.layout_job(job);
                    parts.push((x, Part::Text(galley.clone())));
                    x += galley.size().x;
                    cut = true;
                } else {
                    parts.push((x, Part::Emoji(glyph, width)));
                    x += width;
                }
            }
        }
    }
    Line {
        parts,
        size: vec2(x, height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_font_without_emoji_is_refused() {
        // Ubuntu's text font has no colour images; and neither is a bare file.
        assert!(!usable(include_bytes!(
            "../assets/fonts/Ubuntu-Variable.ttf"
        )));
        assert!(!usable(b"not a font"));
        assert!(usable(patina_emoji_font::FONT));
    }

    fn emoji_of(text: &str) -> Vec<&str> {
        runs(text)
            .into_iter()
            .filter_map(|r| match r {
                Run::Emoji(e, _) => Some(e),
                Run::Text(_) => None,
            })
            .collect()
    }

    #[test]
    fn emoji_and_sequences_become_one_glyph() {
        for text in [
            "🐡",
            "🐧",
            "❤\u{fe0f}",
            "👩\u{200d}👩\u{200d}👧",
            "👋🏽",
            "🇸🇪",
            "1\u{fe0f}\u{20e3}",
        ] {
            assert!(glyph(text).is_some(), "{text:?}");
        }
        assert_ne!(glyph("👋🏽"), glyph("👋"), "the skin tone is its own glyph");
        assert!(glyph("A").is_none() && glyph("中").is_none(), "not emoji");
    }

    #[test]
    fn text_splits_around_its_emoji() {
        assert_eq!(emoji_of("hi 🐡 and 🐧!"), ["🐡", "🐧"]);
        let r = runs("a🐡b");
        assert_eq!(r.len(), 3);
        assert_eq!(r[0], Run::Text("a"));
        assert_eq!(r[2], Run::Text("b"));
        assert_eq!(runs("plain text"), [Run::Text("plain text")]);
        assert!(runs("").is_empty());
    }

    #[test]
    fn sequences_stay_whole() {
        assert_eq!(
            emoji_of("👩\u{200d}👩\u{200d}👧!"),
            ["👩\u{200d}👩\u{200d}👧"]
        );
        assert_eq!(emoji_of("👋🏽"), ["👋🏽"]);
        assert_eq!(emoji_of("🇸🇪🇬🇧"), ["🇸🇪", "🇬🇧"], "flags are pairs");
        assert_eq!(emoji_of("1\u{fe0f}\u{20e3}"), ["1\u{fe0f}\u{20e3}"]);
    }

    #[test]
    fn a_line_is_cut_at_its_width_and_keeps_its_emoji_in_line() {
        assert!(install(patina_emoji_font::FONT));
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let painter = ui.painter();
            let font = egui::FontId::proportional(14.0);
            let white = Color32::WHITE;
            let plain = line(painter, "plain", font.clone(), white, f32::INFINITY);
            let with = line(painter, "plain🐡", font.clone(), white, f32::INFINITY);
            assert!(
                with.size().x > plain.size().x + 10.0,
                "the emoji takes room"
            );
            assert_eq!(with.size().y, plain.size().y);
            let cut = line(painter, "a long name 🐡 with more", font, white, 60.0);
            assert!(cut.size().x <= 60.5, "{}", cut.size().x);
        });
    }

    #[test]
    fn hidden_emoji_are_found_by_character_and_laid_out_invisibly() {
        assert!(install(patina_emoji_font::FONT));
        let mut job = egui::text::LayoutJob::simple_singleline(
            "é🐡b🇸🇪".into(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );
        let placed = hide(&mut job);
        assert_eq!(
            placed.iter().map(|p| (p.start, p.len)).collect::<Vec<_>>(),
            [(1, 1), (3, 2)]
        );
        let hidden: Vec<bool> = job
            .sections
            .iter()
            .map(|s| s.format.color == Color32::TRANSPARENT)
            .collect();
        assert_eq!(hidden, [false, true, false, true]);
        let covered: usize = job
            .sections
            .iter()
            .map(|s| s.byte_range.end.0 - s.byte_range.start.0)
            .sum();
        assert_eq!(covered, job.text.len());
        let mut plain = egui::text::LayoutJob::simple_singleline(
            "ascii".into(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );
        assert!(hide(&mut plain).is_empty());
    }

    #[test]
    fn digits_symbols_and_unmarked_hearts_stay_text() {
        assert_eq!(runs("#1 * 2 ❤ ©"), [Run::Text("#1 * 2 ❤ ©")]);
        assert_eq!(emoji_of("I ❤\u{fe0f} you"), ["❤\u{fe0f}"]);
    }
}
