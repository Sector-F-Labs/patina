# 0014. TabStrip can show tabs that can't be closed

Date: 2026-10-16

## Status

Proposed. The user asked for Messenger's Telegram folders to show as Patina tabs; the option is
my design.

## Context

`TabStrip` (ADR-0009) gives every tab a close cross on hover, closes it on a middle click, and
offers "Close Tab" in its menu. ADR-0012 already let a strip leave out new tabs, for tabs the app
doesn't create. Messenger's tabs are the user's chat folders, plus All Chats and Archived. They're
views the app always has: closing one from the tab would mean deleting the folder on Telegram, or
hiding a view the user can't get back. A cross that does nothing would be worse than none.

## Decision

A builder option, defaulting to today's behaviour:

- `.closable(false)` draws no close cross, ignores a middle click, and leaves "Close" out of a
  tab's menu. `TabAction::Close` then never comes from the strip.
- A tab's title, which normally keeps clear of the cross, may use that space.
- With `.new_tab(false)` as well, a tab has no menu at all.

## Consequences

- Existing callers are unchanged.
- `tabs::keys` still sends `TabAction::New` for Ctrl+Shift+T. An app without new tabs ignores it,
  as with ADR-0012.
