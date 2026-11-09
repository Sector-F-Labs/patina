# 0033. Tables are laid out like HTML's, and hold images

Date: 2026-11-09

## Status

Accepted. The user found the markdown tables wrong: they couldn't show images in cells, had gaps
between the cells, and filled the width whatever they held. They asked for what an HTML table does.

## Context

The editor's table was an `egui::Grid` of single-line text fields, each 140 px wide with 4 px
between them. So a cell never wrapped (long text scrolled inside its field), an image in a cell
showed as its `![alt](src)` source, the cells were separate boxes rather than a table, and a table
was as wide as 140 px times its columns however little was in it.

A cell's text is its inline markdown source (Notebook's ADR-0005), and that source already holds
images (`![alt](src)`, `<img>`) and writes back as it was; only drawing them was missing.

## Decision

- **A table is laid out as an HTML auto-width table:** it is as wide as its content wants, up to the
  width of the text, and no wider. Each column wants, at most, its widest cell on one line and, at
  least, its longest word or its widest image. If every column gets its most, that's the width;
  else each gets its least and the room left is shared by how much more each wants, so text wraps
  and an image keeps its own size. Columns are never under 48 px, so empty cells can be clicked.
- **Cells are joined,** with 1 px borders between them and around the table, the header row on a
  faint fill, and 8 by 5 px inside each cell. Colours come from the visuals.
- **A cell wraps its text** (a multi-line field that Enter doesn't break). One with images shows them
  and its text as they look, one under another, scaled down only if the column can't be as wide
  as the image; clicking it edits its markdown source, until it loses focus. `<br>` is a line break.
- **`embeds::table` takes the document's folder,** to find relative images, as `embeds::image` does.

## Consequences

- Tables look like the ones in a browser, and READMEs with badge or screenshot tables show them.
- A cell's other markdown (bold, links) is still shown as its source: drawing it is a separate step.
- Column widths are measured every frame: each cell's text is laid out once to measure it. Fine for
  the tables notes have; a table of thousands of cells would want the widths kept.
- An image still loading takes 64 px square, so the table is laid out again when it arrives.
