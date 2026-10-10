//! Patina: the shared look and parts of a family of egui desktop apps,
//! Photograph and UnAmp first. Each app can be skinned its own way; Patina
//! is the frame they share and the rules they keep. See `docs/design.md` for
//! the design language and `docs/adr/` for why things are the way they are.
//!
//! - [`frame`]: the app's own window frame, header bar and resize edges.
//! - [`appearance`]: following the desktop's light/dark setting.
//! - [`skin`]: skins in the suite format, shared by every app.
//! - [`tree`]: a lazily loaded sidebar folder tree.

pub mod appearance;
pub mod frame;
pub mod skin;
pub mod tree;

/// The egui Patina is built on, so apps use the same version.
pub use egui;
