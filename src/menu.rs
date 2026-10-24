//! Menus as every Patina app draws them (ADR-0022): egui's menu style, its
//! flat buttons lit under the pointer, with each item [`ITEM`] high, so no
//! item is smaller than the 24×24 Patina keeps every clickable element to
//! (`docs/design.md`).
//!
//! egui's `Response::context_menu` keeps its own style, so apps open theirs
//! with [`context`]; a menu drawn some other way, as on the Patina desktop,
//! takes [`style`].

use egui::{InnerResponse, Popup, Response, Style, Ui};

/// The height of a menu's item, and the least a clickable element should
/// be either way.
pub const ITEM: f32 = 24.0;

/// egui's menu style with Patina's item height. Checkboxes and radio buttons
/// in the menu are as tall as its buttons.
pub fn style(style: &mut Style) {
    egui::containers::menu::menu_style(style);
    style.spacing.interact_size.y = ITEM;
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
    fn a_menu_item_is_24_high() {
        let ctx = egui::Context::default();
        let mut height = 0.0;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.scope(|ui| {
                style(ui.style_mut());
                height = ui.button("Open").rect.height();
            });
        });
        assert_eq!(height, ITEM);
    }
}
