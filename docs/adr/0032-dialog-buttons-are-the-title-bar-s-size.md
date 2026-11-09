# 0032. Dialog buttons are the title bar's size: `patina::dialog`

Date: 2026-11-09

## Status

Accepted. The user found the buttons in the Patina desktop's dialogs (the polkit password dialog,
Files' confirmations, the power menu's question) too small and not like a desktop's, showed the
title bar's buttons as the size they should be, and said this is a rule for Patina apps: put it
in `develop`.

## Context

ADR-0022 made 24 px the least a clickable thing is, and ADR-0028 sized the title bar's buttons as
Photograph's: 10 px of padding either side of the text, 5 above and below, 24 px high. Both are
applied only where Patina draws, inside `TitleBar`. Outside it, a dialog's buttons are egui's: 18 px
high with 4 by 1 px of padding, so a Cancel in a dialog is a smaller target than a Save in the
title bar of the same app, and looks like a toolkit's default, not a desktop's. Each app that
noticed fixed it with its own copy of the same four lines (three in the desktop alone).

Putting it in `Skin::apply`, the style every widget draws from, would change every widget in every
app at once: the tabs and the markdown editor's cells size themselves from `interact_size` too
(ADR-0028 left the left side of the bar alone for that reason), and apps with dense panels would
reflow without being asked.

## Decision

- **`patina::dialog::buttons(ui)`** sizes the buttons of the `ui` it's called in as the title
  bar's: `BUTTON_PADDING` (10 by 5), at least `BUTTON_HEIGHT` (24) high, `BUTTON_GAP` (8) apart.
  It is called first inside a row's layout, so only the row changes. The title bar's own buttons
  use the same constants.
- **`patina::dialog::primary(ui, text)`** is the button Enter presses, filled in the skin's accent
  (its selection colours); the others are plain.
- **A row of buttons in a dialog, panel or popup is right-aligned, primary rightmost**, as other
  desktops have it. The rule is in `docs/design.md` and the `patina-app` skill.
- It's a call apps make, not a change to the global style.

## Consequences

- Apps' dialogs match their title bars with two lines, and can drop their own copies.
- A button outside a row that calls `buttons` is egui's size still; an app that wants all its
  buttons that size sets the same spacing on its style itself. That's a possible follow-up once
  the apps' dense panels have been looked at.
- `buttons` raises the `ui`'s `interact_size.y` for everything after it, so it belongs in the
  row's layout or a scope of its own.
