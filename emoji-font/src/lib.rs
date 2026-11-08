//! Noto Color Emoji, as the bytes of its CBDT font, for `patina::emoji`
//! (Patina ADR-0031). It is its own crate because the font is 10 MB, over
//! crates.io's size limit for `patina-ui`: an app that wants colour emoji
//! depends on this one from git and hands the font over once at start-up.
//!
//! ```ignore
//! patina::emoji::install(patina_emoji_font::FONT);
//! ```
//!
//! Noto Color Emoji is Google's, under the SIL Open Font License
//! (`NotoColorEmoji-LICENSE.txt`).

/// The font file.
pub static FONT: &[u8] = include_bytes!("../NotoColorEmoji.ttf");
