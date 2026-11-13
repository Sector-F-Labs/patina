# 0035. Every button is a title bar button's size by default: `Skin::apply` sets the padding

Date: 2026-11-13

## Status

Accepted. The user saw the gallery's new text input beside a row of dialog buttons, found those
button sizes good, and asked for buttons to be upgraded too. It is the follow-up
[ADR-0032](0032-dialog-buttons-are-the-title-bars-size.md) left open.

## Context

ADR-0032 gave a row of buttons the title bar's size (10 by 5 px of padding, 24 px high) when an
app calls `dialog::buttons(ui)`, and left egui's 18 px buttons (4 by 1 px of padding) everywhere
else, because putting it in `Skin::apply` "would change every widget in every app at once": the
tabs and the markdown editor's cells size themselves from `interact_size`, and dense panels would
reflow. Inputs have since followed buttons (ADR-0034), so a plain button beside one is now the
odd one out, in the gallery and in every app with a toolbar of egui buttons next to a field.

What an app's buttons are made of matters: egui's `Button`, `selectable_label`, `ComboBox` and
`DragValue` take their padding from `style.spacing.button_padding` and a least height from
`interact_size.y`. The tabs, the menus and the title bar set their own, and the sliders and check
boxes use `interact_size` and `icon_width`, not the padding.

## Decision

- **`Skin::apply` sets `spacing.button_padding`** to `[controls] button_padding`, held to 0 to
  24 px, else `dialog::BUTTON_PADDING` (10 by 5): the title bar's. `Skin::spacing()` gives it. The
  skin without a base, which follows the desktop, gets it too.
- **`interact_size` is left alone**, so check boxes, sliders and the rest keep their size; a
  button's height is its text and its padding, about 24 px with the usual font. `dialog::buttons`
  still gives a row the least height and the 8 px gap.
- **A skin that wants egui's size back** writes `button_padding = [4, 1]`.

## Consequences

- Every plain egui button, drop-down, drag value and selectable label in a Patina app is about 6
  px taller and 12 wider than before, with no change in the app. Toolbars of them, and dense
  panels, take more room.
- Widgets that draw their own size (the tabs, menus, the title bar's buttons, Files' toolbar
  glyph buttons) don't change.
- An app that wants its old buttons somewhere sets `ui.spacing_mut().button_padding` in a scope;
  `dialog::buttons` is no longer needed for the padding, only for the gap and the least height.
