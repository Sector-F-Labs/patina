# 0004. Move the suite theme format into Patina, with a shared themes folder and Dark Orange built in

Date: 2026-10-11

## Status

Proposed. The user asked for UnAmp's TOML themes (not its Winamp skins) to move into Patina, and
for Photograph's look to be built in as "Dark Orange" in the gallery's theme menu. The API, the
shared folder and the built-ins split below are my design.

## Context

UnAmp's `skin.rs` already reads the suite theme format (UnAmp ADR-0024):

- shared `[colors]`, `[controls]`, `[window]` and `[window.controls]` sections;
- `[app.<name>]` sections per app;
- unknown keys listed as warnings rather than errors.

Its Photograph skin is Photograph's hand-written visuals in that format. Photograph itself has no
themes.

UnAmp's ADR-0024 put off a shared themes folder "for when there's a second app reading the
format". Patina is where that second reader comes from.

Some of `skin.rs` is UnAmp's alone:

- the `[app.unamp]` section and the palette it resolves (spectrum, time display);
- the classic `.wsz` conversion and renderer;
- the Steam Classic built-in.

## Decision

- **`patina::theme` holds the shared part of `skin.rs`:** parsing (format check, warnings for
  unknown keys), the colour sections, applying a theme to egui's visuals, the frame colours
  (`Theme::frame_style`, built on `FrameStyle::from_visuals`), loading and copying out
  built-ins.
- **App sections stay with the apps.** `[app]` is kept as a raw TOML table, and an app
  deserializes its own section with `Theme::app_section::<T>(name)`, which reports that
  section's unknown keys. Patina never warns about any app's section.
- **Built-ins:**
  - Patina ships `themes/dark-orange.toml` (`theme::BUILT_IN`). It's UnAmp's Photograph skin,
    renamed **Dark Orange** and credited to Patina, and it keeps its `[app.unamp]` section so
    UnAmp loses nothing.
  - An app passes Patina's built-ins together with its own to `load_all`.
- **Folders:**
  - `theme::shared_dir()` is `~/.config/patina/themes/`, which every app reads.
  - An app may add folders of its own; UnAmp keeps `~/.config/unamp/skins/` for its `.wsz` files.
  - Later folders replace earlier themes of the same name.
- **Words:** Patina says "theme". An app can still call them skins in its menus, as UnAmp does.
- **Docs:** `docs/themes.md` holds the format reference (the shared part of UnAmp's
  `docs/skinning.md`). The gallery's Theme menu lists the themes, offers light/dark only under
  Default, and has Reload and Copy built-in themes to folder.

## Consequences

- Photograph gets themes, Dark Orange included, by switching to Patina, with its old look as one
  of the options.
- One theme file in `~/.config/patina/themes/` themes every app.
- When UnAmp switches over, its Photograph skin becomes Patina's Dark Orange. A user whose
  config says `skin = "Photograph"` falls back to Default unless UnAmp maps the old name. Its
  ADRs should record that.
- The format's shared sections now change in Patina, and every app picks up the change with its
  next Patina release. A new shared key needs a Patina release before an app can rely on it.
- `toml`, `serde`, `serde_ignored` and `dirs` become Patina dependencies.
