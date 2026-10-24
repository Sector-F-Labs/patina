# 0023. Menus are egui's own size, as Teleprinter's

Date: 2026-10-24

## Status

Accepted. Supersedes the menu part of [ADR-0022](0022-clickable-elements-are-at-least-24-px.md).
The user found the 24 px items spaced too far apart and said Teleprinter's File, Edit and View
menus, egui's own, are what every menu should look like.

## Context

ADR-0022 made menus' items 24 px high. Teleprinter's menu bar opens its menus with
`ui.menu_button`, which keeps egui's `menu_style`: items `interact_size.y` (18 px) high, 3 px
apart. Next to those, the 24 px menus in Files and on the Patina desktop looked loose.

## Decision

- **Menus are egui's size:** `patina::menu::style` is egui's `menu_style`, unchanged; `ITEM` is
  gone. `menu::context` stays, so context menus and menus drawn elsewhere keep one style that
  Patina can change in one place.
- **The 24×24 rule doesn't cover a menu's items.** A menu item is as wide as the menu, so its
  target is large enough at 18 px. The rule still holds for other clickable elements.

## Consequences

- Files' context menus and the Patina desktop's menus match Teleprinter's menu bar again, with
  no change in the apps.
- Menu bars and context menus share a style, so there's nothing to set on `MenuButton`.

## Revisit when

- 18 px proves too small on a touch screen.
