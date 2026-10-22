# 0021. Tabs can be dragged into another order, and some tabs can be launchers

Date: 2026-10-23

## Status

Proposed. The user asked for the Patina desktop bar to pin apps and to drag its tabs into
another order (patina-desktop ADR-0080). The bar's window list is a `TabStrip` (ADR-0012,
ADR-0018). The API is my design.

## Context

A `TabStrip`'s tabs stayed in the order the app gave them. Browsers, terminals and taskbars let
you drag a tab to another place, and the desktop bar now needs it. The bar also shows pinned apps
that aren't running. Those tabs stand for something to start, not something to show.

A `TabAction::Move` variant was considered and ruled out. Teleprinter, Notebook, Files and Reader
all match `TabAction` exhaustively, and a new variant would break every one of them for a feature
none of them has asked for yet.

## Decision

Two more builder options. Without them nothing changes:

- `.movable(&mut |from, to| ...)`: a tab can be dragged along the strip. While it's dragged, a
  line marks the gap it would drop into, and on the drop the callback gets the tab and its new
  index, as `Vec::remove` then `Vec::insert` would place it. The app reorders its tabs. A drop
  where the tab already was calls nothing, and a click is still a click. A strip in a vertical
  layout drags up and down.
- `.launchers(&[bool])`: those tabs are their icon (or letter) alone whatever the label, at half
  strength, with no cross, and never filled as the shown tab. A click on one is still
  `TabAction::Show(i)`, so the app starts whatever it stands for.

## Consequences

- Existing callers are unchanged.
- An app that wants moving tabs keeps their order itself, as it already keeps the tabs.
- The drop gives an index, not an identity, so the app must apply it to the same list it drew
  from that frame.
- Revisit if most apps want moving tabs: then a `TabAction::Move` is worth the breaking change.
