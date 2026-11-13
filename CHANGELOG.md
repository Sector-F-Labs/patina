# Changelog

What changed in each version of `patina-ui`, newest first. **Upgrading** lists what an app has to
change, or should, to take the version; an app that skips it still builds unless it says
otherwise. The other headings say what's new or different. Why each change was made is in the ADR
it names, in [docs/adr](docs/adr/README.md).

Sector F apps build on `develop`, so they get changes before a release: after
`cargo update -p patina-ui`, read **Unreleased** as well. Each change that alters what apps write
is added there in the same commit (ADR-0025).

## Unreleased

## 0.1.18 — 2026-11-13

### Added

- `Menus::app_name("Photograph")`: the app's name as macOS's menu bar shows it, and the title of
  the app menu. A packaged app gets its name from `Info.plist` (`CFBundleName`); a binary run on its
  own (`cargo run`) showed its file name in lower case. Sets the process name; nothing happens
  off macOS. Optional.

## 0.1.17 — 2026-11-13

### Changed

- A maximised window keeps its rounded corners, border and the window shadow's shape, as on macOS
  and Windows. Squaring them as GNOME does is opt-in now: `frame::set_square_when_maximised(ctx, true)`.
  **Upgrading:** an app that wants the old look on Linux calls it once (e.g. in its first frame).

### Added

- `patina::menu::Menus<A>` (ADR-0036, Proposed): an app's menus as data — `Entry::item` with
  `.shortcut`, `.enabled`, `.check`, `.radio` and `.role`, `Entry::separator`, and
  `Entry::submenu` to any depth — and `menus.show(ui) -> Option<A>`, which draws them as egui
  menus in the title bar's left slot and, on macOS, in the screen's menu bar (through `muda`,
  built only there; `menu::set_native(ctx, false)` keeps them in the window). Shortcuts are the
  model's: it handles them on both. Items with a `Role` (About, Settings, Quit) go to the app
  menu on macOS. `ui.menu_button` still works; apps move over one at a time. Nothing to change in
  an app. The gallery's menus use it.
- `TitleBar::window_buttons(WindowButtons)`: `WindowButtons::MacOs` draws red, yellow and green
  traffic lights at the left, `Patina` the existing buttons at the right. The default is
  `WindowButtons::platform()`: macOS's on macOS, Patina's elsewhere. Nothing to change in an app.

## 0.1.16 — 2026-11-13

### Fixed

- Hovering a button, selectable label or other control no longer moves it by its border's width:
  `Skin::visuals` and `Skin::apply` now apply `skin::steady_visuals` (what `steady_controls` did
  for the tree and the title bar) to every control. Nothing to change in an app.

### Added

- `patina::emoji` (cargo feature `emoji`): colour emoji from a bundled Noto Color Emoji (SIL OFL),
  painted as images because egui draws no colour glyphs. `emoji::label(ui, text)` shows text with
  its emoji in colour; `emoji::runs` splits text into plain and emoji runs, and `glyph`, `texture` and
  `paint` are the pieces, for a widget that lays text out itself. It is Teleprinter's code (its
  ADR-0013), shared. The font is not in `patina-ui`, because at 10 MB it takes the package over
  crates.io's size limit: apps give it one with `emoji::install` or `install_file`
  (Sector F's apps use the `patina-emoji-font` git crate in this repository; see Upgrading). The feature adds about 11 MB to the binary (ADR-0031).
- `patina::dialog`: a row of buttons the size of the title bar's (ADR-0032). `dialog::buttons(ui)`
  gives the buttons in a `ui` 10 by 5 px of padding, at least 24 px high, 8 px apart, as the bar's
  (ADR-0028); `dialog::primary(ui, text)` is the button Enter presses, in the skin's accent.
  The constants are `BUTTON_PADDING`, `BUTTON_HEIGHT` and `BUTTON_GAP`. The title bar's buttons are
  unchanged.
- `dialog::text_edit(ui, &mut text)`: a single-line input padded as a button is (10 by 5 px) and
  at least 24 px high, since egui's `TextEdit` takes its 4 by 2 px margin from no style (ADR-0034).
  The size is `dialog::input_size(ctx)`, which a skin sets with `[controls] input_padding` and
  `input_height` (`Skin::apply` does it), and an app with `dialog::set_input_size`. `INPUT_PADDING`,
  `INPUT_HEIGHT` and `InputSize` are the pieces. The folder tree's filter box uses it, so it is 24
  px high now, up from 22, and the text starts 10 px in, up from 4.
- The markdown `Editor` draws emoji in colour when both `markdown` and `emoji` are enabled: they
  are laid out invisibly, so cursor, selection and wrapping work as before, and the colour image is
  painted over them.

### Changed

- Every button is a title bar button's size by default (ADR-0035): `Skin::apply` sets egui's
  `spacing.button_padding` to 10 by 5 px, or to the skin's `[controls] button_padding` (`[4, 1]` is
  egui's old size), so plain buttons, drop-downs, drag values and selectable labels are about 24 px
  high, as dialog buttons and inputs are, with no change in the app. `Skin::spacing()` is the
  value. Tabs, menus and the title bar, which size themselves, don't change. The folder tree's rows are
  `tree::ROW` (24 px) apart whatever the buttons' size, as a list's are.
- Markdown tables look and lay out like HTML's (ADR-0033): joined cells with borders, text that
  wraps, as wide as the content wants up to the text's width, and images in cells (`![alt](src)`,
  `<img>`, inside a link too) shown. `markdown::embeds::table` takes the document's folder before
  the id; the `Editor` does this itself.

### Upgrading

- **Buttons are bigger by default: look at toolbars and dense panels.** Plain egui buttons are
  about 24 px high now, not 18, and 12 px wider; a row of them takes more room. To keep a place as
  it was, set `ui.spacing_mut().button_padding = egui::vec2(4.0, 1.0)` in a scope there, or have
  the skin say `[controls] button_padding = [4, 1]`. `dialog::buttons` is still how a row gets
  the 8 px gap and the least height. **An app's own list or tree made of `selectable_label`s grows with
  the buttons**: set its own `button_padding` in the list's scope, as `tree::Tree` does.

- **Apps with dialogs, or any row of buttons outside the title bar: call `patina::dialog::buttons(ui)`
  first inside the row's layout, and make the button Enter presses `patina::dialog::primary`.**
  egui's buttons are 18 px high and look small next to the bar's. Right-align the row, the primary
  button rightmost (`Layout::right_to_left`, adding it first). An app with its own copy of the
  bar's button spacing can delete it. Nothing changes until you call it.
- **Apps that show text with emoji in it: enable the `emoji` feature, install a font and show that
  text with `patina::emoji::label`.** The font isn't in `patina-ui` (10 MB is over crates.io's size
  limit) and Patina doesn't look for one: once at start-up, give it the bytes of a Noto Color
  Emoji in its CBDT build (PNG images), or a path:

  ```rust
  patina::emoji::install(font_bytes);   // &'static [u8]
  patina::emoji::install_file(&path);   // a file you ship or the user picks
  ```

  Each says `false` if the font isn't usable; emoji then stay as egui draws them. Sector F's apps
  use our copy, a git dependency (crates.io refuses those, so not for a published app):

  ```toml
  patina-emoji-font = { git = "https://github.com/Sector-F-Labs/patina", branch = "develop" }
  ```

  and `patina::emoji::install(patina_emoji_font::FONT);`. egui's own labels still draw emoji as
  monochrome shapes. Nothing else changes.

## 0.1.15 — 2026-11-06

### Added

- `frame::shadow::request(&frame)` asks the compositor to draw a drop shadow around the window,
  through KDE's `org_kde_kwin_shadow` protocol (the Patina compositor and KWin draw it; others
  show nothing). `request_styled` takes a `shadow::Style` (ADR-0030).
- `shadow::Style` also has `spread`, `offset_x` and `color` (a grown or shrunk shape, a sideways move and
  a tint), with defaults that look as before; build one with `..Style::default()`.
- `frame::shadow::tiles(style)` is public: the eight tiles and offsets a client with Wayland objects of
  its own attaches to a surface (patina-shell does, for the HUD's layer surfaces).

### Upgrading

- **Apps that draw their own frame: call `patina::frame::shadow::request(frame)` from
  `App::ui`,** with eframe's `Frame`, when the frame is the app's own (ADR-0030). Without it the
  window has no shadow; nothing else changes.

### Fixed

- A local image in the markdown editor shows its file again when another program changes it (checked
  twice a second while shown), instead of staying as first loaded until the app restarts.

## 0.1.14 — 2026-11-04

### Fixed

- Hovering a row of the folder tree no longer shifts it and its neighbours by a pixel with skins
  that stroke their controls, and the borders stay: a control is one size idle, hovered or pressed
  whatever its border. New `skin::steady_controls(ui)` does the same for an app's own lists
  of `selectable_label`s, such as a sidebar.

### Added

- Markdown shows SVG images (with their text), and an `<img src=".." alt="..">` tag alone on its line as an image,
  like `![alt](src)`; the tag is written back as it was until its description is edited.
  `markdown::model::Image` has a new `html` field: an app that builds one sets it to `None`.
- `FolderTree::filter_box(ui)`: a filter box for above the tree; typing replaces the tree with a flat
  list of the names that match, fuzzily (case, spaces and dashes don't matter), each with its folder, looking in folders not yet listed. Ctrl+P and Ctrl+N (or Up and Down)
  move through the matches, Enter opens the one marked (`show` returns it). `set_filter`, `filter`.
  Nothing changes for an app that doesn't call it.
- `markdown::Editor::set_image_room(room)`: images wider than the text may reach `room` points past
  each side of it, centred. Off unless an app calls it.
- `FolderTree::lister(..)`: the tree lists folders from a function you give it instead of this
  machine's disk, for a project on another machine (`tree::Lister`, `tree::Entry`; Programmer's
  ADR-0030). Nothing changes for an app that doesn't call it.

## 0.1.13 — 2026-10-28

### Added

- `FolderTree::files(true)`: the tree lists each folder's files after its subfolders, and `show`
  returns a clicked file as well as a clicked folder (ADR-0029).
- **Everforest Dark**, a built-in skin in the Everforest palette.

## 0.1.12 — 2026-10-25

### Upgrading

- **Source drawn on `code_bg_color` by the app itself: use `extreme_bg_color`,** to match code
  blocks. Only the Default skin looks different; skins set both to `surface` (ADR-0026).
- **Apps with a highlighter of their own: use `patina::highlight`** (feature `highlight`, on with
  `markdown`), so code looks the same everywhere. Apps calling `egui_extras::syntax_highlighting`
  need `egui_extras`' `syntect` feature themselves now; Patina no longer turns it on (ADR-0027).

### Added

- `patina::highlight`, behind the `highlight` feature: Text Editor's highlighter, a line at a time
  and kept between edits, over bat's syntaxes (`Highlighter`, `layout_job`, `detect`,
  `language_for`, `languages`) (ADR-0027).

### Changed

- The title bar's right-hand buttons are sized as Photograph's Render and Edit: 10 by 5 px of
  padding and at least 24 px high, where they were egui's 4 by 1 and 18 high (ADR-0028).
- Code blocks (`markdown::embeds::code`) are filled with `extreme_bg_color`, the text-field
  background, not `code_bg_color`: white or near-black in Default instead of grey (ADR-0026).
- Code blocks and `embeds::code_file` are coloured by `patina::highlight`: bat's languages (TOML,
  TypeScript, Dockerfile...), plain text in the skin's text colour (ADR-0027).
- `make test` and `make check` build every feature.

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
