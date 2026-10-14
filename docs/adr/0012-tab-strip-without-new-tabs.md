# 0012. TabStrip can leave out new tabs and name what a tab is

Date: 2026-10-14

## Status

Proposed. The user asked for the Patina desktop's bar to list open windows with Patina's tabs; the
options are my design.

## Context

`TabStrip` (ADR-0009) always ends with `+` and offers "New Tab" and "Close Tab" in a tab's menu,
because Teleprinter's tabs are things the app makes. The Patina desktop's bar shows the open windows
as tabs. The bar doesn't make windows (that's the launcher), and a tab there is a window, so "+"
does nothing useful and "Close Tab" says the wrong thing.

## Decision

Two builder options, defaulting to today's behaviour:

- `.new_tab(false)` leaves out the `+` tab and the menu's "New" item. `TabAction::New` then never
  comes from the strip.
- `.noun("Window")` names a tab in the menu and the close button's tooltip: "New Window", "Close
  Window". The default is "Tab".

## Consequences

- Existing callers are unchanged.
- The keyboard shortcuts (`tabs::keys`) still say nothing of nouns; an app that doesn't want
  Ctrl+Shift+T simply doesn't call them.
