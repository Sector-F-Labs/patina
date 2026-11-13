# 0031. Colour emoji are Patina's: the Teleprinter code, with the font a git crate

Date: 2026-11-07

## Status

Accepted. The user saw, in the other Patina apps, the emoji problems Teleprinter had already
solved, and asked for Patina to bundle that emoji set. The patina-desktop emoji picker is the second
user (its ADR-0120).

## Context

egui (epaint 0.35) draws each glyph from its outline into a coverage-only atlas tinted with the text
colour, and never reads COLR, CBDT or sbix, so emoji in any egui label are monochrome shapes from
egui's own emoji fonts, or boxes. Teleprinter solved it (its ADR-0013) by bundling Noto Color Emoji's
CBDT font and painting emoji as images, found through the font's cmap and GSUB ligatures. The
picker copied that code, and every other app would have had to.

## Decision

- **`patina::emoji`, behind the `emoji` cargo feature**: `glyph(text)` for a sequence's glyph,
  `texture` and `paint` for its image, `runs(text)` to split text into plain and emoji runs, and
  `label(ui, text)` to show it.
- **The font is the app's to bring**, not part of `patina-ui` and not looked for by it. It is 10
  MB, which put the package over crates.io's 10 MiB limit (a cargo feature doesn't help: the limit
  is on the uploaded package, features or not), and where a system font lives, or whether there is
  one, differs on every platform Patina runs on (Linux, macOS), so Patina doesn't guess.
  `emoji::install(bytes)` takes the bytes of any Noto Color Emoji in its CBDT build (PNG images,
  the format `Face::glyph_raster_image` reads; a font without them is refused), and
  `emoji::install_file(path)` reads a file. The app decides where the font comes from: it ships
  one, finds one, or lets the user choose.
- **Sector F's desktop bundles ours**: `patina-emoji-font` (`emoji-font/` in this repository, a
  workspace member with `publish = false`) is the font as `FONT: &[u8]`, and the apps that show
  emoji call `install(patina_emoji_font::FONT)`. It is a git dependency, which crates.io refuses,
  so `patina-ui` can't depend on it (only its tests do, as a dev-dependency), and an app published
  there brings its own font instead.
- **Without a font, nothing breaks**: `install` and the others say `false`, `runs` finds no
  emoji, `label` is a plain label, and emoji look as they did before.
- **Optional**, as `markdown` is, because it adds about 11 MB to a binary.
- **It is the Teleprinter code**, not a rewrite, so an emoji looks the same in every Patina app.
  Teleprinter can drop its copy for this when it next changes.

## Consequences

- An app gets colour emoji by enabling the feature, installing a font and using `emoji::label`
  where its text may have emoji. egui's own labels, text edits and widgets still draw monochrome
  ones; that needs egui to draw colour glyphs, or a widget of our own for each. The markdown `Editor` is one such: with
  both features on, it lays emoji out in transparent text and paints the image over them.
- The packaged crate is about 1 MiB, and publishes to crates.io. An app published there can't
  depend on the git-only `patina-emoji-font`, so it ships or finds a font itself. Only the CBDT
  build of Noto Color Emoji is read: not macOS's Apple Color Emoji (sbix), nor Noto's newer COLR
  build.
- `label` lays plain text and emoji out as separate widgets in a wrapped row, so it wraps between
  runs and the line height can differ slightly from a plain label's.
