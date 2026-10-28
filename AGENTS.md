# Patina

Patina (`patina-ui`, imported as `patina`) is the window frame, skin format and components for
egui desktop apps. Read `README.md` and `docs/design.md` first.

## Building an app on Patina

Use the `patina-app` skill in `skills/patina-app/`. It has the setup, the rules and
compiling templates for a new app.

## Commands

- `make check`: tests and `clippy -D warnings`. Run it before every commit.
- `make gallery`: every component on one screen.
- `make screenshot`: regenerate `docs/gallery-*.png`. With no display, run it under
  `xvfb-run -a -s "-screen 0 1600x1000x24"`.

## Conventions

- A change that alters what apps write adds a line to **Unreleased** in `CHANGELOG.md`, in the
  same commit.
- Decisions that are hard to reverse get an ADR in `docs/adr/` (the `adr` skill).
- `docs/egui-pitfalls.md` and `docs/known-issues.md` list traps; read them before debugging a
  frozen window or one using a full core.
