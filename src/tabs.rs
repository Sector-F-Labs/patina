//! Tabs in the title bar, after the menus, in place of the window title.
//! Moved from Teleprinter (its ADR-0012); see ADR-0009.
//!
//! Call [`TabStrip::show`] at the end of the title bar's left slot, with an
//! empty title, and [`keys`] once a frame before the widgets that would
//! otherwise see the keys. Both return a [`TabAction`]; the app owns its
//! tabs and applies it.
//!
//! ```no_run
//! # use patina::frame::{self, FrameStyle, TitleBar};
//! # use patina::tabs::{self, TabStrip};
//! # fn f(ui: &mut egui::Ui, style: &FrameStyle, titles: &[String], shown: usize) {
//! let mut action = tabs::keys(ui.ctx(), titles.len(), shown);
//! let header = frame::header_frame(ui.style(), style, frame::WINDOW_RADIUS);
//! egui::Panel::top("header").frame(header).show(ui, |ui| {
//!     // The bar's full height, padding and all, for the tabs to fill.
//!     let span = (ui.max_rect() + header.inner_margin).y_range();
//!     TitleBar::new("", style).menu_bar(true).show(
//!         ui,
//!         |ui| {
//!             // ... the menus ...
//!             action = action.or(TabStrip::new(titles, shown, style).span(span).show(ui));
//!         },
//!         |_ui| {},
//!     );
//! });
//! # }
//! ```

use egui::{Color32, Id, Rect};

use crate::frame::FrameStyle;

/// What the user did to the tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
    /// Open a new tab: `+`, a tab's menu, or Ctrl+Shift+T.
    New,
    /// Show this tab: a click, Ctrl+PageUp/PageDown, or Alt+1 to Alt+9.
    Show(usize),
    /// Close this tab: its cross, a middle click, or its menu.
    Close(usize),
}

/// The side of a tab's close button.
const CLOSE: f32 = 18.0;
/// The width of the `+` tab.
const PLUS: f32 = 32.0;

/// The tabs, each named by its title, then `+` for a new one. They share
/// the space left in the row, up to 200 px each, and span the bar's height
/// ([`TabStrip::span`]). They're drawn as tabs, not buttons: square
/// whatever the skin's rounding. The shown one is filled in the panel
/// colour, joining it to the panel below; on a bar that's already that
/// colour, in the extreme background colour instead.
pub struct TabStrip<'a, T: AsRef<str>> {
    titles: &'a [T],
    shown: usize,
    bar: Color32,
    span: Option<egui::Rangef>,
    id: Id,
    new_hint: &'a str,
    new_tab: bool,
    noun: &'a str,
}

impl<'a, T: AsRef<str>> TabStrip<'a, T> {
    pub fn new(titles: &'a [T], shown: usize, style: &FrameStyle) -> Self {
        Self {
            titles,
            shown,
            bar: style.title_bar,
            span: None,
            id: Id::new("patina_tabs"),
            new_hint: "New Tab (Ctrl+Shift+T)",
            new_tab: true,
            noun: "Tab",
        }
    }

    /// Whether to end with the `+` tab and offer "New" in a tab's menu:
    /// off for tabs the app doesn't create, such as a desktop bar's list of
    /// open windows (ADR-0012).
    pub fn new_tab(mut self, new_tab: bool) -> Self {
        self.new_tab = new_tab;
        self
    }

    /// What a tab is, for the menu and the close button: "Close Tab" by
    /// default, "Close Window" with `noun("Window")`.
    pub fn noun(mut self, noun: &'a str) -> Self {
        self.noun = noun;
        self
    }

    /// How far the tabs reach up and down: the bar's full height, padding
    /// included, so a tab takes clicks at the window's top edge. Without
    /// it, the row's height.
    pub fn span(mut self, span: egui::Rangef) -> Self {
        self.span = Some(span);
        self
    }

    /// For more than one strip in a window.
    pub fn id_salt(mut self, salt: impl egui::AsId) -> Self {
        self.id = Id::new(salt);
        self
    }

    /// The `+` tab's tooltip.
    pub fn new_hint(mut self, hint: &'a str) -> Self {
        self.new_hint = hint;
        self
    }

    /// Tab `i`'s id, and its close button's while it's hovered.
    pub fn tab_id(&self, i: usize) -> (Id, Id) {
        (self.id.with(("tab", i)), self.id.with(("close", i)))
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<TabAction> {
        let global = ui.ctx().global_style();
        let shown = if self.bar == global.visuals.panel_fill { global.visuals.extreme_bg_color } else { global.visuals.panel_fill };
        // The menus' text colour, and the panel's for the shown tab.
        let ink = ui.visuals().widgets.inactive.text_color();
        let mut action = None;
        ui.separator();
        ui.spacing_mut().item_spacing.x = 0.0;
        let n = self.titles.len().max(1) as f32;
        let plus = if self.new_tab { PLUS } else { 0.0 };
        let width = ((ui.available_width() - plus) / n).clamp(48.0, 200.0);
        let close_label = format!("Close {}", self.noun);
        for (i, title) in self.titles.iter().enumerate() {
            let title = title.as_ref();
            let (fill, ink) = if i == self.shown { (Some(shown), global.visuals.strong_text_color()) } else { (None, ink) };
            let (id, close_id) = self.tab_id(i);
            let response = tab(ui, id, width, self.span, title, fill, ink).on_hover_text(title);
            // Hovered, a tab has a close button, after its own registration so it wins.
            if ui.rect_contains_pointer(response.rect) && close(ui, close_id, response.rect, ink, &close_label) {
                action = Some(TabAction::Close(i));
            } else if response.clicked() {
                action = Some(TabAction::Show(i));
            }
            if response.middle_clicked() {
                action = Some(TabAction::Close(i));
            }
            response.context_menu(|ui| {
                if self.new_tab && ui.button(format!("New {}", self.noun)).clicked() {
                    action = Some(TabAction::New);
                    ui.close();
                }
                if ui.button(&close_label).clicked() {
                    action = Some(TabAction::Close(i));
                    ui.close();
                }
            });
        }
        if self.new_tab && tab(ui, self.id.with("new"), PLUS, self.span, "+", None, ink).on_hover_text(self.new_hint).clicked() {
            action = Some(TabAction::New);
        }
        action
    }
}

/// Ctrl+Shift+T for a new tab, Ctrl+PageUp/PageDown for the tab before or
/// after `shown`, of `count`, wrapping round, and Alt+1 to Alt+9 for that
/// tab. The keys are consumed, so call this before the widgets that would
/// otherwise get them. An Alt+digit is taken, with the digit egui sends after
/// it as text, only when that tab exists, so past the last tab it still
/// reaches the app (readline's numeric argument, in a terminal).
pub fn keys(ctx: &egui::Context, count: usize, shown: usize) -> Option<TabAction> {
    const DIGITS: [egui::Key; 9] = {
        use egui::Key::*;
        [Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9]
    };
    ctx.input_mut(|i| {
        let mut action = None;
        if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::T) {
            action = Some(TabAction::New);
        }
        if count > 0 && i.consume_key(egui::Modifiers::CTRL, egui::Key::PageUp) {
            action = Some(TabAction::Show((shown + count - 1) % count));
        }
        if count > 0 && i.consume_key(egui::Modifiers::CTRL, egui::Key::PageDown) {
            action = Some(TabAction::Show((shown + 1) % count));
        }
        if let Some(n) = DIGITS.iter().take(count).position(|&key| i.consume_key(egui::Modifiers::ALT, key)) {
            let digit = (n + 1).to_string();
            i.events.retain(|e| !matches!(e, egui::Event::Text(t) if *t == digit));
            action = Some(TabAction::Show(n));
        }
        action
    })
}

/// A tab: `width` wide in the row, `span` tall, square whatever the skin's
/// rounding, filled with `fill`, else when hovered. A tab wide enough for a
/// close button keeps its text clear of one.
fn tab(ui: &mut egui::Ui, id: Id, width: f32, span: Option<egui::Rangef>, text: &str, fill: Option<Color32>, ink: Color32) -> egui::Response {
    let (row, _) = ui.allocate_exact_size(egui::vec2(width, ui.spacing().interact_size.y), egui::Sense::hover());
    let rect = Rect::from_x_y_ranges(row.x_range(), span.unwrap_or(row.y_range()));
    let response = ui.interact(rect, id, egui::Sense::click());
    let painter = ui.painter().with_clip_rect(rect);
    // The pointer, not `hovered`: over the close button the tab isn't hovered, but is still lit.
    if let Some(fill) = fill.or_else(|| ui.rect_contains_pointer(rect).then(|| ui.visuals().widgets.hovered.weak_bg_fill)) {
        painter.rect_filled(rect, 0.0, fill);
    }
    let room = if width > 4.0 * CLOSE { width - 2.0 * CLOSE - 8.0 } else { width - 16.0 };
    // Coloured here: the bar's override colour would win over the painter's.
    let galley = egui::WidgetText::from(egui::RichText::new(text).color(ink)).into_galley(ui, Some(egui::TextWrapMode::Truncate), room, egui::TextStyle::Body);
    painter.galley(rect.center() - galley.size() / 2.0, galley, ink);
    response
}

/// A tab's close button, a cross at its right end; whether it was clicked.
fn close(ui: &mut egui::Ui, id: Id, tab: Rect, ink: Color32, label: &str) -> bool {
    let rect = Rect::from_center_size(egui::pos2(tab.right() - 6.0 - CLOSE / 2.0, tab.center().y), egui::Vec2::splat(CLOSE));
    let response = ui.interact(rect, id, egui::Sense::click()).on_hover_text(label);
    if response.hovered() {
        ui.painter().rect_filled(rect, 0.0, ui.visuals().widgets.active.weak_bg_fill);
    }
    let (c, d) = (rect.center(), 4.0);
    let stroke = egui::Stroke::new(1.2, ink);
    ui.painter().line_segment([c + egui::vec2(-d, -d), c + egui::vec2(d, d)], stroke);
    ui.painter().line_segment([c + egui::vec2(-d, d), c + egui::vec2(d, -d)], stroke);
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{self, TitleBar};
    use egui::{Event, PointerButton, RawInput, pos2};
    use std::cell::Cell;

    const TITLES: [&str; 2] = ["one", "two"];

    thread_local! {
        static TIME: Cell<f64> = const { Cell::new(0.0) };
    }

    /// A frame of a title bar with two tabs, the second shown. Needs
    /// `time`, or every click lands in one multi-click burst.
    fn bar(ctx: &egui::Context, events: Vec<Event>) -> Option<TabAction> {
        bar_with(ctx, events, true)
    }

    fn bar_with(ctx: &egui::Context, events: Vec<Event>, new_tab: bool) -> Option<TabAction> {
        let time = TIME.with(|t| {
            t.set(t.get() + 0.05);
            t.get()
        });
        let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        let input = RawInput { screen_rect: Some(screen), events, time: Some(time), ..Default::default() };
        let mut action = None;
        let _ = ctx.run_ui(input, |ui| {
            let style = FrameStyle::from_visuals(ui.visuals());
            let header = frame::header_frame(ui.style(), &style, 0);
            egui::Panel::top("header").frame(header).show(ui, |ui| {
                let span = (ui.max_rect() + header.inner_margin).y_range();
                TitleBar::new("", &style).menu_bar(true).show(ui, |ui| action = TabStrip::new(&TITLES, 1, &style).span(span).new_tab(new_tab).show(ui), |_| {});
            });
        });
        action
    }

    fn ids(i: usize) -> (Id, Id) {
        TabStrip::new(&TITLES, 1, &FrameStyle::from_visuals(&egui::Visuals::dark())).tab_id(i)
    }

    fn button(pos: egui::Pos2, pressed: bool) -> Event {
        Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Default::default() }
    }

    fn click(ctx: &egui::Context, at: egui::Pos2) -> Option<TabAction> {
        bar(ctx, vec![Event::PointerMoved(at)]);
        bar(ctx, vec![button(at, true)]);
        bar(ctx, vec![button(at, false)])
    }

    #[test]
    fn a_tab_takes_clicks_from_the_top_of_the_bar() {
        let ctx = egui::Context::default();
        bar(&ctx, vec![]);
        let first = ctx.read_response(ids(0).0).unwrap().rect;
        assert_eq!(first.top(), 0.0, "the tab starts at the window's top edge");
        assert_eq!(click(&ctx, pos2(first.center().x, 2.0)), Some(TabAction::Show(0)));
    }

    #[test]
    fn a_hovered_tab_closes_from_its_cross() {
        let ctx = egui::Context::default();
        bar(&ctx, vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        bar(&ctx, vec![Event::PointerMoved(tab.center())]);
        let cross = ctx.read_response(ids(0).1).expect("hovered, the tab has a cross").rect;
        assert_eq!(click(&ctx, cross.center()), Some(TabAction::Close(0)));
    }

    #[test]
    fn plus_opens_a_tab() {
        let ctx = egui::Context::default();
        bar(&ctx, vec![]);
        let last = ctx.read_response(ids(1).0).unwrap().rect;
        assert_eq!(click(&ctx, pos2(last.right() + PLUS / 2.0, last.center().y)), Some(TabAction::New));
    }

    #[test]
    fn without_new_tabs_there_is_no_plus() {
        let ctx = egui::Context::default();
        bar_with(&ctx, vec![], false);
        let last = ctx.read_response(ids(1).0).unwrap().rect;
        let at = pos2(last.right() + PLUS / 2.0, last.center().y);
        bar_with(&ctx, vec![Event::PointerMoved(at)], false);
        bar_with(&ctx, vec![button(at, true)], false);
        assert_eq!(bar_with(&ctx, vec![button(at, false)], false), None);
        assert!(ctx.read_response(Id::new("patina_tabs").with("new")).is_none());
    }

    #[test]
    fn page_keys_cycle_round() {
        let ctx = egui::Context::default();
        let key = |key, modifiers| Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
        let cycle = |key_event, shown| {
            let mut action = None;
            let input = RawInput { events: vec![key_event], ..Default::default() };
            let _ = ctx.run_ui(input, |ui| action = keys(ui.ctx(), 3, shown));
            action
        };
        assert_eq!(cycle(key(egui::Key::PageUp, egui::Modifiers::CTRL), 0), Some(TabAction::Show(2)));
        assert_eq!(cycle(key(egui::Key::PageDown, egui::Modifiers::CTRL), 2), Some(TabAction::Show(0)));
        assert_eq!(cycle(key(egui::Key::T, egui::Modifiers::CTRL | egui::Modifiers::SHIFT), 0), Some(TabAction::New));
        assert_eq!(cycle(key(egui::Key::T, egui::Modifiers::CTRL), 0), None);
    }

    #[test]
    fn alt_and_a_number_shows_that_tab_and_takes_its_digit() {
        let ctx = egui::Context::default();
        let alt = |key| Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::ALT };
        let press = |events: Vec<Event>| {
            let mut out = (None, vec![]);
            let _ = ctx.run_ui(RawInput { events, ..Default::default() }, |ui| {
                out = (keys(ui.ctx(), 2, 0), ui.input(|i| i.events.clone()));
            });
            out
        };
        let (action, left) = press(vec![alt(egui::Key::Num2), Event::Text("2".into())]);
        assert_eq!(action, Some(TabAction::Show(1)));
        assert!(left.is_empty(), "{left:?}");
        // No third tab: Alt+3 and its digit go on to the app.
        let (action, left) = press(vec![alt(egui::Key::Num3), Event::Text("3".into())]);
        assert_eq!((action, left.len()), (None, 2));
    }
}
