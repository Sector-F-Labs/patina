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
- **The font is a separate crate, `patina-emoji-font`**, in this repository (`emoji-font/`, a
  workspace member with `publish = false`; Noto Color Emoji, OFL, licence beside it). It is 10 MB,
  which put the `patina-ui` package over crates.io's 10 MiB limit, and a cargo feature doesn't
  help: the limit is on the uploaded package, features or not. So `patina-ui` doesn't contain the
  font. The app depends on `patina-emoji-font` from git and gives the font over once at start-up
  with `emoji::install(patina_emoji_font::FONT)`. `patina-ui` can't depend on the crate itself,
  because crates.io refuses a crate with a git-only dependency. Only its tests do, as a
  dev-dependency, which publishing leaves out.
- **Without `install`, nothing breaks**: `runs` finds no emoji, `label` is a plain label, and emoji
  look as they did before.
- **Optional**, as `markdown` is, because it adds about 11 MB to a binary.
- **It is the Teleprinter code**, not a rewrite, so an emoji looks the same in every Patina app.
  Teleprinter can drop its copy for this when it next changes.

## Consequences

- An app gets colour emoji by enabling the feature, depending on `patina-emoji-font` from git,
  calling `emoji::install` and using `emoji::label` where its text may have emoji. egui's own labels, text edits and widgets still draw monochrome ones; that needs egui
  to draw colour glyphs, or a widget of our own for each. The markdown `Editor` is one such: with
  both features on, it lays emoji out in transparent text and paints the image over them.
- The packaged crate is about 1 MiB, and publishes to crates.io. Colour emoji need git
  dependencies, so an app that must itself be published to crates.io can't have them, for the same
  reason. Sector F apps build on `develop`, so it doesn't affect them.
- `label` lays plain text and emoji out as separate widgets in a wrapped row, so it wraps between
  runs and the line height can differ slightly from a plain label's.
