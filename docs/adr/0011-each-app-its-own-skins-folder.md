# 0011. Each app keeps its own skins folder, `~/.config/<app>/skins`

Date: 2026-10-13

## Status

Accepted. The user asked for it: "each app should look at their own, not the patina folder, thats
the whole point". Supersedes the shared folder in [ADR-0004](0004-skins-in-patina.md).

## Context

ADR-0004 gave every app one shared folder, `~/.config/patina/skins/`, and the apps' skin menus
copied built-ins into it and opened it. UnAmp had opened its own `~/.config/unamp/skins/` before
the Patina port, and the user's skins were all still there. So its menu opened an empty folder,
and a skin made for one app showed up in all of them. Skins are how an app gets a look of its own
(ADR-0007), so the user wants each app's skins to be its own.

## Decision

- **`skin::app_dir(app)`** is `~/.config/<app>/skins`, named after the app's config folder. Apps
  load from it, and the skin menu's Copy built-in skins and Open skins folder use it.
- **Apps don't read `~/.config/patina/skins/`.** `skin::shared_dir` is deprecated.
- The format stays shared. A skin file copied into another app's folder works there, with that
  app's `[app.<name>]` section.
- The gallery uses `~/.config/patina-gallery/skins`.

## Consequences

- Each app's menu opens the folder of skins it actually shows, and a skin changes one app.
- Using a skin in several apps means copying it into each, and edits don't carry across.
- Skins left in `~/.config/patina/skins/` stop loading; they have to be moved. None existed on the
  user's machine when this was decided.
