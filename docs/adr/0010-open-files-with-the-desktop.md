# 0010. Open files and folders with the desktop's opener, through `patina::open`

Date: 2026-10-13

## Status

Proposed. The user found Notebook's "Open skins folder" opening Firefox instead of Nautilus and
asked for the fix to live in Patina and go into every app; the API is my design.

## Context

Photograph, md-chat and Notebook opened the skins folder with egui's `open_url` and a `file://`
URL. eframe hands every URL to the web browser, so the folder opened as a listing in Firefox.
UnAmp ran `xdg-open` itself and got the file manager, but only on Linux. Notebook later needed the
same thing for "Open Image". Each app writing its own opener is how they came to differ (ADR-0001).

## Decision

- **`patina::open::path(&Path) -> io::Result<()>`** opens a file or folder in the desktop's app
  for it: `xdg-open` on Linux and the BSDs, `open` on macOS, `explorer` on Windows. It returns once
  the opener has started, and reaps it on a thread.
- **Apps use it for anything on disk**, the skins folder first. `open_url` stays for web links.

## Consequences

- "Open skins folder" opens the file manager in every app, and an image opens in the image viewer.
- Patina now starts processes. The opener's own failures (no app for the type, say) aren't seen:
  only failing to start it is an error.
- A desktop without `xdg-open` gets nothing on Linux, where before the browser at least opened.
