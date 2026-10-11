# Patina

Patina is the shared window frame, components and design language of a family of
[egui](https://github.com/emilk/egui) desktop apps. [Photograph](https://github.com/divanvisagie/Photograph)
and [UnAmp](https://github.com/divanvisagie/UnAmp) are the first.

You can skin each app completely differently. What they share is how they behave: one window
frame, one skin format and one set of rules. See [docs/design.md](docs/design.md).

![The Patina gallery](docs/gallery-dark.png)

## Use

```toml
[dependencies]
patina-ui = "0.1"
```

```rust
use patina::frame::{self, FrameStyle, TitleBar};

let style = FrameStyle::from_visuals(ui.visuals());
let radius = frame::corner_radius(ui.ctx(), style.radius);
egui::Panel::top("header")
    .frame(frame::header_frame(ui.style(), &style, radius))
    .show(ui, |ui| {
        TitleBar::new("My App", &style).show(ui, |ui| { /* left */ }, |ui| { /* right */ });
    });
```

The full wiring is in [`src/frame.rs`](src/frame.rs) and [`examples/gallery.rs`](examples/gallery.rs).
It covers the viewport, clear colour, edges and input hook.

Patina re-exports `egui`, and an app has to use the same egui version.

## Components

- **`frame`**: the app's own window frame, with a header bar, GNOME-style window controls, resize
  edges and rounded corners. Moving and resizing go to the compositor.
- **`appearance`**: follows the desktop's light/dark setting through the XDG portal.
- **`skin`**: skins in the suite format, which every app reads, with app sections, a shared
  user folder and the built-ins Dark Orange, Calendar, Notebook and Steam Classic. See [docs/skinning.md](docs/skinning.md).
- **`tree`**: a sidebar folder tree that lists folders on worker threads, so slow network shares
  never block the UI.

Known problems and what's understood about them are in [docs/known-issues.md](docs/known-issues.md).

## Develop

```sh
make              # list targets
make gallery      # every component on one screen
make test         # tests
make check        # tests and clippy
make screenshot   # docs/gallery-*.png
```

Decisions are recorded as ADRs in [docs/adr](docs/adr/README.md), and we recommend the same for
every Patina app.

## License

BSD-3-Clause
