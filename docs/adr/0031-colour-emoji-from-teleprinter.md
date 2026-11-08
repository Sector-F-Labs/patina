# 0031. Colour emoji are Patina's: the Teleprinter code, bundled and shared

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

- **`patina::emoji`, behind the `emoji` cargo feature**: the font (`assets/fonts/NotoColorEmoji.ttf`,
  OFL, licence beside it), `glyph(text)` for a sequence's glyph, `texture` and `paint` for its
  image, `runs(text)` to split text into plain and emoji runs, and `label(ui, text)` to show it.
- **Optional**, as `markdown` is, because it adds about 11 MB to a binary.
- **It is the Teleprinter code**, not a rewrite, so an emoji looks the same in every Patina app.
  Teleprinter can drop its copy for this when it next changes.

## Consequences

- An app gets colour emoji by enabling the feature and using `emoji::label` where its text may
  have emoji. egui's own labels, text edits and widgets still draw monochrome ones; that needs egui
  to draw colour glyphs, or a widget of our own for each. The markdown `Editor` is one such: with
  both features on, it lays emoji out in transparent text and paints the image over them.
- The packaged crate is about 10 MB compressed, at crates.io's limit. Publishing with this feature
  needs checking first; Sector F apps build on `develop`, so it doesn't stop them.
- `label` lays plain text and emoji out as separate widgets in a wrapped row, so it wraps between
  runs and the line height can differ slightly from a plain label's.
