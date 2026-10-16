# 0016. A status bar along the bottom: the version, then colour-coded messages, then the app's own status on the right

Date: 2026-10-16

## Status

Accepted. The user asked for it, after Reader's skin menu showed what Copy built-in skins did as
a note under its buttons: "status updates in a menu are weird, would make more sense to have a
status bar at the bottom of the window and push notifications there".

## Context

Photograph and UnAmp already had a bar along the bottom, built separately. Photograph's held its
version on the left and its GPU on the right. UnAmp's held the window size and errors. The other
apps reported things wherever they happened: skin menus listed skin load errors and copy results
under their buttons, where they only showed while the menu was open and made the menu as wide as
the message. Nothing said where a message should go or what colour it should be.

Reader was used to settle the layout first. It labelled the version "Reader v0.1.0"; the user
dropped the name as redundant, since the title bar and the window list already show it.

## Decision

Every Patina app has a status bar along the bottom of the window:

- **It spans the window and holds both bottom corners** (see the frame's corners in
  [design.md](../design.md)), so it's added before any side panel. Its frame is
  `Frame::side_top_panel` with 10 × 4 px margins.
- **Left: the version,** `v0.1.0`, small, in the skin's weak text colour. No app name.
- **Next to it, after a separator: the latest message,** small, coloured by its level: red
  (`error_fg_color`) for errors, yellow (`warn_fg_color`) for warnings, and the weak text colour
  for anything else. A message stays until the next replaces it. Problems that persist, such as
  skins that failed to load, stay until they're fixed, and list their details on hover.
- **Right: the app's own standing information,** such as Photograph's GPU. Apps with none leave
  it empty.
- **Menus don't report.** What an action in a menu did, or why it failed, goes to the bar.

## Consequences

- Messages are in one place in every app, readable after the menu that caused them closes, and
  the menus stay the size of their entries.
- Each app gives up a row of height to the bar, even when there's nothing to say.
- One message at a time: a second replaces the first, so the bar is no log. If an app needs a
  history, that's a new decision.
- Each app builds the bar itself for now. If the copies drift, it should move into Patina as a
  component, as the frame did.
