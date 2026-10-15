# 0013. Alt and a number shows that tab

Date: 2026-10-15

## Status

Proposed. The user asked for Teleprinter's Alt+1 to Alt+9 (its ADR-0012) to become the standard
for Patina apps with tabs, but not for the desktop's bar.

## Context

`tabs::keys` (ADR-0009) gives apps with tabs Ctrl+Shift+T and Ctrl+PageUp/PageDown. Teleprinter
added Alt+1 to Alt+9 for jumping straight to a tab, as GNOME Terminal and browsers do. Left there,
Notebook and later apps would each add their own, or not, and drift (ADR-0007).

Two things made it more than a key check. egui sends Alt+digit as a key event and then as the
digit typed as text, so taking only the key still types the digit. And readline uses Alt+digit
for a numeric argument, so a terminal shouldn't lose keys that switch to nothing.

## Decision

- **`tabs::keys` takes Alt+1 to Alt+9** and returns `TabAction::Show(n)` for the nth tab. No
  change to its signature: apps calling it get the keys with the next Patina release.
- **Only when that tab exists.** The key and the digit text after it are both consumed; past the
  last tab both go on to the app's widgets. With one tab, only Alt+1 is taken.
- Alt+9 is the ninth tab, not the last one as in browsers: the user asked for "Alt+1 is first,
  Alt+2 is second".
- **The desktop's bar stays out.** It uses `TabStrip` but not `keys` (ADR-0012): its tabs are
  windows, and the compositor owns the desktop's keys. Nothing in Patina changes for it.

## Consequences

- Every Patina app that calls `keys` switches tabs the same way. Notebook gains the keys.
- Alt+1 to Alt+9 no longer reach an app's widgets while those tabs exist. An app needing them
  can skip `keys`, as it already could for the others.
- Teleprinter drops its own copy once it depends on this release.
