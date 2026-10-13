# 0004. Move the suite skin format into Patina, with a shared skins folder and Dark Orange built in

Date: 2026-10-11

## Status

The shared folder is superseded by [ADR-0011](0011-each-app-its-own-skins-folder.md): each app
keeps its own. The rest stands.

Proposed. The user asked for UnAmp's TOML skins (not its Winamp skins) to move into Patina, and
for Photograph's look to be built in as "Dark Orange" in the gallery's skin menu. The API, the
shared folder and the built-ins split below are my design.

## Context

UnAmp's `skin.rs` already reads the suite skin format (UnAmp ADR-0024):

- shared `[colors]`, `[controls]`, `[window]` and `[window.controls]` sections;
- `[app.<name>]` sections per app;
- unknown keys listed as warnings rather than errors.

Its Photograph skin is Photograph's hand-written visuals in that format. Photograph itself has no
skins.

UnAmp's ADR-0024 put off a shared skins folder "for when there's a second app reading the
format". Patina is where that second reader comes from.

Some of `skin.rs` is UnAmp's alone:

- the `[app.unamp]` section and the palette it resolves (spectrum, time display);
- the classic `.wsz` conversion and renderer;
- the Steam Classic built-in.

## Decision

- **`patina::skin` holds the shared part of `skin.rs`:** parsing (format check, warnings for
  unknown keys), the colour sections, applying a skin to egui's visuals, the frame colours
  (`Skin::frame_style`, built on `FrameStyle::from_visuals`), loading and copying out
  built-ins.
- **App sections stay with the apps.** `[app]` is kept as a raw TOML table, and an app
  deserializes its own section with `Skin::app_section::<T>(name)`, which reports that
  section's unknown keys. Patina never warns about any app's section.
- **Built-ins:**
  - Patina ships `skins/dark-orange.toml` (`skin::BUILT_IN`). It's UnAmp's Photograph skin,
    renamed **Dark Orange** and credited to Patina, and it keeps its `[app.unamp]` section so
    UnAmp loses nothing.
  - An app passes Patina's built-ins together with its own to `load_all`.
- **Folders:**
  - `skin::shared_dir()` is `~/.config/patina/skins/`, which every app reads.
  - An app may add folders of its own; UnAmp keeps `~/.config/unamp/skins/` for its `.wsz` files.
  - Later folders replace earlier skins of the same name.
- **Words:** they're called **skins** everywhere: in the code (`patina::skin`, `Skin`), the
  folders, the docs and the menus. The user chose this, matching UnAmp's menus and the
  old-school idea behind Patina. "Theme" is kept for egui's light/dark theme only.
- **Docs:** `docs/skinning.md` holds the format reference (the shared part of UnAmp's
  `docs/skinning.md`). The gallery's Skin menu lists the skins, offers light/dark only under
  Default, and has Reload and Copy built-in skins to folder.

## Consequences

- Photograph gets skins, Dark Orange included, by switching to Patina, with its old look as one
  of the options.
- One skin file in `~/.config/patina/skins/` skins every app.
- When UnAmp switches over, its Photograph skin becomes Patina's Dark Orange. A user whose
  config says `skin = "Photograph"` falls back to Default unless UnAmp maps the old name. Its
  ADRs should record that.
- The format's shared sections now change in Patina, and every app picks up the change with its
  next Patina release. A new shared key needs a Patina release before an app can rely on it.
- `toml`, `serde`, `serde_ignored` and `dirs` become Patina dependencies.
