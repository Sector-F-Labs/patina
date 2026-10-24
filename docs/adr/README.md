# Architecture Decision Records

This directory holds ADRs for Patina — short records of significant architecture decisions,
written at the time they're made, following Michael Nygard's format as popularized by Martin
Fowler: <https://martinfowler.com/bliki/ArchitectureDecisionRecord.html>.

## When to write one

Write an ADR when a decision is:

- Hard, or expensive, to reverse once other code depends on it.
- Not obvious from reading the code — a "why", not a "what".
- The kind of thing that'll get re-litigated later without a record ("didn't we already decide
  this?").

Small, reversible choices don't need one. Rule of thumb: if you'd want to explain the choice to a
new contributor in a paragraph beyond what the code itself shows, it's ADR-sized.

An ADR is a point-in-time record of *a decision as it was made*, including the options considered
and rejected. It does not get rewritten later — a decision that reverses or replaces an earlier one
gets its own new ADR that supersedes the old one, which stays in place with its status updated.

## Convention

- One file per decision: `NNNN-short-kebab-title.md`, numbered sequentially, zero-padded to 4
  digits. Numbers are never reused, even for a decision that gets reversed.
- Start from [`0000-template.md`](0000-template.md) — copy it, don't write from scratch.
- `Status` is one of: `Proposed`, `Accepted`, `Rejected`, `Deprecated`, or `Superseded by
  ADR-NNNN` (linked to the superseding record).
- Every new ADR gets a row in the index below.
- Keep records short — a paragraph or two per section is normal. An ADR documents a decision and
  its reasoning, not a design document.

For the design language itself, see [../design.md](../design.md).

If you are working with Claude Code in this repo, `.claude/skills/adr/SKILL.md` handles the
mechanics of adding one. Patina apps should keep ADRs the same way
([ADR-0002](0002-adrs-in-every-patina-app.md)): copy this README (minus the index),
`0000-template.md` and the skill.

## Index

| # | Title | Status |
|---|-------|--------|
| [0001](0001-one-crate-for-the-suite.md) | Share the suite's frame and components as one published crate, Patina | Accepted |
| [0002](0002-adrs-in-every-patina-app.md) | Recommend ADRs for every Patina app, and keep them for Patina itself | Accepted |
| [0003](0003-frame-api.md) | The frame's API: a TitleBar builder with left and right slots, styled by FrameStyle | Proposed |
| [0004](0004-skins-in-patina.md) | Move the suite skin format into Patina, with a shared skins folder and Dark Orange built in | Proposed; folder superseded by [0011](0011-each-app-its-own-skins-folder.md) |
| [0005](0005-app-skins-built-into-patina.md) | Build Calendar, Notebook and Steam Classic into Patina, and colour controls on a coloured bar | Accepted |
| [0006](0006-title-bar-menus-left-output-actions-right.md) | Title bar: word menus on the left, the app's output actions on the right | Accepted |
| [0007](0007-unity-in-variety.md) | Unity in variety: the structure is shared, the skin varies, and GNOME is only a source of defaults | Accepted |
| [0008](0008-window-buttons-follow-the-compositor.md) | Show only the window buttons the compositor supports, found by compositor name for now | Accepted |
| [0009](0009-tabs-in-the-title-bar.md) | Tabs in the title bar, as a component apps fill with their own tabs | Proposed |
| [0010](0010-open-files-with-the-desktop.md) | Open files and folders with the desktop's opener, through `patina::open` | Proposed |
| [0011](0011-each-app-its-own-skins-folder.md) | Each app keeps its own skins folder, `~/.config/<app>/skins` | Accepted |
| [0012](0012-tab-strip-without-new-tabs.md) | TabStrip can leave out new tabs and name what a tab is | Proposed |
| [0013](0013-alt-and-a-number-shows-that-tab.md) | Alt and a number shows that tab | Proposed |
| [0014](0014-tabs-that-cant-close.md) | TabStrip can show tabs that can't be closed | Proposed |
| [0015](0015-markdown-from-notebook.md) | Notebook's markdown display moves into Patina, behind a feature | Accepted |
| [0016](0016-a-status-bar-for-messages.md) | A status bar along the bottom: the version, then colour-coded messages, then the app's own status | Accepted |
| [0017](0017-the-skin-menu-holds-only-controls.md) | The skin menu holds only controls, in the same order in every app | Accepted |
| [0018](0018-tabs-with-icons.md) | TabStrip can show an icon on each tab, with or without its title | Proposed |
| [0019](0019-kinetic-scrolling-on-a-touchpad.md) | A touchpad flick keeps scrolling after the fingers lift, through `patina::kinetic` | Proposed |
| [0020](0020-ctrl-t-and-w-and-a-terminals-keys.md) | Tabs take Ctrl+T and Ctrl+W; a terminal takes them with Shift | Proposed |
| [0021](0021-tabs-that-move-and-launchers.md) | Tabs can be dragged into another order, and some tabs can be launchers | Proposed |
| [0022](0022-clickable-elements-are-at-least-24-px.md) | Clickable elements are at least 24 px, and menus' items 24 px high | Menus superseded by 0023 |
| [0023](0023-menus-are-egui-s-own-size.md) | Menus are egui's own size, as Teleprinter's | Accepted |
| [0024](0024-the-menu-bar-s-menus-fill-the-bar.md) | The menu bar's menus fill the bar, and light as a hovered tab | Accepted |
| [0025](0025-a-changelog-for-apps.md) | A changelog for apps: what changed in each version, and what an app has to change | Accepted |

## Decision Relationship

```mermaid
flowchart TD
    A[0001: One crate for the suite] --> B[0002: ADRs in every Patina app]
    A --> C[0003: Frame API]
    A --> D[0004: Skins in Patina]
    C --> D
    D --> E[0005: App skins built in, bar controls]
    C --> F[0006: Menus left, output actions right]
    C --> E
    D --> G[0007: Unity in variety]
    C --> G
    C --> H[0008: Window buttons follow the compositor]
    F --> I[0009: Tabs in the title bar]
    A --> J[0010: Open files with the desktop]
    D -->|folder superseded by| K[0011: Each app its own skins folder]
    I --> L[0012: Tab strip without new tabs]
    I --> M[0013: Alt and a number shows that tab]
    I --> N[0014: Tabs that can't close]
    I --> Q[0018: Tabs with icons]
    Q --> S[0021: Tabs that move, and launchers]
    C --> O[0016: Status bar for messages]
    K --> P[0017: Skin menu holds only controls]
    O --> P
    C --> R[0019: Kinetic scrolling on a touchpad]
    P --> T[0022: Clickable at least 24 px, menus 24 an item]
    T --> U[0023: Menus are egui's own size]
    F --> V[0024: Menu bar's menus fill the bar]
    I --> V
    A --> W[0025: A changelog for apps]
```

## Revisit Triggers

- An app needs a frame option the builder lacks, or a desktop where the frame misbehaves. See
  [ADR-0003](0003-frame-api.md).
- egui releases break the apps' shared upgrade, or one app needs to stay behind. See
  [ADR-0001](0001-one-crate-for-the-suite.md).
- A shared skin key needs to differ between apps, or users want per-app overrides of a shared
  skin. See [ADR-0004](0004-skins-in-patina.md).
- winit exposes Wayland's `wm_capabilities`, or a compositor in the window-button table changes
  what it supports. See [ADR-0008](0008-window-buttons-follow-the-compositor.md).
- egui glides after a touchpad lift itself, or winit exposes libinput's hold gesture. See
  [ADR-0019](0019-kinetic-scrolling-on-a-touchpad.md).
- egui lights a menu bar's open menu, or lets the bar find its menus' buttons. See
  [ADR-0024](0024-the-menu-bar-s-menus-fill-the-bar.md).
- A component only one app uses keeps growing app-specific options. Move it back into the app.
