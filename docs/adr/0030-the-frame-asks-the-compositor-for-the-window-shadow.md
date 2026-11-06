# 0030. The frame asks the compositor for the window's shadow: `frame::shadow::request`

Date: 2026-11-06

## Status

Accepted. The user wanted Patina apps to draw a shadow like Firefox does, chose to have the
compositor draw it when asked, and asked for the call to be in the frame code. The patina-desktop
compositor side is its ADR-0112; the prototype ran with Settings in its Linux VM.

## Context

A Patina app opens without system decorations and draws its own frame, so on Wayland it has no
shadow, and the desktop adds none unless it is told to: in GNOME Shell an app on this frame has
no shadow on any side, while GTK and Firefox windows have theirs.

Drawing the shadow in the window's own surface, as those do, needs `xdg_surface.set_window_geometry`
with the margin left out. winit sets it to the whole surface and has no API to change it
(rust-windowing/winit#3438), so a margin would count as window to the compositor and break tiling
and snapping. Asking the compositor to draw the shadow outside the surface avoids it. KDE's
`org_kde_kwin_shadow` protocol does that, and KWin and the Patina compositor draw it.

Reaching it from an app is the awkward part: it needs a request on the window's own `wl_surface`,
which belongs to winit's Wayland connection. egui's `Context` has no handle to it. eframe's `Frame`
does, through `raw-window-handle`. That is why the call can't happen inside `TitleBar::show` or
`edges`, which only have a `Context`.

## Decision

- **`frame::shadow::request(&frame)`**, given eframe's `Frame` (or anything with window and
  display handles), asks once per window. It makes a second event queue on winit's own
  `wl_display` (not a second connection, which couldn't name winit's surface), binds the shadow
  manager and `wl_shm`, and hands over eight generated tiles: the blurred edge of a rectangle
  the window's size, black, moved down a little. The tiles also reach 16 px in under the window,
  past its corner radius: the corners are transparent, and without shadow behind them each showed
  a notch of what lies below. `request_styled` takes a `Style` (blur 8,
  drop 3, opacity 0.5 by default: measured on a white background, the sides, bottom and top of
  Firefox's focused shadow are 61, 79 and 44 of 255 dark, and these give about the same).
- **It does nothing** off Linux, on X11, and where the compositor has no shadow protocol.
- **The frame's wiring is six places now:** an app calls `request` from `App::ui`, when it draws its own
  frame.
- **New dependencies,** on Linux: `wayland-client`, `wayland-backend` (`client_system`) and
  `wayland-protocols-plasma`; and `raw-window-handle` everywhere.

## Consequences

- Apps get a shadow on the Patina desktop and KWin by adding one line, which the module docs
  list. Forgetting it leaves the app flat; nothing warns.
- The call can't be folded into the frame's other helpers while they only take a `Context`.
  Moving it there would mean passing the `Frame` to them, which changes every app's call.
- The request is a roundtrip on winit's connection, once, the first time `ui` runs.
- patina-ui now has Wayland client dependencies, which apps already link through winit.
- GNOME and compositors without the protocol still show Patina apps flat. Drawing it in a
  margin there waits on the winit change.
- The tiles are generated at run time and the look is `Style`, not the skin. Putting it in the
  skin is a later change.
