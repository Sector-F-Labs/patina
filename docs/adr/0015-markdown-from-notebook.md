# 0015. Notebook's markdown display moves into Patina, behind a feature

Date: 2026-10-16

## Status

Accepted. The user chose to share it in Patina, over a crate of its own or a copy in each app.

## Context

Notebook shows markdown with a WYSIWYG widget of its own (its ADR-0002): no markdown syntax on
screen, headings, lists, tasks, quotes, tables, highlighted code blocks and images, in the skin's
colours and Ubuntu's variable fonts. Reader, a read-only viewer for a project's markdown and
Rust files, should show documents the same way, and the plan is for it to become Patina's help
viewer, so any Patina app's Help opens documentation that looks like Notebook. A copy of the
widget in Reader would drift from Notebook's.

The widget brings dependencies most Patina apps don't need: pulldown-cmark, syntect through
egui_extras, image decoders, and 1.7 MB of fonts.

## Decision

- Notebook's `markdown`, `doc`, `editor`, `embeds` and `fonts` move to `patina::markdown`
  (`model`, `doc`, `editor`, `embeds`, `fonts`), with their tests, behind the cargo feature
  `markdown`; `markdown::install` sets up the fonts and image loaders.
- The editor gains a read-only mode (`Editor::set_read_only`): selecting and copying work, there's
  no caret, nothing edits (keys, paste, tasks, table and code embeds), the menu is Copy and Select
  All, and a plain click on a link is handed to the app (`Editor::take_link`) to follow, where
  editing needs Ctrl+click and opens the browser.
- `embeds::code_file` shows a whole source file highlighted the same way, with line numbers.
- Notebook depends on Patina's develop branch with the feature, and keeps only its app.

## Consequences

- Notebook, Reader and help show markdown identically; a fix in one is a fix in all.
- Apps without the feature build as before, with none of its dependencies.
- Notebook now follows Patina's develop branch rather than a release, like the desktop.
- Comments in the moved code still cite Notebook's ADRs, which recorded the decisions.
