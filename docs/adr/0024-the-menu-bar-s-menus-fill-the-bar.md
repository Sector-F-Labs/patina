# 0024. The menu bar's menus fill the bar, and light as the shown tab

Date: 2026-10-24

## Status

Accepted. The user asked for a larger target on the title bar's menus, one you can see under the
pointer, lit in the same colour as the shown tab. Filling the bar's full height is my reading of
that, to match the tabs beside them.

## Context

The title bar's menus (File, Edit, View, ADR-0006) were egui's menu-bar buttons: as high as the
text, with 2 px padding either side and 8 px gaps between them, and lit in egui's small rounded
hover fill. The tabs after them (ADR-0009) fill the bar's whole height and are square, the shown
one in the panel colour, or the extreme background where the bar is already the panel colour.
Next to the tabs the menus were small targets, and the gaps between them were dead space.

egui's `MenuBar` takes a style for its buttons, so the title bar can restyle the `menu_button`s
apps already put in it. The menus inside the popups stay egui's (ADR-0023).

## Decision

- **`TitleBar::span(range)`**, the bar's full height as `TabStrip::span` takes it. With a menu
  bar, the left side spans it, and each menu is that tall. Without `span` they fill the 25 px
  row.
- **The menus have 10 px either side of their names and no gap between them**, so the whole bar
  above and between the names opens a menu.
- **Under the pointer or pressed, a menu is filled with the shown tab's colour**
  (`frame::lit`, shared with the tabs) with the strong text colour, square, so the target is
  what you see.
- On a bar the skin colours, a menu's name keeps the bar's title colour at rest and takes the
  strong text colour when lit, where the bar's title colour might not read.
- The tabs after the menus keep their own hover colours: the title bar keeps the bar's widget
  colours aside for the tab strip, since the menu bar's style covers its whole slot.

## Consequences

- Apps with tabs already have the span and add `.span(span)` to their `TitleBar`. Apps without
  tabs get menus the row's height until they pass it too.
- The left slot is laid out `span` high, so anything else an app puts there is centred on the
  bar as before.
- A menu is lit only while hovered or pressed; once open, egui doesn't light its button, so
  moving into the menu leaves its name unlit. Lighting the open one needs the button's id, which
  only the app has from `menu_button`'s response.

## Revisit when

- egui lights a menu bar's open menu, or lets the bar find its menus' buttons.
