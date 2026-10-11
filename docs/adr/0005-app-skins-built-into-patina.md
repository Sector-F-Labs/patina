# 0005. Build Calendar, Notebook and Steam Classic into Patina, and colour controls on a coloured bar

Date: 2026-10-11

## Status

Accepted: the user asked for the Notebook, Calendar and Steam Classic skins to move into Patina's
core so every app offers them. Moving `bar_controls` into `TitleBar` follows from that and was
already Patina's rule for code a second app wants (ADR-0001).

## Context

Calendar and Notebook each compiled in a skin of their own (a red and a brown title bar on paper),
and UnAmp ships Steam Classic. Patina's only built-in was Dark Orange (ADR-0004).

A skin with a coloured title bar needs the bar's menus and buttons recoloured to read on it.
Calendar and Notebook each had an identical `bar_controls` for this. Offered in any other app
without it, Calendar and Notebook would draw dark menus on red and brown.

## Decision

- **Built-ins:** `skin::BUILT_IN` holds Dark Orange, Calendar, Notebook and Steam Classic, all
  credited to Patina. The files are copied unchanged from the apps apart from `author` and
  comments. Steam Classic keeps its `[app.unamp]` section, as Dark Orange does.
- **Bar controls:** `TitleBar` colours its left and right controls whenever the skin's
  `title_bar` differs from the panel colour: text in `title_text`, fills from `[window.controls]`
  hover and pressed, and a wash of `title_text` at rest. Popups keep the skin's normal colours.
  Bars in the panel colour (Default, Dark Orange, Steam Classic) are untouched.
- **Title:** the title is painted in `title_text` explicitly, rather than the ui's strong text
  colour.
- **Apps keep choosing their starting skin.** Calendar starts in Calendar and Notebook in
  Notebook; that stays in each app.
- Released as patina-ui 0.1.2.

## Consequences

- Every Patina app offers five skins out of the box, and the gallery shows them all.
- Calendar and Notebook drop their own skin files and `bar_controls`. A change to either skin is
  now a Patina release.
- UnAmp isn't on Patina yet, so it keeps its own copy of Steam Classic until it switches.
- Skin names are now shared across apps: a user's skin named "Calendar" replaces the built-in in
  every app.
