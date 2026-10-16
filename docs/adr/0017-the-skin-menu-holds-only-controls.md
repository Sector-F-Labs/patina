# 0017. The skin menu holds only controls, in the same order in every app

Date: 2026-10-16

## Status

Accepted. The user found the apps' skin menus inconsistent: Reader's had Follow desktop, UnAmp's
had Copy built-in skins, and none had both. Reader's was refined first and is the model.

## Context

Every app has a skin menu ([ADR-0011](0011-each-app-its-own-skins-folder.md) makes it open and
fill the app's own folder), but each grew its own: different entries, different casing ("Reload
Skins" in Reader, "Reload skins" in UnAmp), and results and load errors listed inside it. egui
also wraps a menu's labels to the menu's first width rather than widening it, so long skin names
and messages broke over lines.

## Decision

The skin menu, top to bottom:

1. **Every skin, as radio buttons.** The user's own skins read `Name — Author` and show their
   file's path on hover.
2. A separator, then **Follow desktop, Light, Dark**, enabled only while the skin follows the
   desktop.
3. A separator, then **Reload skins**, **Copy built-in skins to folder** (which never overwrites,
   then reloads, so the copies are what's in use) and **Open skins folder**.

Labels are in sentence case, as above. The menu sets `TextWrapMode::Extend`, so it's as wide as
its widest entry. It shows no messages: what Copy did and skins that failed to load go to the
status bar ([ADR-0016](0016-a-status-bar-for-messages.md)).

## Consequences

- The menu reads the same in every app, and a user who learns it in one knows it in all.
- An app can't add its own entries to the skin menu without a new decision; app-specific look
  settings go in the app's own menus.
- The menu is copied into each app for now. It's the obvious first candidate to move into Patina
  as a shared `skin::menu`.
