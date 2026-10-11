# 0006. Title bar: word menus on the left, the app's output actions on the right

Date: 2026-10-11

## Status

Accepted: the user set this rule for every Patina app.

## Context

Patina's frame gives every app a header bar, with a slot on each side of the centred title
(ADR-0003). How apps fill them had varied:

- Photograph had a Render button on the left and the GPU status on the right.
- Notebook first copied the GTK version's GNOME-style buttons, then moved to word menus at the
  user's request.
- Calendar put Today and the view switch on the right.

The user stated the rule they want all of them to follow.

## Decision

- **Left: a standard window menu bar,** in words: File, Edit, View, the app's own menus, Skins.
  Not a hamburger. `TitleBar::menu_bar(true)` lays it out.
- **Right, next to the window buttons: the actions that serve the app's goal,** the main output
  the app exists to produce:
  - Photograph: Render, and Edit for the selected photo.
  - Notebook: the Markdown source switch and formatting.
  - Calendar: what we're doing today, so Today and the Day/Week/Month views.
- **Consumption apps** with no output to make, such as UnAmp, leave the right side empty. The
  rule is about productivity apps.
- **Everything else goes elsewhere:** settings, status and secondary information go in the menus,
  a sidebar or a status bar. Photograph's GPU status moved to a status bar at the bottom to free
  the right side for Render.

## Consequences

- Every Patina app's header reads the same way: what you can do on the left, what you're making
  on the right.
- When an app's right side gets crowded, that suggests something there isn't output and belongs
  in a menu or status bar.
- The skill for building Patina apps, and its template, follow this rule.
