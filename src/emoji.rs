//! Colour emoji (feature `emoji`, ADR-0031). egui draws every glyph from its
//! outline in one colour and never reads a colour font, so emoji in egui text
//! come out as monochrome shapes, or as boxes. This module paints emoji as
//! images from Noto Color Emoji (its CBDT build, SIL OFL), which the app
//! installs: the font is the `patina-emoji-font` crate, git only. The
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

/// Gives this module the Noto Color Emoji font: call it once at start-up
/// with `patina_emoji_font::FONT` (ADR-0031). The font isn't part of this
/// crate, for it is 10 MB, over crates.io's limit. Until it is installed,
/// emoji stay in the plain text, as monochrome shapes or boxes. A second
/// call does nothing.
pub fn install(font: &'static [u8]) {
    let _ = FONT.set(font);
}

const ZWJ: char = '\u{200d}';
const VS16: char = '\u{fe0f}';
const KEYCAP: char = '\u{20e3}';

fn face() -> Option<&'static Face<'static>> {
    static FACE: OnceLock<Option<Face<'static>>> = OnceLock::new();
    #[cfg(test)]
    install(patina_emoji_font::FONT);
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
    let height = ui.text_style_height(&egui::TextStyle::Body);
    let parts = runs(text);
    if parts.iter().all(|r| matches!(r, Run::Text(_))) {
        ui.label(text);
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for run in parts {
            match run {
                Run::Text(t) => {
                    ui.label(t);
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn digits_symbols_and_unmarked_hearts_stay_text() {
        assert_eq!(runs("#1 * 2 ❤ ©"), [Run::Text("#1 * 2 ❤ ©")]);
        assert_eq!(emoji_of("I ❤\u{fe0f} you"), ["❤\u{fe0f}"]);
    }
}
