# 0001. Share the suite's frame and components as one published crate, Patina

Date: 2026-10-11

## Status

Accepted

## Context

Photograph and UnAmp were built in the same style, and code started moving between them by copy:

- **`frame.rs`:** UnAmp drew its own window frame first, and Photograph ported it. Within a day
  the two copies differed by 164 lines. UnAmp had theming for the frame and a fix for the menu
  bar overflowing the window. Photograph had left and right toolbar slots.
- **A fix landed twice.** The fix for moving the window failing every other time was committed
  to both repos a minute apart (Photograph `ff518e4`, UnAmp `fb47aa2`).
- **Plain copies:** `appearance.rs` (desktop light/dark) differed by 4 lines and `locations.rs`
  by 22. UnAmp's ADR-0005 chose copying for now, with "a third app, or the copies diverge" as
  the trigger to extract a crate.
- **Shared themes were already planned.** UnAmp's theme format (its ADR-0024) has sections shared
  across apps and was designed for a suite.
- **Photograph wants more:** UnAmp's lazy folder tree, and eventually skins.

We considered:

- **Only the window frame and theme.** It doesn't cover the tree, or other components later.
- **A general egui widget library.** That would be scope without a second user.
- **One crate for the family**, holding components a second app wants plus the design
  language's documentation. This won. The user named the family style **Patina**: rust made
  beautiful, where each piece ages its own way. The look varies, the rules don't.

## Decision

We will build Patina as one library crate in `com/sectorflabs/patina`.

- **Name:** `patina-ui` on crates.io (`patina` is taken), imported as `patina`.
- **Published to crates.io**, because UnAmp is published there and crates.io doesn't accept git
  or path dependencies.
- **What goes in:** a component goes in when a second app wants it, and it uses only std, egui
  and small helper crates (zbus for the portal), never an app's own types. It starts with
  `frame`, `appearance` and `tree`, each moved from UnAmp, the more developed copy.
  - **Later steps:** the theme loader ([ADR-0004](0004-themes-in-patina.md)), and maybe mount discovery behind a
    feature.
- **What stays out:** app-specific parts such as UnAmp's classic `.wsz` renderer, playlist
  reordering and window tiling.
- **License:** BSD-3-Clause. It's permissive, so both the BSD UnAmp and the GPL-2.0 Photograph
  can use it.
- **Re-export `egui`**, so apps build against the same version.
- **Docs and gallery:** `docs/design.md` is the home of the design language, and
  `examples/gallery.rs` shows every component and produces the docs screenshots.

## Consequences

- A frame fix lands once and reaches both apps on their next dependency bump.
- Photograph gets the frame theming and overflow fix, and the tree, by switching over rather
  than porting.
- Both apps have to move to a new egui together, in step with Patina.
- A change that spans Patina and an app is now two changes and a release. During development a
  `[patch.crates-io]` path override keeps that cheap.
- The design rules that were spread across both apps' ADRs now live in one place. The apps'
  ADRs point here when they switch.
