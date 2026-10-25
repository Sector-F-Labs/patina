# 0028. The title bar's buttons are sized as Photograph's Render and Edit

Date: 2026-10-25

## Status

Accepted. The user asked for the style of Photograph's Render and Edit buttons to move into
Patina, as better than the bar's buttons in other apps. Photograph keeps Patina's menus
(ADR-0024).

## Context

The title bar's right side holds the app's output actions (ADR-0006): Photograph's Render and
Edit, Notebook's formatting buttons, Calendar's Today. Patina left them at egui's sizes: 4 px
either side of the text, 1 px above and below, 18 px high. Photograph set its own app-wide spacing,
10 by 5 px of padding, so its Render and Edit stood about 25 px high, filling the 25 px row, with
room around their names. Next to them other apps' buttons looked cramped, and at 18 px they were
under ADR-0022's 24 px minimum.

The left side of the bar holds the menus, which ADR-0024 styles, and tabs, which are sized from
the same `interact_size` and would grow if it changed there.

## Decision

- **`TitleBar` gives the buttons on its right 10 px of padding either side and 5 px above and
  below, and makes them at least 24 px high**, as Photograph's. They take their fill, rounding
  and colours from the skin, as before.
- **Only the right side.** The menus keep ADR-0024's style, and tabs on the left keep their
  height.

## Consequences

- Every app's output actions look like Photograph's, with no change in the app. An app that sized
  them itself can drop that, as long as it sized them only for the bar.
- The 25 px row still fits them; text larger than the Body style would make the row taller.
- Toggles and other widgets on the right take the padding too, since they size from the same
  spacing.
