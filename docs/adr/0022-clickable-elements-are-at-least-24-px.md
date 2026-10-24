# 0022. Clickable elements are at least 24 px, and menus' items 24 px high

Date: 2026-10-24

## Status

Accepted; the menu part superseded by [ADR-0023](0023-menus-are-egui-s-own-size.md),
which keeps menus egui's own size. The user found the Patina desktop's menus, then drawn as egui's context menus are,
too short, chose 24 px for an item, and asked that nothing clickable be smaller than 24×24;
text that's only read can be any size. `patina::menu` is my design.

## Context

egui's menus set the button padding to nothing, so an item is `interact_size.y` high: 18 px by
default, with 3 px between items. Files' context menus and the Patina desktop's (patina-menu) are
18 px an item, which is a small target for a pointer and smaller than menus on other desktops
(macOS about 22, GNOME about 30). egui's other widgets default to 18 px as well.

egui's `Response::context_menu` always applies its own `menu_style`; there's no global setting
for a menu's style. `Popup::context_menu(response).style(...)` takes another.

## Decision

- **A clickable element is at least 24×24 px**, a rule in `docs/design.md`. Text only read is
  exempt.
- **`patina::menu`**: `ITEM` (24.0); `style`, egui's `menu_style` with `interact_size.y` at
  `ITEM`, so buttons, checkboxes and radio buttons in a menu are 24 high; and
  `context(&response, add)`, a context menu in that style, for apps to use in place of
  `response.context_menu`.
- The Patina desktop's patina-menu draws its menus with `menu::style`.

## Consequences

- Apps' context menus change by moving to `menu::context`; Files first.
- Menus opened from a menu bar (`ui.menu_button`) and submenus still take egui's style. They'd
  need `MenuButton` with a `MenuConfig` in Patina's style.
- Widgets outside menus keep egui's 18 px default until each app or a skin gives them room; the
  rule says what they should be, it doesn't yet enforce it.

## Revisit when

- egui lets a menu's style be set for the whole context.
- 24 px proves too tall in dense menus, or too small on touch screens.
