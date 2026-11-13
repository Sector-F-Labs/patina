# 0036. Menus are data the app describes and the platform draws

Date: 2026-11-13

## Status

Proposed: the direction is the user's; the prototype in `patina::menu` is how we find out whether it holds.

## Context

Every app builds its menus as egui closures: `ui.menu_button("File", |ui| { ... })` inside the
title bar's left slot (ADR-0006). That is right on Linux, where Patina draws the whole frame. On
macOS it is not: a Mac app's menus live in the screen's menu bar, and an app that draws them in
its window looks foreign (the same reason the window buttons now have a macOS style).

egui menus are closures run every frame, so nothing can walk them to build an `NSMenu` from.
Native menus have to come from data. What the apps use today, found by reading them:

- items with shortcut text (`Ctrl+Shift+N`), shown but not handled by the menu;
- checkboxes and radio items (the Skins menu, "Gaps Between Panes");
- disabled items (`Larger Text` at the maximum);
- separators;
- submenus (Processes' View → Update Speed), and a submenu whose items load lazily (Files'
  "Open With", in a context menu).

`muda` (Tauri's menu crate) builds native menus on macOS, Windows and GTK. It has submenus to any
depth, check items, icons, accelerators, separators and section headers, a context-menu popup,
and the macOS Window and Help menus. It has no radio item, which macOS lacks too (a radio is a
checkmark there). winit has no menu API, so `muda` sits beside eframe. Considered and ruled out:

- **Each app writes `#[cfg(target_os = "macos")]` itself.** Every app repeats the work, and
  the two copies of each menu drift.
- **Converting egui menus automatically.** Nothing to convert from; they are closures.
- **Predefined native items** (`PredefinedMenuItem::copy`, `quit`). They send selectors to the
  responder chain, which egui's text fields are not in, and `quit` ends the process before the
  app can ask about unsaved work.

## Decision

- **`patina::menu` gets a model:** `Menus<A>` of `Menu`s of `Entry`s: `Item`, `Separator` and
  `Submenu`, nested to any depth. `A` is the app's own action type. An item has a label, an
  action, an optional `egui::KeyboardShortcut`, `enabled`, and a `Mark`: none, check or radio.
  Whatever macOS menus can show, the model can say; the egui drawing supports the same set.
- **One call draws it:** `menus.show(ui) -> Option<A>`. Which backend draws is decided inside:
  `EguiMenus` (today's look, inside the title bar's bar) is the default and the only one off
  macOS; `NativeMenus` (`muda`, `#[cfg(target_os = "macos")]`) is used on macOS unless the app
  turns it off with `menu::set_native(ctx, false)`. Both return the clicked item's action, so
  apps handle one value type.
- **The model owns the shortcuts.** Egui consumes them when it draws the menus; the native
  backend gives them to the system as key equivalents. Apps stop handling them separately.
- **Roles.** `Role::About`, `Settings` and `Quit` on an item mean "this belongs in the app
  menu" on macOS (the native backend moves them there and builds the rest of that menu: Services,
  Hide, Hide Others, Show All); elsewhere they stay where the app put them. They always deliver
  the item's action; none is a predefined native item.
- **The native menu is rebuilt when the model changes** (compared each frame by what it shows),
  and `muda`'s events wake egui (`request_repaint`).
- **Not in the model:** widgets that aren't items, such as a slider or colour picker in a popup.
  An app keeps `menu_button` for those, in the window, on every platform.
- **Context menus** keep `menu::context`; the same `Entry`s will be shown through `muda`'s popup
  later, which the lazily loaded "Open With" needs worked out first.
- `menu_button` stays working; apps move to the model one at a time.

## Consequences

- Apps describe a menu once and get the platform's own on macOS. Shortcuts, submenus and
  checkmarks work the same either side.
- The model is a subset of egui's freedom: no arbitrary widgets in menus. Apps that used them
  have to split the menu.
- Dynamic state (checks, disabled, lists that change) means building the model every frame, and
  the native backend comparing it each time. Cheap for menus of this size; to be measured.
- A second backend to keep working, and one that only builds on macOS: Linux CI can't catch its
  errors, so it is tried on the Mac.
- Menu text that is long or that wraps, hover tooltips and icons in egui menus have no
  native counterpart yet.
- Revisit if `muda`'s accelerators and winit's key handling double-fire a shortcut, or if
  rebuilding the `NSMenu` while it is open causes trouble.
