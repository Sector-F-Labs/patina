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

use egui::{Button, Context, Margin, RichText, TextEdit, Ui, Vec2, vec2};

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

/// Padding inside a text input, either side of the text and above and below
/// it: a button's, so an input and the button beside it are the same size
/// (ADR-0034).
pub const INPUT_PADDING: Vec2 = BUTTON_PADDING;
/// How high a text input is at least: a button's.
pub const INPUT_HEIGHT: f32 = BUTTON_HEIGHT;

/// How big a single-line text input is: the padding round its text and the
/// least height. The default is [`INPUT_PADDING`] and [`INPUT_HEIGHT`]; a skin
/// changes it with `[controls] input_padding` and `input_height`, and an app
/// with [`set_input_size`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputSize {
    /// Either side of the text, and above and below it.
    pub padding: Vec2,
    /// The input is at least this high.
    pub height: f32,
}

impl Default for InputSize {
    fn default() -> Self {
        Self {
            padding: INPUT_PADDING,
            height: INPUT_HEIGHT,
        }
    }
}

impl InputSize {
    /// The most padding either way and the range of heights a skin's values
    /// are held to, so one can't make an input unusable.
    pub const MAX_PADDING: f32 = 24.0;
    pub const HEIGHTS: (f32, f32) = (16.0, 64.0);

    /// This size with its values held to the limits above.
    pub fn clamped(self) -> Self {
        Self {
            padding: vec2(
                self.padding.x.clamp(0.0, Self::MAX_PADDING),
                self.padding.y.clamp(0.0, Self::MAX_PADDING),
            ),
            height: self.height.clamp(Self::HEIGHTS.0, Self::HEIGHTS.1),
        }
    }
}

fn input_size_id() -> egui::Id {
    egui::Id::new("patina_input_size")
}

/// Sets the size of the text inputs made with [`text_edit`] in this context.
/// `Skin::apply` does it from the skin, so call this after it to override
/// the skin's.
pub fn set_input_size(ctx: &Context, size: InputSize) {
    ctx.data_mut(|d| d.insert_temp(input_size_id(), size.clamped()));
}

/// The size of a text input now: the skin's or the app's, else the default.
pub fn input_size(ctx: &Context) -> InputSize {
    ctx.data(|d| d.get_temp(input_size_id())).unwrap_or_default()
}

/// A single-line text input at the size [`input_size`] gives: padded as a
/// button is and at least 24 px high by default, so it fits the title bar's
/// buttons and the tabs. Add it with `ui.add`. egui's own `TextEdit` is 4 by 2 px
/// of padding and doesn't read it from its style, so inputs a Patina app
/// draws itself use this to match; the tree's filter box does.
pub fn text_edit<'t>(ui: &Ui, text: &'t mut String) -> TextEdit<'t> {
    let size = input_size(ui.ctx());
    // egui (0.35) lets the margin alone set a single line's height, so what
    // padding the least height needs on top of the text's line is added.
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let down = size.padding.y.max(((size.height - line) / 2.0).ceil());
    TextEdit::singleline(text)
        .margin(Margin::symmetric(size.padding.x.round() as i8, down.round() as i8))
        .min_size(vec2(0.0, size.height))
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
    fn inputs_are_a_buttons_size_unless_set() {
        let ctx = egui::Context::default();
        assert_eq!(input_size(&ctx), InputSize::default());
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut text = String::new();
            let height = ui.add(text_edit(ui, &mut text)).rect.height();
            assert!((BUTTON_HEIGHT..=BUTTON_HEIGHT + 1.0).contains(&height), "{height}");
        });
        set_input_size(
            &ctx,
            InputSize {
                padding: vec2(100.0, 0.0),
                height: 200.0,
            },
        );
        assert_eq!(input_size(&ctx).padding, vec2(InputSize::MAX_PADDING, 0.0));
        assert_eq!(input_size(&ctx).height, InputSize::HEIGHTS.1);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut text = String::new();
            let height = ui.add(text_edit(ui, &mut text)).rect.height();
            assert!((InputSize::HEIGHTS.1..=InputSize::HEIGHTS.1 + 1.0).contains(&height), "{height}");
        });
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
