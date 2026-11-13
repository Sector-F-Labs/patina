//! Drawing a [`Menus`]: egui's menus in the title bar, or the platform's own.

use super::model::{Entry, Mark, Menus};
use egui::{Button, Id, Ui};

fn native_id() -> Id {
    Id::new("patina_native_menus")
}

/// Whether menus are drawn by the platform (macOS's menu bar) and not in the
/// window. True on macOS unless the app turned it off with [`set_native`];
/// false everywhere else. A [`TitleBar`](crate::frame::TitleBar) has nothing
/// to hold on the left then, and apps can ask to leave room out.
pub fn is_native(ctx: &egui::Context) -> bool {
    cfg!(target_os = "macos")
        && ctx
            .data(|d| d.get_temp::<bool>(native_id()))
            .unwrap_or(true)
}

/// Draws menus in the window even where the platform has its own (`false`),
/// or goes back to the platform's (`true`, the default). Does nothing off macOS.
pub fn set_native(ctx: &egui::Context, native: bool) {
    ctx.data_mut(|d| d.insert_temp(native_id(), native));
}

impl<A: Clone + 'static> Menus<A> {
    /// Draws the menus where the platform keeps them and returns the action
    /// of the item clicked, or whose shortcut was pressed, this frame. Call
    /// it every frame, in the title bar's left slot: egui draws the menus
    /// there, and on macOS this puts them in the menu bar and draws nothing.
    pub fn show(&self, ui: &mut Ui) -> Option<A> {
        #[cfg(target_os = "macos")]
        if is_native(ui.ctx()) {
            return super::native::show(ui.ctx(), self);
        }
        self.show_in_window(ui)
    }

    /// As [`show`](Self::show), always as egui menus in the window.
    pub fn show_in_window(&self, ui: &mut Ui) -> Option<A> {
        let mut out = None;
        for menu in &self.menus {
            ui.menu_button(&menu.label, |ui| draw(ui, &menu.entries, &mut out));
        }
        // Shortcuts of items that aren't open on screen.
        let ctx = ui.ctx().clone();
        if out.is_none() {
            let mut all = Vec::new();
            for menu in &self.menus {
                collect(&menu.entries, &mut all);
            }
            out = ctx.input_mut(|i| {
                all.into_iter()
                    .find(|(s, _)| i.consume_shortcut(s))
                    .map(|(_, a)| a.clone())
            });
        }
        out
    }
}

/// The enabled items that have a shortcut, longest modifier set first so
/// ⌘⇧N isn't taken for ⌘N.
fn collect<'a, A>(entries: &'a [Entry<A>], into: &mut Vec<(egui::KeyboardShortcut, &'a A)>) {
    for e in entries {
        match e {
            Entry::Item {
                action,
                shortcut: Some(s),
                enabled: true,
                ..
            } => into.push((*s, action)),
            Entry::Submenu {
                entries,
                enabled: true,
                ..
            } => collect(entries, into),
            _ => {}
        }
    }
    into.sort_by_key(|(s, _)| {
        let m = s.modifiers;
        std::cmp::Reverse(
            u8::from(m.alt)
                + u8::from(m.ctrl)
                + u8::from(m.shift)
                + u8::from(m.command || m.mac_cmd),
        )
    });
}

fn draw<A: Clone>(ui: &mut Ui, entries: &[Entry<A>], out: &mut Option<A>) {
    for entry in entries {
        match entry {
            Entry::Separator => {
                ui.separator();
            }
            Entry::Submenu {
                label,
                enabled,
                entries,
            } => {
                ui.add_enabled_ui(*enabled, |ui| {
                    ui.menu_button(label, |ui| draw(ui, entries, out));
                });
            }
            Entry::Item {
                label,
                action,
                shortcut,
                enabled,
                mark,
                ..
            } => {
                let clicked = ui
                    .add_enabled_ui(*enabled, |ui| match mark {
                        Mark::Check(on) => ui.checkbox(&mut { *on }, label).clicked(),
                        Mark::Radio(on) => ui.radio(*on, label).clicked(),
                        Mark::None => {
                            let mut b = Button::new(label);
                            if let Some(s) = shortcut {
                                b = b.shortcut_text(ui.ctx().format_shortcut(s));
                            }
                            ui.add(b).clicked()
                        }
                    })
                    .inner;
                if clicked {
                    *out = Some(action.clone());
                    ui.close();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Key, KeyboardShortcut as K, Modifiers as M, RawInput};

    #[derive(Clone, Debug, PartialEq)]
    enum A {
        Open,
        OpenRecent,
        Off,
    }

    fn menus() -> Menus<A> {
        Menus::new().menu(
            "File",
            [
                Entry::item("Open", A::Open).shortcut(K::new(M::COMMAND, Key::O)),
                Entry::submenu(
                    "Recent",
                    [Entry::item("Again", A::OpenRecent)
                        .shortcut(K::new(M::COMMAND | M::SHIFT, Key::O))],
                ),
                Entry::item("Off", A::Off)
                    .shortcut(K::new(M::COMMAND, Key::P))
                    .enabled(false),
            ],
        )
    }

    fn press(key: Key, modifiers: M) -> Option<A> {
        let ctx = egui::Context::default();
        let mut out = None;
        let input = RawInput {
            modifiers,
            events: vec![Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| out = menus().show_in_window(ui));
        out
    }

    #[test]
    fn a_shortcut_gives_its_items_action() {
        assert_eq!(press(Key::O, M::COMMAND), Some(A::Open));
    }

    #[test]
    fn a_submenus_shortcut_works_and_the_longer_one_wins() {
        assert_eq!(press(Key::O, M::COMMAND | M::SHIFT), Some(A::OpenRecent));
    }

    #[test]
    fn a_disabled_items_shortcut_does_nothing() {
        assert_eq!(press(Key::P, M::COMMAND), None);
    }

    #[test]
    fn menus_are_not_native_off_macos() {
        let ctx = egui::Context::default();
        assert_eq!(is_native(&ctx), cfg!(target_os = "macos"));
        set_native(&ctx, false);
        assert!(!is_native(&ctx));
    }
}
