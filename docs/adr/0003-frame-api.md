# 0003. The frame's API: a TitleBar builder with left and right slots, styled by FrameStyle

Date: 2026-10-11

## Status

Proposed. The user asked for the shared frame. The API below is my design.

## Context

The two copies of the frame had different signatures:

- **Photograph:** `title_bar(ui, title, custom, left, right)`, painted straight from egui's
  visuals.
- **UnAmp:** `title_bar(ui, style, title, custom, menus)`. It has a `FrameStyle` resolved from
  its skin, and wraps its controls in a `MenuBar`. It lays the row out in separate child areas,
  because a menu bar takes its whole width and pushed the panel wider than the window.

Each app also repeated the same setup around the bar:

- the viewport flags;
- the header panel's padding and corner rounding;
- a transparent clear colour;
- the edges call;
- the input hook that returns the button release the compositor keeps.

## Decision

- **`TitleBar::new(title, &style)`**, with `.custom(bool)` (default on) and `.menu_bar(bool)`
  (default off), finished by `.show(ui, left, right)`. Both slots are always there, so
  UnAmp passes an empty right slot.
- **Layout:** each part is laid out in its own child area, in this order:
  - the window buttons, at the right;
  - `right`, right to left, 8 px from the buttons;
  - `left`, in the width that's left, measured from inside so a menu bar doesn't count as full
    width.

  The title is centred on the window and truncated. This is UnAmp's overflow fix, extended to
  the right slot.
- **`FrameStyle`** holds every colour the frame paints, the window controls apart from the
  app's buttons. `FrameStyle::from_visuals` is the unskinned look, with the fallbacks from
  UnAmp's skin code.
  - This means unskinned Photograph's window-button icons take the widget foreground colours
    rather than the text colours. The difference is slight.
- **Helpers for the setup:** `viewport`, `header_frame` (12×8 px padding, top corners rounded),
  `clear_color`, `corner_radius`, `edges` and `release_grab`. The module docs list the five
  places an app wires them up.
- **`WINDOW_RADIUS` = 15**, libadwaita's, as both apps used.

## Consequences

- Apps can't easily drift from the header-bar padding and corners, because the helpers set them.
- A builder can grow options, such as hiding maximise for a fixed-size window, without breaking
  callers.
- `FrameStyle` will be filled by the skin loader once that moves here. Until then each app
  builds its own, and Photograph uses `from_visuals`.
- Both apps change their call sites when they switch.
- Apps still own their panels' bottom-corner rounding, which depends on their layout. A
  helper for that waits until a pattern shows up.
