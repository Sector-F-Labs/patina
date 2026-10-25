# Changelog

What changed in each version of `patina-ui`, newest first. **Upgrading** lists what an app has to
change, or should, to take the version; an app that skips it still builds unless it says
otherwise. The other headings say what's new or different. Why each change was made is in the ADR
it names, in [docs/adr](docs/adr/README.md).

Sector F apps build on `develop`, so they get changes before a release: after
`cargo update -p patina-ui`, read **Unreleased** as well. Each change that alters what apps write
is added there in the same commit (ADR-0025).

## Unreleased

### Upgrading

- **Source drawn on `code_bg_color` by the app itself: use `extreme_bg_color`,** to match code
  blocks. Only the Default skin looks different; skins set both to `surface` (ADR-0026).

### Changed

- Code blocks (`markdown::embeds::code`) are filled with `extreme_bg_color`, the text-field
  background, not `code_bg_color`: white or near-black in Default instead of grey (ADR-0026).

## 0.1.11 — 2026-10-24

### Upgrading

- **`TitleBar` with `.menu_bar(true)`: add `.span(frame::header_span(ui))`,** called inside the
  header panel, so the menus fill the bar's height. Apps with a `TabStrip` already compute the
  span; pass the same one to both. Without it the menus fill only the 25 px row (ADR-0024).
- **Drop your own Ctrl+T and Ctrl+W.** `tabs::keys` now returns `TabAction::New` for Ctrl+T and
  `TabAction::Close(shown)` for Ctrl+W, and no longer takes Ctrl+Shift+T. A terminal, whose Ctrl+T
  and Ctrl+W belong to the shell, calls `tabs::terminal_keys` instead, which takes them with Shift
  and otherwise matches `keys` (ADR-0020).
- **Context menus: use `patina::menu::context(&response, |ui| ...)`** in place of
  `response.context_menu(...)`, so every menu shares one style Patina can change in one place
  (ADR-0022, ADR-0023).

### Added

- `TitleBar::span(range)` and `frame::header_span(ui)`: the header bar's full height, for its menus
  and tabs (ADR-0024).
- `TitleBar::minimise(false)` leaves the minimise button out, for a desktop with nowhere to
  minimise to.
- `TabStrip::movable(&mut |from, to| ...)`: tabs can be dragged into another order; the app
  reorders its own list (ADR-0021).
- `TabStrip::launchers(&[bool])`: those tabs show only their icon, at half strength, with no
  close button, and are never filled as the shown tab (ADR-0021).
- `tabs::terminal_keys` (ADR-0020).
- `patina::menu`: `style`, egui's menu style, and `context` (ADR-0022, ADR-0023).

### Changed

- The menu bar's menus fill the bar's height, have 10 px either side of their names and no gaps
  between them, and light square in the bar's hover colour. The first menu's name stays where it
  was, and the divider before the tabs stays the row's height (ADR-0024).
- `TabStrip`'s `+` hint says Ctrl+T (ADR-0020).
- Menus are egui's own size. 0.1.11 briefly made menu items 24 px high (ADR-0022); that was
  reversed before the release (ADR-0023). The 24×24 minimum stays for other clickable elements.

### Fixed

- A short panel at a window corner, such as the status bar, keeps the window's full corner radius,
  so its background no longer shows outside the border's curve.

## 0.1.10 — 2026-10-19

### Upgrading

- **Markdown: enable the `markdown` feature** to use `patina::markdown`, and call
  `patina::markdown::install(ctx)` once at start-up for its fonts and image loaders
  (ADR-0015).
- **Touchpad scrolling: call `patina::kinetic::glide(ctx, raw_input)` from
  `App::raw_input_hook`,** before anything else reads the scroll events, so a flick keeps
  scrolling after the fingers lift (ADR-0019).
- **Every app has a status bar** along the bottom: the version, then the latest message coloured
  by level, then the app's own status. Menus don't report; their results go to the bar. Apps build
  it themselves for now (ADR-0016).
- **The skin menu lists every skin, then Follow desktop / Light / Dark, then Reload skins, Copy
  built-in skins to folder and Open skins folder,** the same in every app, with no messages in it
  (ADR-0017).

### Added

- `patina::markdown`, behind the `markdown` feature: Notebook's markdown `Editor`, with a
  read-only mode, find (`set_search`, `find`) and the model, embeds and fonts it's built on.
  `embeds::code_file` and `embeds::mark` show a file's code with marked lines (ADR-0015).
- `patina::kinetic::glide` (ADR-0019).
- `TabStrip::icons` and `TabStrip::label(TabLabel)`: tabs with icons, titles or both (ADR-0018).
- `TabStrip::tab_menu`: an app's own items in each tab's menu, before Close.
- `TabStrip::own_menu`: the app opens a tab's menu itself, where it has no room for one.
- `TabStrip::own_tooltip`: the app shows a hovered tab's title itself.
- `TabStrip::separator(false)`: tabs that stand alone, without the line before them.

### Changed

- `TabStrip::show` takes `mut self`; callers are unchanged.

### Fixed

- A tab's icon is drawn on whole pixels, so it isn't blurred.

## 0.1.9 — 2026-10-16

### Added

- `TabStrip::closable(false)`: tabs that can't be closed (ADR-0014).

## 0.1.8 — 2026-10-15

### Added

- `TabStrip::new_tab(false)` leaves out the `+` tab and the menu's New item, and
  `TabStrip::noun("Window")` makes the menu and close button say Close Window (ADR-0012).
- `tabs::keys`: Alt+1 to Alt+9 show that tab (ADR-0013).

## 0.1.7 — 2026-10-14

### Upgrading

- **Load skins from `skin::app_dir("<app>")`,** `~/.config/<app>/skins`, named after the app's
  config folder, and point the skin menu's Copy and Open at it. `skin::shared_dir` is deprecated
  and apps no longer read `~/.config/patina/skins/`; skins there have to be moved (ADR-0011).

### Added

- `skin::app_dir` (ADR-0011).

## 0.1.6 — 2026-10-13

### Added

- `patina::open::path(path)`: opens a file or folder with the desktop's opener, not the browser
  (ADR-0010).

## 0.1.5 — 2026-10-13

### Added

- `patina::tabs`: `TabStrip`, tabs in the title bar moved from Teleprinter, with `span`,
  `id_salt`, `new_hint` and `tab_id`; `TabAction` (`New`, `Show`, `Close`); and `tabs::keys`
  for Ctrl+PageUp/PageDown and Ctrl+Shift+T (ADR-0009).

## 0.1.4 — 2026-10-13

### Added

- `frame::can_minimise_and_maximise()`, for apps that offer minimise or maximise elsewhere
  (ADR-0008).

### Changed

- Where the compositor can't minimise or maximise (dwl for now), the header bar shows only close,
  and the window menu and double-click leave them out (ADR-0008).
- Adds `libc` as a Linux-only dependency.

## 0.1.3 — 2026-10-12

### Upgrading

- **Apps with text fields: call `patina::ime::drop_repeated_preedits(ctx, raw_input)` from
  `App::raw_input_hook`.** Without it, on GNOME's Wayland, a focused text field keeps an idle
  window repainting at a full core. See [docs/egui-pitfalls.md](docs/egui-pitfalls.md).

### Added

- `patina::ime::drop_repeated_preedits`.
- [docs/egui-pitfalls.md](docs/egui-pitfalls.md): finding what keeps an egui app repainting.

## 0.1.2 — 2026-10-11

### Added

- Built-in Calendar, Notebook and Steam Classic skins in `skin::BUILT_IN` (ADR-0005).

### Changed

- `TitleBar` recolours its controls when the skin's title bar differs from the panel, and paints
  the title in the skin's `title_text`. Apps that did this themselves (Calendar, Notebook) can
  drop it (ADR-0005).

## 0.1.1 — 2026-10-11

### Fixed

- Title bar controls no longer shift when one is hovered under a skin that strokes controls
  (Dark Orange).

## 0.1.0 — 2026-10-11

The first release: the frame, folder tree and design language of Photograph and UnAmp.

- `patina::frame`: the app's own window frame, with `TitleBar`, `FrameStyle`, GNOME-style window
  controls, resize edges and rounded corners (ADR-0003).
- `patina::appearance`: follows the desktop's light/dark setting.
- `patina::skin`: skins in the suite format, with app sections and Dark Orange built in
  (ADR-0004).
- `patina::tree`: a sidebar folder tree that lists folders on worker threads.
- Re-exports `egui` 0.35; apps use the same version.
