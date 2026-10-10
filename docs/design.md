# The Patina design language

Patina is how a family of egui desktop apps looks and behaves. Photograph and UnAmp are the first.
The idea comes from an older way of thinking about desktop apps: each app can be skinned until it
looks nothing like its siblings, yet they all keep the same rules. Patina is those rules, plus the
shared code that makes them easy to follow.

KDE and GNOME make their apps look alike. Patina only makes them behave alike, and leaves the
look to each app and its skins.

![The gallery in light and dark](gallery-light.png)

## Principles

1. **Stock egui first.** With no skin chosen, an app uses egui's default `Style`, `Visuals` and
   fonts, following the desktop's light/dark setting. Only paint your own widget when egui has
   none, and then take its colours from the active `Visuals`.
2. **Skinnable all the way down.** Every colour an app paints must be reachable from its theme,
   including the window frame. A theme a user writes must never crash the app. Bad values are
   reported, and unknown keys are ignored with a warning.
3. **Diversity within rules.** Apps share the frame, how it behaves, the theme format's shared
   sections and the rules on this page. Anything particular to one app goes under that app's own
   theme section and code.
4. **Feel native to the desktop.** Moving, resizing, snapping and tiling are left to the
   compositor. Window controls and corners follow GNOME/libadwaita, and the light/dark setting
   follows the XDG portal. A system title bar is always available as a setting, in case the
   app's own frame misbehaves on someone's desktop.
5. **Never block the UI.** File and network I/O happen off the UI thread. A hung network share
   can slow a view down but must never freeze the window (see `tree`).
6. **Record decisions.** Every Patina app keeps ADRs ([ADR-0002](adr/0002-adrs-in-every-patina-app.md)).

## The window

Implemented by `patina::frame`.

- **The app draws its own frame.** The window opens undecorated and transparent, and the top
  panel is a header bar.
- **Header bar layout:** the app's controls on the left, the title in the centre, and more
  controls on the right next to the window buttons.
  - The title is centred on the window, not on the gap between the controls, and gets an
    ellipsis when there's no room.
  - Left controls are either buttons (Photograph's toolbar) or a menu bar (UnAmp's menus).
  - The bar's padding is 12 px horizontal and 8 px vertical. The row is 25 px high.
- **Window buttons:** minimise, maximise/restore and close, 30×22 px, 4 px apart, kept 8 px from
  the right-hand controls.
  - They're quiet until hovered. Close turns GNOME red (`#c01c28`) with a white icon on hover.
  - Their colours are themed apart from the app's buttons.
- **On the bar's empty space:** drag to move, double-click to maximise or restore, right-click
  for a window menu.
- **Edges:** a one-pixel border in the window stroke colour, and invisible resize handles 5 px
  deep (12 px at the corners), drawn above everything else.
- **Corners:** libadwaita's 15 px window radius, square while maximised or full screen.
  - The clear colour is transparent, so the panel holding each corner carries the rounding.
  - The header bar holds both top corners. The bottom corners belong to whichever panels sit
    there, e.g. a sidebar on the left and the central panel on the right.

## Themes

Implemented by `patina::theme`. The format is documented in [themes.md](themes.md).

- **Same files in every app:** themes use the suite format, with a `format` version, shared
  `[colors]`, `[controls]` and `[window]` sections, and one `[app.<name>]` section per app.
  - Keys are named for a role (`surface`, `pressed`, `title_bar`), not for a widget or an app
    feature.
- **Where they come from:**
  - **Default:** stock egui, following the desktop.
  - **Patina's built-ins:** Dark Orange, Photograph's look.
  - **The app's own built-ins.**
  - **The user's themes:** in `~/.config/patina/themes/`, shared by every app.
- **Within the rules:** a theme dropped into any app brings its shared look with it. The app's
  own section waits until that app reads the file. This is how an app can look completely
  different from its siblings while sharing everything else.

## Components

| Module | What | From |
|--------|------|------|
| `frame` | Own window frame: header bar, window buttons, edges, compositor hand-off | UnAmp, Photograph |
| `appearance` | Follow the desktop's light/dark setting (XDG portal) | UnAmp, Photograph |
| `theme` | Suite theme format: parse, apply to egui, frame colours, app sections, loading | UnAmp |
| `tree` | Lazily loaded sidebar folder tree | UnAmp |

Run `make gallery` to see each one. A component belongs in Patina when a second app wants it.
It may use only std, egui and small helper crates, never an app's own types
([ADR-0001](adr/0001-one-crate-for-the-suite.md)).
