# 0018. TabStrip can show an icon on each tab, with or without its title

Date: 2026-10-17

## Status

Proposed. The user asked for app icons in the Patina desktop bar's window list, which is a
`TabStrip` (ADR-0012), with a setting for icons only, titles only, or both. The API is my design.

## Context

A tab only showed its title. The desktop bar's tabs are open windows, and an app's icon tells
them apart faster than a title. With many windows open, icon-only tabs also take a fraction of the
room. Other apps may want the same: a browser shows site icons, a messenger shows chat avatars.

## Decision

Two builder options. Without them nothing changes:

- `.icons(&[Option<TextureId>])` gives each tab an icon, drawn 18 px square before the title
  and centred with it. The caller loads the textures. A tab whose icon is `None`, because it
  isn't loaded yet or there isn't one, shows its title's first letter in a small filled square
  in the icon's place. That keeps the tabs lined up and fills them as icons arrive.
- `.label(TabLabel)` picks what a tab shows: `Title` (the default), `Icon` or `IconAndTitle`.
  Without `.icons` it's always the title.
- Icon-only tabs are a fixed 44 px wide and show their title on hover. They have no close cross,
  because it would cover the icon. A middle click and the tab's menu still close them.

## Consequences

- Existing callers are unchanged.
- Icon lookup (themes, `.desktop` files) stays with the caller. Patina doesn't read the file
  system for icons.
- Icon-only tabs can't be closed by a plain left click anywhere. That's the trade for not
  covering the icon.
