//! Patina: the shared look and parts of a family of egui desktop apps,
//! Photograph and UnAmp first. Each app can be skinned its own way; Patina
//! is the frame they share and the rules they keep. See `docs/design.md` for
//! the design language and `docs/adr/` for why things are the way they are.
//!
//! - [`frame`]: the app's own window frame, header bar and resize edges.
//! - [`appearance`]: following the desktop's light/dark setting.
//! - [`skin`]: skins in the suite format, shared by every app.
//! - [`tabs`]: tabs in the title bar, in place of the window title.
//! - [`tree`]: a lazily loaded sidebar folder tree.
//! - [`ime`]: stops the input method keeping an idle window repainting.
//! - [`menu`]: menus, their items 24 high, and context menus in that style.
//! - [`kinetic`]: a touchpad flick keeps scrolling after the fingers lift.
//! - [`open`]: opens files and folders in the desktop's apps, not the browser.
//! - [`markdown`] (feature `markdown`): markdown shown and edited as Notebook
//!   does, read-only for documentation; source files highlighted.

pub mod appearance;
pub mod frame;
pub mod ime;
pub mod kinetic;
#[cfg(feature = "markdown")]
pub mod markdown;
pub mod menu;
pub mod open;
pub mod skin;
pub mod tabs;
pub mod tree;

/// The egui Patina is built on, so apps use the same version.
pub use egui;
