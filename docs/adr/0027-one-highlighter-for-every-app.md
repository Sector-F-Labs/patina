# 0027. One highlighter for every app, a line at a time, from Text Editor

Date: 2026-10-25

## Status

Accepted. The user asked for Text Editor's highlighter to move into Patina, so Reader's source and
code blocks look the same as Text Editor.

## Context

Code blocks (`markdown::embeds::code`) and source files (`embeds::code_file`, Reader) highlighted
through `egui_extras`, which runs syntect over the whole text on every change, with syntect's 75
default languages (no TOML), and draws plain text in the theme's own grey.

Text Editor wrote its own (its ADR-0003 and ADR-0009): syntect a line at a time, each line keeping
the parser's state, so an edit recolours only until the state meets what it was; bat's 199
languages through `two-face`; and text in the theme's foreground drawn in the skin's text colour.
The same code then looked different in Text Editor and in Reader, and a second app wanted the
component, which is when it belongs in Patina (ADR-0001).

## Decision

`patina::highlight`, behind a `highlight` feature that `markdown` turns on, is Text Editor's
highlighter:

- `Highlighter`, kept by an editor across edits: `update` with the text, `advance` for a time
  budget, `job` for the coloured layout.
- `layout_job(ui, id, text, language, font)`: a whole text coloured, its highlighter kept in egui's
  memory under `id`, for code blocks and files shown to read.
- `detect` (a file name or `#!` line), `language_for` (a fence tag or extension), `languages`.

Code blocks and `code_file` use it, and `egui_extras` no longer builds syntect.

## Consequences

- Code reads the same in Text Editor, Reader and Notebook: the same languages, colours and plain
  text in the skin's text colour.
- TOML, TypeScript, Dockerfile and the rest of bat's set are highlighted in code blocks too.
- Patina carries syntect and two-face (about a megabyte of syntaxes) for apps with `highlight` or
  `markdown`.
- `make test` and `make check` build every feature, so the markdown and highlight code is tested.
