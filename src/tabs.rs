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
    /// Open a new tab: `+`, a tab's menu, or Ctrl+T (Ctrl+Shift+T in a
    /// terminal, [`terminal_keys`]).
    New,
    /// Show this tab: a click, Ctrl+PageUp/PageDown, or Alt+1 to Alt+9.
    Show(usize),
    /// Close this tab: its cross, a middle click, its menu, or Ctrl+W for
    /// the shown one (Ctrl+Shift+W in a terminal).
    Close(usize),
}

/// What a tab shows: its title, its icon, or both (ADR-0018). Without
/// [`TabStrip::icons`], always the title.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TabLabel {
    #[default]
    Title,
    Icon,
    IconAndTitle,
}

/// The side of a tab's close button.
const CLOSE: f32 = 18.0;
/// The width of the `+` tab.
const PLUS: f32 = 32.0;
/// The side of a tab's icon, and the space after it.
const ICON: f32 = 18.0;
const ICON_GAP: f32 = 6.0;
/// The width of a tab showing only its icon.
const ICON_TAB: f32 = 44.0;

/// What's drawn before a tab's title: an icon, or, where there's none, the
/// title's first letter in its place.
#[derive(Clone, Copy)]
enum Mark {
    Icon(egui::TextureId),
    Letter(char),
}

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
    closable: bool,
    noun: &'a str,
    icons: Option<&'a [Option<egui::TextureId>]>,
    label: TabLabel,
    menu: Option<&'a mut TabMenu<'a>>,
    own_menu: Option<&'a mut OwnMenu<'a>>,
    own_tooltip: Option<&'a mut OwnTooltip<'a>>,
    moved: Option<&'a mut Moved<'a>>,
    launchers: &'a [bool],
    separator: bool,
}

/// Adds items to tab `usize`'s menu (see [`TabStrip::tab_menu`]).
pub type TabMenu<'a> = dyn FnMut(&mut egui::Ui, usize) + 'a;
/// Opens tab `usize`'s menu at the pointer (see [`TabStrip::own_menu`]).
pub type OwnMenu<'a> = dyn FnMut(usize, egui::Pos2) + 'a;
/// Shows tab `usize`'s title beside its `Rect` (see [`TabStrip::own_tooltip`]).
pub type OwnTooltip<'a> = dyn FnMut(usize, Rect) + 'a;
/// Moves tab `from` to `to` (see [`TabStrip::movable`]).
pub type Moved<'a> = dyn FnMut(usize, usize) + 'a;

impl<'a, T: AsRef<str>> TabStrip<'a, T> {
    pub fn new(titles: &'a [T], shown: usize, style: &FrameStyle) -> Self {
        Self {
            titles,
            shown,
            bar: style.title_bar,
            span: None,
            id: Id::new("patina_tabs"),
            new_hint: "New Tab (Ctrl+T)",
            new_tab: true,
            closable: true,
            noun: "Tab",
            icons: None,
            label: TabLabel::Title,
            menu: None,
            own_menu: None,
            own_tooltip: None,
            moved: None,
            launchers: &[],
            separator: true,
        }
    }

    /// Each tab's icon, by index, `None` where it has none (yet): drawn
    /// before the title, as [`TabStrip::label`] says. A tab without one
    /// shows its title's first letter in its place, so the tabs line up.
    pub fn icons(mut self, icons: &'a [Option<egui::TextureId>]) -> Self {
        self.icons = Some(icons);
        self
    }

    /// Whether tabs show their icon, their title, or both; the title alone
    /// by default, and without [`TabStrip::icons`]. Icon-only tabs are
    /// narrow, have no cross (a middle click or the menu still closes
    /// them), and show their title on hover.
    pub fn label(mut self, label: TabLabel) -> Self {
        self.label = label;
        self
    }

    /// Tabs that start something rather than show it, by index (`false`
    /// past the end), as a desktop bar's pinned apps that aren't open
    /// (ADR-0021): only their icon whatever the label, faded, with no
    /// cross, and never filled as the shown tab. A click on one is still a
    /// `TabAction::Show`, for the app to start what it stands for.
    pub fn launchers(mut self, launchers: &'a [bool]) -> Self {
        self.launchers = launchers;
        self
    }

    fn launcher(&self, i: usize) -> bool {
        self.launchers.get(i).copied().unwrap_or(false)
    }

    /// The tabs can be dragged along the strip to put them in another
    /// order (ADR-0021): a line marks where a dragged tab would go, and on
    /// the drop this is called with the tab and its new index, as
    /// `Vec::remove` then `Vec::insert` would place it. The app reorders
    /// its tabs; a drop where the tab already was calls nothing.
    pub fn movable(mut self, moved: &'a mut Moved<'a>) -> Self {
        self.moved = Some(moved);
        self
    }

    /// What tab `i` draws before its title, and whether it shows the title.
    fn content(&self, i: usize, title: &str) -> (Option<Mark>, bool) {
        let label = if self.launcher(i) {
            TabLabel::Icon
        } else if self.icons.is_some() {
            self.label
        } else {
            TabLabel::Title
        };
        let mark = || match self.icons.and_then(|icons| icons.get(i).copied().flatten()) {
            Some(texture) => Mark::Icon(texture),
            None => Mark::Letter(title.chars().next().map_or('?', |c| c.to_uppercase().next().unwrap_or(c))),
        };
        match label {
            TabLabel::Title => (None, true),
            TabLabel::Icon => (Some(mark()), false),
            TabLabel::IconAndTitle => (Some(mark()), true),
        }
    }

    /// Whether a tab can be closed, from its cross, a middle click or its
    /// menu: off for tabs that are views the app always has, such as a
    /// messenger's chat folders (ADR-0014). `TabAction::Close` then never
    /// comes from the strip.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Whether to end with the `+` tab and offer "New" in a tab's menu:
    /// off for tabs the app doesn't create, such as a desktop bar's list of
    /// open windows (ADR-0012).
    pub fn new_tab(mut self, new_tab: bool) -> Self {
        self.new_tab = new_tab;
        self
    }

    /// More items for each tab's right-click menu, before Close: called
    /// with the menu and the tab's index, as a window list's "New Window"
    /// for that tab's app. Close the menu with `ui.close()` after acting.
    pub fn tab_menu(mut self, menu: &'a mut TabMenu<'a>) -> Self {
        self.menu = Some(menu);
        self
    }

    /// The app opens each tab's menu itself, as a bar too small to hold
    /// one does (in a menu of its own on the screen): a right click calls
    /// this with the tab and the pointer, and the strip draws no menu.
    pub fn own_menu(mut self, open: &'a mut OwnMenu<'a>) -> Self {
        self.own_menu = Some(open);
        self
    }

    /// The app shows a hovered tab's title itself, as a bar too small to
    /// hold a tooltip does: called each frame a tab is hovered, with the tab
    /// and its rectangle, and the strip shows no tooltip of its own.
    pub fn own_tooltip(mut self, show: &'a mut OwnTooltip<'a>) -> Self {
        self.own_tooltip = Some(show);
        self
    }

    /// Whether a line sets the tabs off from what's before them, as from a
    /// title bar's menus; on by default. Off for tabs that stand alone, as
    /// a window list of its own does.
    pub fn separator(mut self, separator: bool) -> Self {
        self.separator = separator;
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

    pub fn show(mut self, ui: &mut egui::Ui) -> Option<TabAction> {
        let mut menu = self.menu.take();
        let mut own_menu = self.own_menu.take();
        let mut own_tooltip = self.own_tooltip.take();
        let mut moved = self.moved.take();
        let global = ui.ctx().global_style();
        let shown = if self.bar == global.visuals.panel_fill { global.visuals.extreme_bg_color } else { global.visuals.panel_fill };
        // The menus' text colour, and the panel's for the shown tab.
        let ink = ui.visuals().widgets.inactive.text_color();
        let mut action = None;
        if self.separator {
            ui.separator();
        }
        ui.spacing_mut().item_spacing.x = 0.0;
        let launchers = (0..self.titles.len()).filter(|&i| self.launcher(i)).count();
        let n = (self.titles.len() - launchers).max(1) as f32;
        let plus = if self.new_tab { PLUS } else { 0.0 };
        let icons_only = self.icons.is_some() && self.label == TabLabel::Icon;
        let width = if icons_only { ICON_TAB } else { ((ui.available_width() - plus - launchers as f32 * ICON_TAB) / n).clamp(48.0, 200.0) };
        let close_label = format!("Close {}", self.noun);
        let sense = if moved.is_some() { egui::Sense::click_and_drag() } else { egui::Sense::click() };
        // Each tab's place, and the one being dragged, if it was let go.
        let mut places = Vec::with_capacity(self.titles.len());
        let mut dragged = None;
        for (i, title) in self.titles.iter().enumerate() {
            let title = title.as_ref();
            let launcher = self.launcher(i);
            let (fill, ink) = if i == self.shown && !launcher { (Some(shown), global.visuals.strong_text_color()) } else { (None, ink) };
            let (id, close_id) = self.tab_id(i);
            let (mark, titled) = self.content(i, title);
            let text = if titled { title } else { "" };
            let crossed = self.closable && !icons_only && !launcher;
            let look = Look { mark, text, fill, ink, closable: crossed, faded: launcher };
            let width = if launcher { ICON_TAB } else { width };
            let response = tab(ui, id, width, self.span, sense, look);
            places.push(response.rect);
            if response.dragged() || response.drag_stopped() {
                dragged = Some((i, response.drag_stopped()));
            }
            let response = match own_tooltip.as_mut() {
                Some(show) => {
                    if response.hovered() {
                        show(i, response.rect);
                    }
                    response
                }
                None => response.on_hover_text(title),
            };
            // Hovered, a tab has a close button, after its own registration so it wins.
            if crossed && ui.rect_contains_pointer(response.rect) && close(ui, close_id, response.rect, ink, &close_label) {
                action = Some(TabAction::Close(i));
            } else if response.clicked() {
                action = Some(TabAction::Show(i));
            }
            if self.closable && response.middle_clicked() {
                action = Some(TabAction::Close(i));
            }
            if let Some(open) = own_menu.as_mut() {
                if response.secondary_clicked()
                    && let Some(at) = response.interact_pointer_pos()
                {
                    open(i, at);
                }
            } else if self.new_tab || self.closable || menu.is_some() {
                response.context_menu(|ui| {
                    if self.new_tab && ui.button(format!("New {}", self.noun)).clicked() {
                        action = Some(TabAction::New);
                        ui.close();
                    }
                    if let Some(menu) = menu.as_mut() {
                        menu(ui, i);
                    }
                    if self.closable && ui.button(&close_label).clicked() {
                        action = Some(TabAction::Close(i));
                        ui.close();
                    }
                });
            }
        }
        if let (Some(moved), Some((from, dropped))) = (moved.as_mut(), dragged)
            && let Some(pointer) = ui.ctx().pointer_latest_pos()
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            let horizontal = ui.layout().is_horizontal();
            let along = |p: egui::Pos2| if horizontal { p.x } else { p.y };
            // The gap the pointer is nearest: before the first tab whose
            // middle is past it, or after the last.
            let gap = places.iter().filter(|r| along(r.center()) < along(pointer)).count();
            let to = if gap > from { gap - 1 } else { gap };
            if to != from {
                if dropped {
                    moved(from, to);
                } else {
                    let line = egui::Stroke::new(2.0, global.visuals.strong_text_color());
                    // Moving, there are tabs: the line is before the next, or after the last.
                    let (next, last) = (places.get(gap), places[places.len() - 1]);
                    match (horizontal, next) {
                        (true, Some(next)) => ui.painter().vline(next.left(), next.y_range(), line),
                        (true, None) => ui.painter().vline(last.right(), last.y_range(), line),
                        (false, Some(next)) => ui.painter().hline(next.x_range(), next.top(), line),
                        (false, None) => ui.painter().hline(last.x_range(), last.bottom(), line),
                    };
                }
            }
        }
        if self.new_tab {
            let look = Look { mark: None, text: "+", fill: None, ink, closable: false, faded: false };
            if tab(ui, self.id.with("new"), PLUS, self.span, egui::Sense::click(), look).on_hover_text(self.new_hint).clicked() {
                action = Some(TabAction::New);
            }
        }
        action
    }
}

/// What a tab draws: its `mark` and `text`, centred together, in `ink`, on
/// `fill` (else lit when hovered), clear of a close button if `closable`;
/// `faded`, a launcher's.
struct Look<'t> {
    mark: Option<Mark>,
    text: &'t str,
    fill: Option<Color32>,
    ink: Color32,
    closable: bool,
    faded: bool,
}

/// Ctrl+T for a new tab, Ctrl+W to close the shown one, Ctrl+PageUp/PageDown
/// for the tab before or after `shown`, of `count`, wrapping round, and
/// Alt+1 to Alt+9 for that tab (ADR-0020). Ctrl+T and Ctrl+W are taken only
/// without Shift, which a browser keeps for reopening a tab. The keys are
/// consumed, so call this before the widgets that would otherwise get them.
/// An Alt+digit is taken, with the digit egui sends after it as text, only
/// when that tab exists, so past the last tab it still reaches the app.
///
/// On the Patina desktop, Super+T and Super+W arrive as these keys.
pub fn keys(ctx: &egui::Context, count: usize, shown: usize) -> Option<TabAction> {
    keys_with(ctx, count, shown, egui::Modifiers::CTRL)
}

/// [`keys`] for a terminal, whose Ctrl+T and Ctrl+W belong to the shell:
/// Ctrl+Shift+T and Ctrl+Shift+W instead. Alt+digit past the last tab still
/// reaches the shell (readline's numeric argument).
pub fn terminal_keys(ctx: &egui::Context, count: usize, shown: usize) -> Option<TabAction> {
    keys_with(ctx, count, shown, egui::Modifiers::CTRL | egui::Modifiers::SHIFT)
}

/// The tab keys, with `new_close` held for the new and close keys.
fn keys_with(ctx: &egui::Context, count: usize, shown: usize, new_close: egui::Modifiers) -> Option<TabAction> {
    const DIGITS: [egui::Key; 9] = {
        use egui::Key::*;
        [Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9]
    };
    ctx.input_mut(|i| {
        let mut action = None;
        if consume_exact(i, new_close, egui::Key::T) {
            action = Some(TabAction::New);
        }
        if count > 0 && consume_exact(i, new_close, egui::Key::W) {
            action = Some(TabAction::Close(shown.min(count - 1)));
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

/// Like `consume_key`, but Shift and Alt must match too, so Ctrl+T isn't
/// taken for Ctrl+Shift+T.
fn consume_exact(i: &mut egui::InputState, modifiers: egui::Modifiers, key: egui::Key) -> bool {
    let before = i.events.len();
    i.events.retain(|e| {
        !matches!(e, egui::Event::Key { key: k, pressed: true, modifiers: m, .. } if *k == key && m.matches_exact(modifiers))
    });
    i.events.len() != before
}

/// A tab: `width` wide in the row, `span` tall, square whatever the skin's
/// rounding, drawn as `look` says. A closable tab wide enough for a close
/// button keeps its text clear of one.
fn tab(ui: &mut egui::Ui, id: Id, width: f32, span: Option<egui::Rangef>, sense: egui::Sense, look: Look) -> egui::Response {
    let Look { mark, text, fill, ink, closable, faded } = look;
    let (row, _) = ui.allocate_exact_size(egui::vec2(width, ui.spacing().interact_size.y), egui::Sense::hover());
    let rect = Rect::from_x_y_ranges(row.x_range(), span.unwrap_or(row.y_range()));
    let response = ui.interact(rect, id, sense);
    // A launcher's icon, or letter, is half there.
    let tint = if faded { Color32::WHITE.gamma_multiply(0.5) } else { Color32::WHITE };
    let painter = ui.painter().with_clip_rect(rect);
    // The pointer, not `hovered`: over the close button the tab isn't hovered, but is still lit.
    if let Some(fill) = fill.or_else(|| ui.rect_contains_pointer(rect).then(|| ui.visuals().widgets.hovered.weak_bg_fill)) {
        painter.rect_filled(rect, 0.0, fill);
    }
    let room = if closable && width > 4.0 * CLOSE { width - 2.0 * CLOSE - 8.0 } else { width - 16.0 };
    let (lead, room) = match (mark, text.is_empty()) {
        (None, _) => (0.0, room),
        (Some(_), true) => (ICON, room),
        (Some(_), false) => (ICON + ICON_GAP, room - ICON - ICON_GAP),
    };
    // Coloured here: the bar's override colour would win over the painter's.
    let galley = egui::WidgetText::from(egui::RichText::new(text).color(ink)).into_galley(ui, Some(egui::TextWrapMode::Truncate), room.max(0.0), egui::TextStyle::Body);
    let whole = lead + if text.is_empty() { 0.0 } else { galley.size().x };
    let left = rect.center().x - whole / 2.0;
    if let Some(mark) = mark {
        // On whole pixels, so the icon's texels land on the screen's: half a
        // pixel off (a 41 px bar's middle is 20.5) and it's drawn blurred.
        let corner = egui::pos2(left, rect.center().y - ICON / 2.0);
        let corner = egui::emath::GuiRounding::round_to_pixels(corner, ui.ctx().pixels_per_point());
        let at = Rect::from_min_size(corner, egui::Vec2::splat(ICON));
        match mark {
            Mark::Icon(texture) => {
                let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                painter.image(texture, at, uv, tint);
            }
            Mark::Letter(letter) => {
                painter.rect_filled(at, 4.0, ui.visuals().widgets.inactive.bg_fill.gamma_multiply(tint.a() as f32 / 255.0));
                painter.text(at.center(), egui::Align2::CENTER_CENTER, letter, egui::FontId::proportional(ICON * 0.6), ink.gamma_multiply(tint.a() as f32 / 255.0));
            }
        }
    }
    if !text.is_empty() {
        painter.galley(egui::pos2(left + lead, rect.center().y - galley.size().y / 2.0), galley, ink);
    }
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
        bar_with(ctx, events, true, true)
    }

    fn bar_with(ctx: &egui::Context, events: Vec<Event>, new_tab: bool, closable: bool) -> Option<TabAction> {
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
                TitleBar::new("", &style).menu_bar(true).show(ui, |ui| action = TabStrip::new(&TITLES, 1, &style).span(span).new_tab(new_tab).closable(closable).show(ui), |_| {});
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
        bar_with(&ctx, vec![], false, true);
        let last = ctx.read_response(ids(1).0).unwrap().rect;
        let at = pos2(last.right() + PLUS / 2.0, last.center().y);
        bar_with(&ctx, vec![Event::PointerMoved(at)], false, true);
        bar_with(&ctx, vec![button(at, true)], false, true);
        assert_eq!(bar_with(&ctx, vec![button(at, false)], false, true), None);
        assert!(ctx.read_response(Id::new("patina_tabs").with("new")).is_none());
    }

    #[test]
    fn a_tabs_menu_has_the_apps_own_items_for_that_tab() {
        let ctx = egui::Context::default();
        let shown = std::cell::Cell::new(None);
        let chosen = std::cell::Cell::new(None);
        let frame = |events| {
            let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
            let time = TIME.with(|t| {
                t.set(t.get() + 0.05);
                t.get()
            });
            let input = RawInput { screen_rect: Some(screen), events, time: Some(time), ..Default::default() };
            let _ = ctx.run_ui(input, |ui| {
                let style = FrameStyle::from_visuals(ui.visuals());
                let mut menu = |ui: &mut egui::Ui, i: usize| {
                    let item = ui.button("New Window");
                    shown.set(Some((i, item.rect)));
                    if item.clicked() {
                        chosen.set(Some(i));
                        ui.close();
                    }
                };
                TabStrip::new(&TITLES, 1, &style).new_tab(false).closable(false).tab_menu(&mut menu).show(ui);
            });
        };
        frame(vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        let right = |pressed| Event::PointerButton { pos: tab.center(), button: PointerButton::Secondary, pressed, modifiers: Default::default() };
        frame(vec![Event::PointerMoved(tab.center())]);
        frame(vec![right(true)]);
        frame(vec![right(false)]);
        frame(vec![]);
        let (i, item) = shown.get().expect("a right click opens the tab's menu, with the app's item");
        assert_eq!(i, 0, "for the tab clicked");
        frame(vec![Event::PointerMoved(item.center())]);
        frame(vec![button(item.center(), true)]);
        frame(vec![button(item.center(), false)]);
        assert_eq!(chosen.get(), Some(0));
    }

    #[test]
    fn an_app_can_open_a_tabs_menu_itself() {
        let ctx = egui::Context::default();
        let asked = std::cell::Cell::new(None);
        let frame = |events| {
            let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
            let time = TIME.with(|t| {
                t.set(t.get() + 0.05);
                t.get()
            });
            let input = RawInput { screen_rect: Some(screen), events, time: Some(time), ..Default::default() };
            let _ = ctx.run_ui(input, |ui| {
                let style = FrameStyle::from_visuals(ui.visuals());
                let mut open = |i: usize, at: egui::Pos2| asked.set(Some((i, at)));
                TabStrip::new(&TITLES, 1, &style).new_tab(false).own_menu(&mut open).show(ui);
            });
        };
        frame(vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        let right = |pressed| Event::PointerButton { pos: tab.center(), button: PointerButton::Secondary, pressed, modifiers: Default::default() };
        frame(vec![Event::PointerMoved(tab.center())]);
        frame(vec![right(true)]);
        frame(vec![right(false)]);
        assert_eq!(asked.get(), Some((0, tab.center())), "the tab and where it was clicked");
        frame(vec![]);
        assert!(!egui::Popup::is_any_open(&ctx), "and no menu of the strip's own");
    }

    #[test]
    fn an_app_can_show_a_hovered_tabs_title_itself() {
        let ctx = egui::Context::default();
        let hovered = std::cell::Cell::new(None);
        let frame = |events| {
            let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
            let input = RawInput { screen_rect: Some(screen), events, ..Default::default() };
            let _ = ctx.run_ui(input, |ui| {
                let style = FrameStyle::from_visuals(ui.visuals());
                let mut show = |i: usize, rect: Rect| hovered.set(Some((i, rect)));
                TabStrip::new(&TITLES, 1, &style).own_tooltip(&mut show).show(ui);
            });
        };
        frame(vec![]);
        let tab = ctx.read_response(ids(1).0).unwrap().rect;
        frame(vec![Event::PointerMoved(tab.center())]);
        assert_eq!(hovered.get(), Some((1, tab)));
    }

    #[test]
    fn a_tab_that_cant_close_has_no_cross_and_ignores_a_middle_click() {
        let ctx = egui::Context::default();
        let frame = |events| bar_with(&ctx, events, false, false);
        frame(vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        frame(vec![Event::PointerMoved(tab.center())]);
        assert!(ctx.read_response(ids(0).1).is_none(), "no cross on a hovered tab");
        let middle = |pressed| Event::PointerButton {
            pos: tab.center(),
            button: PointerButton::Middle,
            pressed,
            modifiers: Default::default(),
        };
        frame(vec![middle(true)]);
        assert_eq!(frame(vec![middle(false)]), None);
        // A click still shows it.
        frame(vec![button(tab.center(), true)]);
        assert_eq!(frame(vec![button(tab.center(), false)]), Some(TabAction::Show(0)));
    }

    /// A frame of two tabs with icons, the first one's not loaded, shown
    /// as `label`.
    fn iconic(ctx: &egui::Context, label: TabLabel, events: Vec<Event>) -> Option<TabAction> {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        let time = TIME.with(|t| {
            t.set(t.get() + 0.05);
            t.get()
        });
        let input = RawInput { screen_rect: Some(screen), events, time: Some(time), ..Default::default() };
        let icons = [None, Some(egui::TextureId::Managed(0))];
        let mut action = None;
        let _ = ctx.run_ui(input, |ui| {
            let style = FrameStyle::from_visuals(ui.visuals());
            action = TabStrip::new(&TITLES, 1, &style).icons(&icons).label(label).new_tab(false).show(ui);
        });
        action
    }

    #[test]
    fn icon_only_tabs_are_narrow_and_have_no_cross() {
        let ctx = egui::Context::default();
        iconic(&ctx, TabLabel::Icon, vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        assert_eq!(tab.width(), ICON_TAB);
        iconic(&ctx, TabLabel::Icon, vec![Event::PointerMoved(tab.center())]);
        assert!(ctx.read_response(ids(0).1).is_none(), "no cross over the icon");
        let middle = |pressed| Event::PointerButton { pos: tab.center(), button: PointerButton::Middle, pressed, modifiers: Default::default() };
        iconic(&ctx, TabLabel::Icon, vec![middle(true)]);
        assert_eq!(iconic(&ctx, TabLabel::Icon, vec![middle(false)]), Some(TabAction::Close(0)), "a middle click still closes");
    }

    #[test]
    fn tabs_with_titles_keep_their_width_and_cross() {
        let ctx = egui::Context::default();
        iconic(&ctx, TabLabel::IconAndTitle, vec![]);
        let tab = ctx.read_response(ids(0).0).unwrap().rect;
        assert_eq!(tab.width(), 200.0);
        iconic(&ctx, TabLabel::IconAndTitle, vec![Event::PointerMoved(tab.center())]);
        assert!(ctx.read_response(ids(0).1).is_some());
    }

    #[test]
    fn a_missing_icon_is_the_titles_first_letter() {
        let strip_icons = [None, Some(egui::TextureId::Managed(3))];
        let style = FrameStyle::from_visuals(&egui::Visuals::dark());
        let strip = TabStrip::new(&TITLES, 0, &style).icons(&strip_icons).label(TabLabel::IconAndTitle);
        assert!(matches!(strip.content(0, "one"), (Some(Mark::Letter('O')), true)));
        assert!(matches!(strip.content(1, "two"), (Some(Mark::Icon(_)), true)));
        let titles_only = TabStrip::new(&TITLES, 0, &style).label(TabLabel::Icon);
        assert!(matches!(titles_only.content(0, "one"), (None, true)), "no icons given: titles");
    }

    /// A frame of three tabs, the first a launcher, that can be dragged:
    /// what was moved where, if a tab was dropped.
    fn movable(ctx: &egui::Context, events: Vec<Event>) -> Option<(usize, usize)> {
        let screen = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        let time = TIME.with(|t| {
            t.set(t.get() + 0.05);
            t.get()
        });
        let input = RawInput { screen_rect: Some(screen), events, time: Some(time), ..Default::default() };
        let mut dropped = None;
        let _ = ctx.run_ui(input, |ui| {
            ui.horizontal(|ui| {
                let style = FrameStyle::from_visuals(ui.visuals());
                let mut moved = |from, to| dropped = Some((from, to));
                TabStrip::new(&["one", "two", "three"], 0, &style).new_tab(false).launchers(&[true]).movable(&mut moved).show(ui);
            });
        });
        dropped
    }

    /// Drags from `from` to `to` in steps, and lets go there.
    fn drag(ctx: &egui::Context, from: egui::Pos2, to: egui::Pos2) -> Option<(usize, usize)> {
        movable(ctx, vec![Event::PointerMoved(from)]);
        movable(ctx, vec![button(from, true)]);
        for step in 1..=4 {
            movable(ctx, vec![Event::PointerMoved(from.lerp(to, step as f32 / 4.0))]);
        }
        movable(ctx, vec![button(to, false)])
    }

    #[test]
    fn a_tab_dragged_past_another_moves_there() {
        let ctx = egui::Context::default();
        movable(&ctx, vec![]);
        let rect = |i| ctx.read_response(ids(i).0).unwrap().rect;
        assert_eq!(drag(&ctx, rect(1).center(), rect(2).right_center() - egui::vec2(4.0, 0.0)), Some((1, 2)));
        assert_eq!(drag(&ctx, rect(2).center(), rect(0).left_center() + egui::vec2(4.0, 0.0)), Some((2, 0)));
    }

    #[test]
    fn a_tab_dropped_where_it_was_stays() {
        let ctx = egui::Context::default();
        movable(&ctx, vec![]);
        let tab = ctx.read_response(ids(1).0).unwrap().rect;
        assert_eq!(drag(&ctx, tab.center(), tab.center() + egui::vec2(20.0, 0.0)), None);
        // And a click is still a click.
        movable(&ctx, vec![button(tab.center(), true)]);
        assert_eq!(movable(&ctx, vec![button(tab.center(), false)]), None);
    }

    #[test]
    fn a_launcher_is_only_its_icon_with_no_cross() {
        let ctx = egui::Context::default();
        movable(&ctx, vec![]);
        let launcher = ctx.read_response(ids(0).0).unwrap().rect;
        assert_eq!(launcher.width(), ICON_TAB);
        assert!(ctx.read_response(ids(1).0).unwrap().rect.width() > ICON_TAB, "the others have their titles");
        movable(&ctx, vec![Event::PointerMoved(launcher.center())]);
        assert!(ctx.read_response(ids(0).1).is_none());
        let style = FrameStyle::from_visuals(&egui::Visuals::dark());
        let strip = TabStrip::new(&TITLES, 0, &style).launchers(&[true]);
        assert!(matches!(strip.content(0, "one"), (Some(Mark::Letter('O')), false)), "without icons, its letter");
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
    }

    #[test]
    fn ctrl_t_and_w_in_an_app_with_shift_in_a_terminal() {
        let ctx = egui::Context::default();
        let ctrl = egui::Modifiers::CTRL;
        let ctrl_shift = egui::Modifiers::CTRL | egui::Modifiers::SHIFT;
        let press = |terminal: bool, key, modifiers| {
            let mut action = None;
            let events = vec![Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }];
            let _ = ctx.run_ui(RawInput { events, ..Default::default() }, |ui| {
                action = if terminal { terminal_keys(ui.ctx(), 3, 1) } else { keys(ui.ctx(), 3, 1) };
            });
            action
        };
        assert_eq!(press(false, egui::Key::T, ctrl), Some(TabAction::New));
        assert_eq!(press(false, egui::Key::W, ctrl), Some(TabAction::Close(1)));
        assert_eq!(press(false, egui::Key::T, ctrl_shift), None, "a browser's reopen");
        assert_eq!(press(false, egui::Key::W, ctrl_shift), None);
        assert_eq!(press(true, egui::Key::T, ctrl_shift), Some(TabAction::New));
        assert_eq!(press(true, egui::Key::W, ctrl_shift), Some(TabAction::Close(1)));
        assert_eq!(press(true, egui::Key::T, ctrl), None, "the shell's");
        assert_eq!(press(true, egui::Key::W, ctrl), None);
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
