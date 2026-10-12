# 0008. Show only the window buttons the compositor supports, found by compositor name for now

Date: 2026-10-13

## Status

Accepted

## Context

Patina's header bar always drew minimise, maximise and close. On dwl, which we're looking at as
the base of a Patina desktop, a nested test showed the frame sending the right requests
(`set_minimized`, `set_maximized`) and dwl ignoring them. dwl has no minimise, doesn't support
maximise, and tells each window so: it advertises only the fullscreen capability in xdg-shell's
`wm_capabilities`. So two of the three buttons did nothing.

The proper fix is to read `wm_capabilities`, and we can't. smithay-client-toolkit parses it into
`WindowConfigure.capabilities`, but winit 0.30, which eframe 0.35 uses, never reads that field and
has no API for it. Patina can't listen on winit's `xdg_toplevel` itself, and a second Wayland
connection can't see another connection's windows.

Options considered:

- **Patch winit** to expose the capabilities. This would be proper detection, but Patina is a
  library, so every app would need a `[patch.crates-io]` pointing at a fork we'd have to maintain.
- **Guess from `XDG_CURRENT_DESKTOP`.** dwl doesn't set it, so this misses the one case we need.
- **An env var and a `TitleBar` option** for which buttons to show. It works anywhere, but needs
  setting up per session or per app.
- **Identify the compositor by its process**, as fastfetch does: connect to the Wayland socket,
  ask the kernel for the peer's PID (`SO_PEERCRED`) and read `/proc/<pid>/comm`. Then look the
  name up in a table of compositors without minimise and maximise.

We only need dwl to work right now, so the table is enough.

## Decision

We will hide minimise and maximise wherever `frame::can_minimise_and_maximise()` says no. That
covers the buttons, the window menu and double-click on the bar. Close is always shown.

The function is public and is the stable seam: apps can call it too. For now it identifies the
compositor through `SO_PEERCRED` and checks the name against `WITHOUT_MIN_MAX`, which lists only
`dwl`. The result is cached for the life of the process. A compositor that can't be identified,
such as from inside a sandbox with its own PID namespace or on a non-Linux build, is assumed to
support both, so the frame looks as it did before.

## Consequences

- On dwl, the header bar shows only close, and no control in the frame does nothing.
- Elsewhere nothing changes, including on GNOME, where the compositor is `gnome-shell`.
- **The table checks names, not capabilities.** It has to be kept up to date by hand and is
  wrong for a compositor that's renamed, forked or wrapped. A Patina fork of dwl that adds
  minimise or maximise would need its own entry, or dwl's entry removed.
- **Sandboxed apps** (Flatpak) can't see the compositor's process, so they show all three
  buttons even on dwl.
- **Small new costs:** a direct `libc` dependency on Linux (it was already in the tree) and one
  `unsafe` `getsockopt` call, because std's `UnixStream::peer_cred` is still unstable. It also
  opens one extra short-lived connection to the compositor, once per process.
- **Revisit when winit exposes `wm_capabilities`** (or a later eframe moves to a winit that
  does). `can_minimise_and_maximise` should then read the real capabilities and the table can
  go. Callers won't change.
