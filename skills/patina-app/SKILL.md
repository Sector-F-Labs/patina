---
name: patina-app
description: Build a native egui desktop app on Patina (the patina-ui crate), or change Patina itself. Use when asked to set up Patina, make or port a Patina app, or work on an app's title bar, skins, frame, Makefile, ADRs or screenshots.
---

# Building apps on Patina

Patina is a window frame, a skin format and a few components for egui desktop apps. Apps look
as different as their skins make them and behave alike. The crate is `patina-ui`, imported as
`patina`. Repository: <https://github.com/Sector-F-Labs/patina>.

## Get set up

1. Check the tools: `rustc --version` (edition 2024, so Rust 1.85 or newer), and on Linux a C
   compiler plus the Wayland/X11 development headers eframe needs. `make` is used for the
   targets below.
2. Clone Patina next to the app, to read it: `git clone https://github.com/Sector-F-Labs/patina`.
   Its source is the reference. Read these before designing anything visual:
   - `docs/design.md`, the design language;
   - `docs/skinning.md`, the skin format and the built-in skins;
   - `examples/gallery.rs`, a complete app with every component and a Skin menu. Run it with
     `make gallery`;
   - `docs/egui-pitfalls.md` and `docs/known-issues.md`, before debugging a window that burns a
     core, freezes or won't take focus.
3. To have this skill in a new app's project, copy `skills/patina-app` from the clone into
   wherever your agent keeps skills, in the project or for every project.

Other apps built on Patina, to learn from: [Photograph](https://github.com/divanvisagie/Photograph),
[UnAmp](https://github.com/divanvisagie/UnAmp), [Teleprinter](https://github.com/Sector-F-Labs/teleprinter)
and [md-chat](https://github.com/Sector-F-Labs/md-chat).

## What Patina gives an app

- `frame`: the window frame, title bar, window buttons, resize edges and rounded corners.
- `skin`: skins in one format, built-ins, and a skins folder per app (`skin::app_dir`).
- `appearance`: follows the desktop's light/dark setting.
- `tabs`: tabs in the title bar.
- `tree`: a folder sidebar that never blocks the UI (`FolderTree`).
- `markdown` (cargo feature): a markdown editor and viewer, with highlighted code and images.
- `highlight` (cargo feature): syntax highlighting.
- `open`: open a file or folder in the desktop's app for it.
- `ime`: `drop_repeated_preedits`, for `App::raw_input_hook`.

Prefer these to writing your own. When a second app wants a component, it moves into Patina
rather than being copied.

## Make a new app

Start from the templates in `templates/` next to this file. They compile as they are.

1. `cargo init --name <bin>`. Use the **same egui version as Patina** (see Patina's
   `Cargo.toml`; it re-exports `egui`):
   ```toml
   eframe = { version = "0.35", features = ["persistence"] }
   egui = "0.35"
   patina-ui = "0.1"
   ```
   Add `features = ["markdown"]` and so on as the app needs. To build against unreleased
   changes, use `patina-ui = { git = "https://github.com/Sector-F-Labs/patina", branch = "develop" }`
   and read **Unreleased** in Patina's `CHANGELOG.md`.
2. Copy `templates/main.rs` and `templates/app.rs` into `src/`, and replace `APP_NAME`
   (display name), `APP_BIN` (the binary) and `APP_ID` (a reverse-DNS id such as
   `com.example.MyApp`).
3. Copy `templates/Makefile` to the app's root and replace `APP_BIN`, and `APP_ENV` (the
   env-var prefix, e.g. `MYAPP`) in the screenshot target.
4. Build the app's own panels in `app.rs`'s central panel, in its own modules.
5. Add `README.md`, a `.gitignore` with `target/`, a `LICENSE` and an `AGENTS.md` with the
   commands, layout and gotchas.
6. Run `make check`, then `make` to see the help, then look at the window (below).

### The frame needs five things

The template wires them all; keep each when you edit:

- `patina::frame::viewport(builder, custom)` on the `ViewportBuilder`;
- `frame::header_frame` and `TitleBar::new(title, &style).show(ui, left, right)` in the top
  panel, with `.menu_bar(true)` when the left side is `menu_button`s, and
  `.span(frame::header_span(ui))` so the menus fill the bar's height;
- `frame::edges(ctx, radius, border)` after everything else is drawn;
- `frame::clear_color` from `App::clear_color`, so the rounded corners show the desktop;
- `frame::release_grab` from `App::raw_input_hook`, or every second window drag fails.

Also call `patina::appearance::watch(&cc.egui_ctx)` once at start-up, and compute
`skin.frame_style(ui.visuals())` every frame, because Default follows the desktop.

## Rules for a Patina app

- **Title bar: menus left, output right.** Left is a menu bar in words: File, Edit, View, the
  app's own menus, then Skins. No hamburger and no icon toolbar. Right, next to the window
  buttons, are the actions that make the app's main output (Photograph's Render). An app with no
  output leaves it empty. Settings and status go in a menu, a sidebar or a status bar.
- **Stock egui first.** Use the default `Style`, `Visuals` and fonts. Paint your own only where
  egui has no widget, and take colours from the active `Visuals`, so skins reach it.
- **Skinnable all the way down.** Every colour the app paints comes from the skin or the
  visuals. A bad user skin shows an error in the UI; it never crashes the app. An app's own
  colours go under `[app.<bin>]` in the skin, read with `skin.app_section::<T>("<bin>")`.
- **An app's own look is a built-in skin in Patina,** not one compiled into the app. Add the
  `.toml` to Patina's `skins/` and `skin::BUILT_IN`, so every app offers it.
- **`Skin::apply` replaces the whole egui style.** Re-apply the app's own spacing after it.
- **Offer "System Title Bar"** (`ViewportCommand::Decorations`) as an escape hatch.
- **Clickable things are at least 24 px.** Menus use egui's own size (ADR-0022, ADR-0023).
- **Never block the UI.** Do file and network I/O off the UI thread.
- **Editing surfaces get a right-click menu:** Cut, Copy, Paste, Select All.
- **Test egui code through egui:** build an `egui::Context`, feed `RawInput` events (real
  pointer events for clicks and focus) into `ctx.run_ui`, and assert on the result.

Each rule has a reason in `docs/design.md` or an ADR in `docs/adr/`.

## Look at it

Build the app and see it before saying it looks right.

- **Screenshot mode:** with `<APP>_SCREENSHOT=path.png` set, the app sends
  `ViewportCommand::Screenshot` after about a second, saves the `Event::Screenshot` image with
  the `image` crate, and quits. `<APP>_SKIN` sets that run's skin without saving it.
  `examples/gallery.rs` is the working example; copy that code into the app. `make screenshot`
  produces the images for the README.
- **No display?** Run under a virtual one: `xvfb-run -a -s "-screen 0 1600x1000x24" make screenshot`.
- **Look at the PNG** (read the file back), and change what's wrong.
- **Scratch settings:** run with `XDG_CONFIG_HOME` and `XDG_DATA_HOME` pointed at scratch
  folders, as the Makefile's `SCRATCH_ENV` does, so runs never touch real settings. Copy
  `user-dirs.dirs` into the scratch config if the app needs the standard folders.

## Decisions

Record decisions that are hard to reverse as ADRs in `docs/adr/` (see Patina's
`docs/adr/README.md` and ADR-0002). Number them in order, never rewrite a merged one (write a
new one that supersedes it), and write each when the decision is made.

## Changing Patina

- `make check` (tests and `clippy -D warnings`) must pass.
- A change that alters what apps write adds an **Upgrading** line to **Unreleased** in
  `CHANGELOG.md`, in the same commit.
- Decisions about Patina belong in Patina's ADRs.
- If an app needs something Patina should provide, change Patina rather than working around it.
