//! The window frame every Patina app draws for itself: the window opens
//! without system decorations, and this draws a header bar holding the app's
//! own controls, the window title and minimise/maximise/close buttons, plus a
//! one-pixel border and invisible resize handles along the edges. Moved from
//! UnAmp (its ADR-0020; Photograph's ADR-0022), with the API in
//! `docs/adr/0003-frame-api.md`.
//!
//! Moving and resizing are handed to the compositor (`StartDrag`,
//! `BeginResize`), so snapping and tiling behave as with a native frame.
//!
//! An app wires it up in five places:
//!
//! 1. [`viewport`] on the `ViewportBuilder`, to drop the system frame;
//! 2. [`header_frame`] and [`TitleBar`] in the top panel;
//! 3. [`edges`] after everything else is drawn;
//! 4. [`clear_color`] from `App::clear_color`, so the rounded corners show the
//!    desktop;
//! 5. [`release_grab`] from `App::raw_input_hook`.

use egui::{Color32, CursorIcon, Id, Order, ResizeDirection, Sense, Stroke, ViewportCommand, vec2};

/// How far in from the window edge a press starts a resize.
const EDGE: f32 = 5.0;
/// Corners get a larger grab area than straight edges.
const CORNER: f32 = 12.0;
const BUTTON: egui::Vec2 = vec2(30.0, 22.0);
/// Height of the header bar's row, set by Photograph's GTK-style buttons.
const ROW_HEIGHT: f32 = 25.0;
/// Space between the app's right-hand controls and the window buttons.
const CONTROLS_GAP: f32 = 8.0;

/// libadwaita's `--window-radius`, as shipped in Ubuntu's Yaru styles too.
pub const WINDOW_RADIUS: u8 = 15;

/// The frame's colours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStyle {
    pub title_bar: Color32,
    pub title_text: Color32,
    pub border: Color32,
    /// Corner rounding of the window when it isn't maximised.
    pub radius: u8,
    pub controls: WindowControlColors,
}

/// Minimise, maximise and close.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowControlColors {
    pub icon: Color32,
    pub icon_hover: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub close_hover: Color32,
    pub close_icon_hover: Color32,
    pub radius: egui::CornerRadius,
}

impl FrameStyle {
    /// The frame drawn in the active egui visuals: panel colours for the
    /// bar, the widget colours for the window controls, and GNOME's red close
    /// button. A skin overrides what it sets and starts from this.
    pub fn from_visuals(v: &egui::Visuals) -> Self {
        Self {
            title_bar: v.panel_fill,
            title_text: v.text_color(),
            border: v.window_stroke.color,
            radius: WINDOW_RADIUS,
            controls: WindowControlColors {
                icon: v.widgets.inactive.fg_stroke.color,
                icon_hover: v.widgets.hovered.fg_stroke.color,
                hover: v.widgets.hovered.weak_bg_fill,
                pressed: v.widgets.active.weak_bg_fill,
                close_hover: Color32::from_rgb(0xc0, 0x1c, 0x28),
                close_icon_hover: Color32::WHITE,
                radius: v.widgets.hovered.corner_radius,
            },
        }
    }
}

/// Sets up the window for the app's own frame: no system decorations, and
/// transparent so the rounded corners show the desktop. With `custom` off
/// the desktop draws the frame as usual.
pub fn viewport(builder: egui::ViewportBuilder, custom: bool) -> egui::ViewportBuilder {
    builder.with_decorations(!custom).with_transparent(true)
}

/// For `App::clear_color`: transparent behind the app's own frame, so its
/// rounded corners show the desktop, and the panel colour otherwise. The
/// panels paint everything else.
pub fn clear_color(visuals: &egui::Visuals, custom: bool) -> [f32; 4] {
    if custom { [0.0; 4] } else { visuals.panel_fill.to_normalized_gamma_f32() }
}

fn maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| {
        let v = i.viewport();
        v.maximized.unwrap_or(false) || v.fullscreen.unwrap_or(false)
    })
}

/// Corner rounding for the window right now: `radius`, or square while
/// maximised or full screen, as GNOME does.
pub fn corner_radius(ctx: &egui::Context, radius: u8) -> u8 {
    if maximized(ctx) { 0 } else { radius }
}

/// The top panel's frame for a [`TitleBar`]: the bar's colour, the header-bar
/// padding the suite shares, and the window's top corners rounded by
/// `radius` (from [`corner_radius`]).
pub fn header_frame(style: &egui::Style, frame: &FrameStyle, radius: u8) -> egui::Frame {
    egui::Frame::side_top_panel(style)
        .fill(frame.title_bar)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .corner_radius(egui::CornerRadius { nw: radius, ne: radius, sw: 0, se: 0 })
}

/// The header bar: the app's controls on the left and right, the title
/// centred on the window, and with the custom frame on, window buttons at
/// the far right and drag-to-move on the empty space. With it off the
/// desktop draws the frame and this is a plain toolbar.
///
/// ```no_run
/// # fn f(ui: &mut egui::Ui, style: &patina::frame::FrameStyle) {
/// patina::frame::TitleBar::new("Photograph", style).show(
///     ui,
///     |ui| { ui.button("Render"); },
///     |ui| { ui.button("Edit"); },
/// );
/// # }
/// ```
pub struct TitleBar<'a> {
    title: &'a str,
    style: &'a FrameStyle,
    custom: bool,
    menu_bar: bool,
}

impl<'a> TitleBar<'a> {
    pub fn new(title: &'a str, style: &'a FrameStyle) -> Self {
        Self { title, style, custom: true, menu_bar: false }
    }

    /// Whether the app draws its own frame (the default). Off, the title and
    /// window buttons are left to the desktop's title bar.
    pub fn custom(mut self, custom: bool) -> Self {
        self.custom = custom;
        self
    }

    /// Lays the left-hand controls out as a menu bar, for apps whose left
    /// side is `menu_button`s.
    pub fn menu_bar(mut self, menu_bar: bool) -> Self {
        self.menu_bar = menu_bar;
        self
    }

    /// Draws the bar. `left` is laid out left to right from the left edge;
    /// `right` right to left, ending at the window buttons.
    pub fn show(self, ui: &mut egui::Ui, left: impl FnOnce(&mut egui::Ui), right: impl FnOnce(&mut egui::Ui)) {
        let Self { title, style, custom, menu_bar } = self;
        let ctx = ui.ctx().clone();
        let bar = ui.max_rect();
        if custom {
            // Registered before the controls and buttons, so they win where
            // they overlap and only empty space moves the window.
            let resp = ui.interact(bar, Id::new("title_bar_drag"), Sense::click_and_drag());
            if resp.drag_started_by(egui::PointerButton::Primary) {
                hand_to_compositor(&ctx, ViewportCommand::StartDrag);
            }
            if resp.double_clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized(&ctx)));
            }
            resp.context_menu(|ui| window_menu(ui, &ctx));
        }

        // Laid out side by side in areas of their own, so nothing can push
        // the row wider than the bar: the window buttons at the right first,
        // then the right-hand controls, then the left-hand ones in the width
        // that's left. (A menu bar takes its whole width, so sharing one row
        // with the buttons made the panel overflow the window by the spacing
        // between them, and its rounded corner with it.)
        let row = egui::Rect::from_min_size(bar.min, vec2(bar.width(), ROW_HEIGHT));
        let child = |ui: &mut egui::Ui, rect: egui::Rect, layout: egui::Layout| {
            ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout))
        };
        let mut buttons_left = row.right();
        if custom {
            let mut buttons = child(ui, row, egui::Layout::right_to_left(egui::Align::Center));
            buttons.spacing_mut().item_spacing.x = 4.0;
            if button(&mut buttons, &style.controls, Button::Close).on_hover_text("Close").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            let max = maximized(&ctx);
            let (kind, tip) = if max { (Button::Restore, "Restore") } else { (Button::Maximize, "Maximise") };
            if button(&mut buttons, &style.controls, kind).on_hover_text(tip).clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
            }
            if button(&mut buttons, &style.controls, Button::Minimize).on_hover_text("Minimise").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            buttons_left = buttons.min_rect().left() - CONTROLS_GAP;
        }
        let mut right_ui = child(ui, row.with_max_x(buttons_left), egui::Layout::right_to_left(egui::Align::Center));
        right(&mut right_ui);
        let right_width = (bar.right() - right_ui.min_rect().left()).max(bar.right() - buttons_left);

        let left_rect = row.with_max_x(row.right() - right_width - ui.spacing().item_spacing.x);
        let mut left_ui = child(ui, left_rect, egui::Layout::left_to_right(egui::Align::Center));
        // Where the controls end, measured inside: a menu bar spans its area.
        let measured = |ui: &mut egui::Ui| {
            left(ui);
            ui.cursor().left()
        };
        let left_end = if menu_bar {
            egui::MenuBar::new().ui(&mut left_ui, measured).inner
        } else {
            measured(&mut left_ui)
        };
        let left_width = left_end - bar.left();
        ui.advance_cursor_after_rect(row);

        // Centred on the window, not on the gap between the controls, and
        // shortened with an ellipsis rather than running into either side.
        let room = bar.width() - 2.0 * left_width.max(right_width) - 16.0;
        if custom && room > 40.0 {
            let galley = egui::WidgetText::from(egui::RichText::new(title).strong()).into_galley(
                ui,
                Some(egui::TextWrapMode::Truncate),
                room,
                egui::TextStyle::Body,
            );
            let pos = bar.center() - galley.size() / 2.0;
            ui.painter().galley(pos, galley, style.title_text);
        }
    }
}

#[derive(Clone, Copy)]
enum Button {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn window_menu(ui: &mut egui::Ui, ctx: &egui::Context) {
    if ui.button("Minimise").clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }
    let max = maximized(ctx);
    if ui.button(if max { "Restore" } else { "Maximise" }).clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
    }
    ui.separator();
    if ui.button("Close").clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn button(ui: &mut egui::Ui, colors: &WindowControlColors, kind: Button) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(BUTTON, Sense::click());
    let fg = if resp.hovered() || resp.has_focus() { colors.icon_hover } else { colors.icon };
    let (fg, bg) = match kind {
        Button::Close if resp.hovered() => (colors.close_icon_hover, Some(colors.close_hover)),
        _ if resp.is_pointer_button_down_on() => (fg, Some(colors.pressed)),
        _ if resp.hovered() => (fg, Some(colors.hover)),
        _ => (fg, None),
    };
    let painter = ui.painter();
    if let Some(bg) = bg {
        painter.rect_filled(rect, colors.radius, bg);
    }
    let stroke = Stroke::new(1.2, fg);
    let c = rect.center();
    let r = 4.5;
    match kind {
        Button::Minimize => {
            painter.line_segment([c + vec2(-r, r), c + vec2(r, r)], stroke);
        }
        Button::Maximize => {
            painter.rect_stroke(egui::Rect::from_center_size(c, vec2(2.0 * r, 2.0 * r)), 0.0, stroke, egui::StrokeKind::Middle);
        }
        Button::Restore => {
            let back = egui::Rect::from_center_size(c + vec2(1.5, -1.5), vec2(2.0 * r - 1.5, 2.0 * r - 1.5));
            let front = egui::Rect::from_center_size(c + vec2(-1.5, 1.5), vec2(2.0 * r - 1.5, 2.0 * r - 1.5));
            painter.line_segment([back.left_top(), back.right_top()], stroke);
            painter.line_segment([back.right_top(), back.right_bottom()], stroke);
            painter.line_segment([back.left_top(), egui::pos2(back.left(), front.top())], stroke);
            painter.line_segment([back.right_bottom(), egui::pos2(front.right(), back.bottom())], stroke);
            painter.rect_stroke(front, 0.0, stroke, egui::StrokeKind::Middle);
        }
        Button::Close => {
            painter.line_segment([c + vec2(-r, -r), c + vec2(r, r)], stroke);
            painter.line_segment([c + vec2(-r, r), c + vec2(r, -r)], stroke);
        }
    }
    resp
}

/// Where the press that a compositor move or resize took over went down.
fn grab_id() -> Id {
    Id::new("compositor_grab")
}

/// Sends a move or resize, which hands the pointer to the compositor until
/// the button is let go. The compositor keeps that release, so egui would go
/// on thinking the button is down and the title bar still being dragged —
/// and the next attempt to move the window would start no new drag and do
/// nothing. `release_grab` stands in for the lost release.
fn hand_to_compositor(ctx: &egui::Context, command: ViewportCommand) {
    let pos = ctx.input(|i| i.pointer.press_origin().or(i.pointer.latest_pos()));
    if let Some(pos) = pos {
        ctx.data_mut(|d| d.insert_temp(grab_id(), pos));
    }
    ctx.send_viewport_cmd(command);
}

/// Adds the button release the compositor kept after a move or resize to the
/// next frame's input (see `hand_to_compositor`). Call from
/// `App::raw_input_hook`.
pub fn release_grab(ctx: &egui::Context, raw_input: &mut egui::RawInput) {
    let Some(pos) = ctx.data_mut(|d| d.remove_temp::<egui::Pos2>(grab_id())) else {
        return;
    };
    // Moved well away first, so the release isn't taken for a click on
    // whatever is under the press; then gone, as the compositor has it.
    let away = pos + vec2(1.0e4, 1.0e4);
    let release = [
        egui::Event::PointerMoved(away),
        egui::Event::PointerButton {
            pos: away,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: raw_input.modifiers,
        },
        egui::Event::PointerGone,
    ];
    raw_input.events.splice(0..0, release);
}

/// Which edge or corner of `window` `pos` is on, if any.
fn edge_at(window: egui::Rect, pos: egui::Pos2) -> Option<ResizeDirection> {
    if !window.contains(pos) {
        return None;
    }
    let near = |d: f32, zone: f32| d < zone;
    let (l, r) = (pos.x - window.left(), window.right() - pos.x);
    let (t, b) = (pos.y - window.top(), window.bottom() - pos.y);
    let corner = |a: f32, b: f32| near(a, CORNER) && near(b, CORNER) && (near(a, EDGE) || near(b, EDGE));
    Some(match () {
        _ if corner(t, l) => ResizeDirection::NorthWest,
        _ if corner(t, r) => ResizeDirection::NorthEast,
        _ if corner(b, l) => ResizeDirection::SouthWest,
        _ if corner(b, r) => ResizeDirection::SouthEast,
        _ if near(t, EDGE) => ResizeDirection::North,
        _ if near(b, EDGE) => ResizeDirection::South,
        _ if near(l, EDGE) => ResizeDirection::West,
        _ if near(r, EDGE) => ResizeDirection::East,
        _ => return None,
    })
}

fn cursor(direction: ResizeDirection) -> CursorIcon {
    match direction {
        ResizeDirection::North => CursorIcon::ResizeNorth,
        ResizeDirection::South => CursorIcon::ResizeSouth,
        ResizeDirection::East => CursorIcon::ResizeEast,
        ResizeDirection::West => CursorIcon::ResizeWest,
        ResizeDirection::NorthEast => CursorIcon::ResizeNorthEast,
        ResizeDirection::SouthEast => CursorIcon::ResizeSouthEast,
        ResizeDirection::NorthWest => CursorIcon::ResizeNorthWest,
        ResizeDirection::SouthWest => CursorIcon::ResizeSouthWest,
    }
}

/// The border and resize handles. Call after everything else is drawn, so
/// the handles sit above the floating windows along the edges.
pub fn edges(ctx: &egui::Context, radius: u8, border: Color32) {
    if maximized(ctx) {
        return;
    }
    let window = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(Order::Foreground, Id::new("window_border")));
    painter.rect_stroke(window, radius, Stroke::new(1.0, border), egui::StrokeKind::Inside);

    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    let Some(direction) = edge_at(window, pos) else {
        return;
    };
    // An area over just the edge under the pointer, on top, so the press
    // goes to the resize and not to whatever is drawn beneath it.
    let grab = egui::Rect::from_center_size(pos, vec2(2.0 * EDGE, 2.0 * EDGE)).intersect(window);
    egui::Area::new(Id::new("window_resize"))
        .order(Order::Foreground)
        .fixed_pos(grab.min)
        .constrain(false)
        .show(ctx, |ui| {
            let (_, resp) = ui.allocate_exact_size(grab.size(), Sense::drag());
            let resp = resp.on_hover_cursor(cursor(direction));
            if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()) {
                hand_to_compositor(ctx, ViewportCommand::BeginResize(direction));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(800.0, 600.0))
    }

    /// Runs a frame of a 800x600 window with just the title bar, through
    /// `release_grab` like an app's input hook, and returns whether it asked
    /// the compositor to move the window.
    fn frame(ctx: &egui::Context, time: f64, events: Vec<egui::Event>) -> bool {
        let mut input = egui::RawInput {
            screen_rect: Some(window()),
            time: Some(time),
            events,
            ..Default::default()
        };
        release_grab(ctx, &mut input);
        let style = FrameStyle::from_visuals(&egui::Visuals::dark());
        let output = ctx.run_ui(input, |ui| {
            egui::Panel::top("bar").show(ui, |ui| TitleBar::new("Title", &style).show(ui, |_| {}, |_| {}));
        });
        output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, ViewportCommand::StartDrag))
    }

    fn press(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }

    #[test]
    fn the_title_bar_moves_the_window_every_time() {
        let ctx = egui::Context::default();
        let mut t = 0.0;
        let mut step = |events| {
            t += 0.016;
            frame(&ctx, t, events)
        };
        step(vec![egui::Event::PointerMoved(egui::pos2(300.0, 10.0))]);
        for attempt in 0..3 {
            let start = egui::pos2(300.0, 10.0);
            step(vec![egui::Event::PointerMoved(start), press(start, true)]);
            let mut moved = false;
            for i in 1..6 {
                let pos = start + vec2(10.0 * i as f32, 0.0);
                moved |= step(vec![egui::Event::PointerMoved(pos)]);
            }
            assert!(moved, "attempt {attempt} didn't move the window");
            // The compositor keeps the release; the pointer comes back later.
            step(vec![egui::Event::PointerGone]);
        }
    }

    #[test]
    fn edges_and_corners_resize_the_right_way() {
        let w = window();
        assert_eq!(edge_at(w, egui::pos2(400.0, 2.0)), Some(ResizeDirection::North));
        assert_eq!(edge_at(w, egui::pos2(400.0, 598.0)), Some(ResizeDirection::South));
        assert_eq!(edge_at(w, egui::pos2(1.0, 300.0)), Some(ResizeDirection::West));
        assert_eq!(edge_at(w, egui::pos2(799.0, 300.0)), Some(ResizeDirection::East));
        assert_eq!(edge_at(w, egui::pos2(2.0, 10.0)), Some(ResizeDirection::NorthWest));
        assert_eq!(edge_at(w, egui::pos2(790.0, 598.0)), Some(ResizeDirection::SouthEast));
    }

    #[test]
    fn the_inside_and_the_outside_do_not_resize() {
        let w = window();
        assert_eq!(edge_at(w, egui::pos2(400.0, 300.0)), None);
        assert_eq!(edge_at(w, egui::pos2(10.0, 10.0)), None);
        assert_eq!(edge_at(w, egui::pos2(-1.0, 300.0)), None);
    }
}
