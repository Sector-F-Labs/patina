//! Menus as every Patina app draws them (ADR-0023): egui's own menu style,
//! its flat buttons lit under the pointer, 18 px an item with 3 px between,
//! as Teleprinter's File, Edit and View menus are.
//!
//! A context menu opened with [`context`] and a menu drawn some other way,
//! as on the Patina desktop, with [`style`], look like the menus a menu bar
//! opens. The style is Patina's to change in one place.

use egui::{InnerResponse, Popup, Response, Style, Ui};

/// A menu's style: egui's, as its menu bars use.
pub fn style(style: &mut Style) {
    egui::containers::menu::menu_style(style);
}

/// The context menu `response` opens on a secondary click, as
/// `Response::context_menu` does but in Patina's [`style`].
pub fn context(response: &Response, add: impl FnOnce(&mut Ui)) -> Option<InnerResponse<()>> {
    Popup::context_menu(response).style(style).show(add)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_menu_item_is_as_high_as_in_a_menu_bar() {
        let ctx = egui::Context::default();
        let mut height = 0.0;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.scope(|ui| {
                style(ui.style_mut());
                height = ui.button("Open").rect.height();
            });
        });
        assert_eq!(height, egui::Style::default().spacing.interact_size.y);
    }
}
