# 0026. Code blocks sit on the text-field background, not egui's code grey

Date: 2026-10-25

## Status

Accepted. The user found Reader's source files and code fences "quite awful with their light grey
background on default skin", and asked for Text Editor's look instead.

## Context

`markdown::embeds::code` filled code blocks with egui's `code_bg_color`. Every Patina skin sets that
and `extreme_bg_color` (text fields) to the same `surface` colour, so in skins the two never
differed. In Default, stock egui's, they do: `code_bg_color` is grey 230 in light and 64 in dark,
while text fields are white and near-black. Code blocks were a heavy grey slab in a white document.

Text Editor, a Patina app, draws its whole editing area on `extreme_bg_color`, and the user prefers
it.

## Decision

Code blocks are filled with `extreme_bg_color`, the text-field background. Inline code in the
markdown editor keeps `code_bg_color`: on a document it needs to stand out from the text around it.

## Consequences

- In Default, code blocks are white (light) or near-black (dark), like text fields and editors; in
  every skin nothing changes, since `surface` sets both.
- Apps painting source on `code_bg_color` themselves (Reader did) should change to match.
- A skin can no longer give code blocks a colour apart from text fields; that would need a new skin
  key.
