# 0034. Text inputs are a button's size: `dialog::text_edit` and `[controls] input_padding`

Date: 2026-11-13

## Status

Accepted. The user saw the folder tree's filter box in Programmer next to the title bar's buttons
and the tabs, found it short and tight, and said inputs should always be at least the buttons'
size, adjustable by the user, without breaking the API or looking worse.

## Context

ADR-0032 gave buttons the title bar's size (10 by 5 px of padding, 24 px high). An input beside
them was egui's: 4 by 2 px of padding, about 22 px high. egui's `TextEdit` takes that margin from
no style: it is a field of the widget (`TextEdit::margin`), so it can't be set once on the
context as button padding can, and changing every `TextEdit` Patina draws from outside isn't
possible. The least height does come from `interact_size.y`, but raising that for a `ui` also
resizes everything else in it (ADR-0032).

## Decision

- **`dialog::text_edit(ui, &mut text)`** returns a single-line `TextEdit` with the margin and the
  least height of `dialog::input_size(ctx)`: by default `INPUT_PADDING` (a button's, 10 by 5) and
  `INPUT_HEIGHT` (24). It's still a `TextEdit`: everything chains on as before.
- **The size lives on the context**, as `InputSize { padding, height }`: `set_input_size(ctx, size)`
  sets it and `input_size(ctx)` reads it.
- **Skins adjust it**: `[controls] input_padding = [across, down]` and `input_height`.
  `Skin::apply` sets it, so a skin without them gives the default again. Values are held to 0 to
  24 px of padding and 16 to 64 px of height, so a skin can make inputs denser or looser but not
  unusable. An app that wants its own calls `set_input_size` after `apply`.
- **Patina's own single-line inputs use it**: the folder tree's filter box. The markdown editor's
  text areas and an image's caption, which are written in the text, are left as they were.
- Nothing is removed or changed in the API: `TextEdit` still works anywhere, and an app opts its
  inputs in by swapping `egui::TextEdit::singleline(&mut text)` for `dialog::text_edit(ui, &mut text)`.

## Consequences

- The tree's filter box is 2 px taller and its text 6 px further in, in every app, at once.
- An app's own inputs match only when they are made with `text_edit`; egui's are unchanged.
- The default skin and skins that don't mention the keys look as `develop` did, apart from the
  tree's box.
