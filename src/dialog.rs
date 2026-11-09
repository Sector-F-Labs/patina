//! Buttons the size of a title bar's (ADR-0032).
//!
//! egui's buttons are 18 px high with 4 by 1 px of padding, which next to a
//! Patina title bar's look small and are under ADR-0022's 24 px. The bar's
//! buttons are Photograph's (ADR-0028): 10 px either side of the text, 5 above
//! and below, and never under 24 px high. Dialogs, panels and menus that have
//! a row of buttons give it the same size by calling [`buttons`] first inside
//! the row's layout, so only that row changes:
//!
//! ```no_run
//! # fn row(ui: &mut patina::egui::Ui) {
//! use patina::egui::{Align, Layout};
//! ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
//!     patina::dialog::buttons(ui);
//!     // The button Enter presses, then the others, since the layout runs
//!     // from the right.
//!     if ui.add(patina::dialog::primary(ui, "Delete")).clicked() {}
//!     if ui.button("Cancel").clicked() {}
//! });
//! # }
//! ```

use egui::{Button, RichText, Ui, Vec2, vec2};

/// Padding inside a button, either side of the text and above and below it:
/// the title bar's (ADR-0028).
pub const BUTTON_PADDING: Vec2 = vec2(10.0, 5.0);
/// How high a button is at least (ADR-0022, ADR-0028).
pub const BUTTON_HEIGHT: f32 = 24.0;
/// Space between the buttons of a row.
pub const BUTTON_GAP: f32 = 8.0;

/// Sizes the buttons in this `ui` as the title bar's: [`BUTTON_PADDING`] of
/// padding and at least [`BUTTON_HEIGHT`] high, [`BUTTON_GAP`] apart. Call it
/// inside the row's own layout, or in a scope of its own, since it changes the
/// `ui`'s spacing for everything after it, which sizes other widgets too.
pub fn buttons(ui: &mut Ui) {
    let spacing = ui.spacing_mut();
    spacing.button_padding = BUTTON_PADDING;
    spacing.interact_size.y = spacing.interact_size.y.max(BUTTON_HEIGHT);
    spacing.item_spacing.x = BUTTON_GAP;
}

/// The button Enter presses, in the skin's accent (its selection colours),
/// where the others are plain.
pub fn primary(ui: &Ui, text: impl Into<String>) -> Button<'static> {
    let v = ui.visuals();
    Button::new(RichText::new(text).color(v.selection.stroke.color)).fill(v.selection.bg_fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_of_buttons_is_the_bars_size() {
        let ctx = egui::Context::default();
        let mut seen = None;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let before = ui.spacing().interact_size.y;
            assert!(before < BUTTON_HEIGHT, "egui's own are smaller");
            ui.scope(|ui| {
                buttons(ui);
                seen = Some((ui.spacing().button_padding, ui.spacing().interact_size.y));
            });
            assert_eq!(ui.spacing().interact_size.y, before, "only the row changed");
        });
        assert_eq!(seen, Some((BUTTON_PADDING, BUTTON_HEIGHT)));
    }

    #[test]
    fn a_taller_row_stays_taller() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.spacing_mut().interact_size.y = 30.0;
            buttons(ui);
            assert_eq!(ui.spacing().interact_size.y, 30.0);
        });
    }
}
