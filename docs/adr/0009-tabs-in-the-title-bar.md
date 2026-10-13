# 0009. Tabs in the title bar, as a component apps fill with their own tabs

Date: 2026-10-13

## Status

Proposed. The user asked for Teleprinter's tabs to move into Patina so other apps can use them; the
API is my design.

## Context

Teleprinter put tabs in the title bar after its menus, in place of the window title (its ADR-0012):
square, the bar's full height, the shown one filled in the panel colour to join the panel below, a
close cross on hover, `+`, middle click and a right-click menu, and Ctrl+Shift+T and
Ctrl+PageUp/PageDown. Its tabs are layouts of terminal panes, but none of the strip depends on that.
Other apps (Notebook's open files, say) would want the same thing, and drawing it again in each
would let them drift (ADR-0007).

## Decision

- **`patina::tabs::TabStrip`**, a builder like `TitleBar`: `TabStrip::new(titles, shown, &style)`,
  with `.span(range)` for the bar's full height, `.id_salt` and `.new_hint` (the `+` tooltip),
  finished by `.show(ui)` at the end of the title bar's left slot. The title bar's own title is
  left empty.
- **The app owns its tabs.** The strip takes their titles and the shown index and returns an
  `Option<TabAction>` (`New`, `Show(i)`, `Close(i)`), which the app applies. Patina keeps no tab
  state, so a tab can be anything.
- **`tabs::keys(ctx, count, shown)`** takes Ctrl+Shift+T and Ctrl+PageUp/PageDown, wrapping round,
  and returns the same actions. Apps call it before their widgets, since it consumes the keys.
- What closing the last tab does is the app's call (Teleprinter closes the window).

## Consequences

- Apps with tabs get the same look and keys, and Teleprinter loses ~100 lines.
- The keys are fixed. An app whose widgets need Ctrl+PageUp/PageDown can skip `keys`.
- Not yet: reordering by drag, renaming, overflow scrolling when tabs hit their 48 px minimum, or
  tabs anywhere but the title bar. These would be builder options.
