# 0020. Tabs take Ctrl+T and Ctrl+W; a terminal takes them with Shift

Date: 2026-10-19

## Status

Proposed. The user asked for Super+T to open a tab in every Sector F app, as Cmd+T does on a Mac;
the desktop's half is patina-desktop's ADR-0067. The split into two functions is my design.

## Context

`tabs::keys` (ADR-0009) took Ctrl+Shift+T for a new tab, a terminal's key, since Teleprinter's
tabs came first. Other apps open a tab with Ctrl+T and close it with Ctrl+W; in a browser
Ctrl+Shift+T reopens a closed tab. Files and Reader took Ctrl+T and Ctrl+W themselves, so the
Sector F apps already disagreed with each other.

Super never reaches a Patina app on Linux: egui-winit drops it. So the Patina desktop now sends
a Super key as Ctrl+key, or as Ctrl+Shift+key to a terminal (patina-desktop ADR-0067), and the
apps only need to agree on the Ctrl keys.

## Decision

- **`tabs::keys` takes Ctrl+T for `TabAction::New` and Ctrl+W for `Close` of the shown tab,**
  only without Shift or Alt. Ctrl+Shift+T is no longer taken by it.
- **`tabs::terminal_keys`** is the same with Ctrl+Shift+T and Ctrl+Shift+W, for a terminal, whose
  Ctrl+T and Ctrl+W are the shell's. Teleprinter calls it.
- `TabStrip`'s `+` hint says Ctrl+T; a terminal sets its own with `new_hint`.
- Ctrl+PageUp/PageDown and Alt+1 to Alt+9 are unchanged in both.

## Consequences

- Every app with tabs opens and closes them with the same keys, and with Super+T and Super+W on
  the Patina desktop. Apps drop their own Ctrl+T and Ctrl+W.
- An app whose widgets need Ctrl+W (none yet) can skip `keys`, as before.
- An app with tabs that can't close (ADR-0014) gets `Close` from Ctrl+W and ignores it.
- Notebook's Ctrl+Shift+T becomes Ctrl+T, a change for anyone used to it.
