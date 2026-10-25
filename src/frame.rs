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
pub(crate) const ROW_HEIGHT: f32 = 25.0;
/// Space between the app's right-hand controls and the window buttons.
const CONTROLS_GAP: f32 = 8.0;
/// The header bar's padding.
const HEADER_MARGIN: egui::Margin = egui::Margin::symmetric(12, 8);
/// Space either side of a menu's name in the menu bar.
const MENU_PADDING: f32 = 10.0;
/// egui's, in its `menu_style`.
const EGUI_MENU_PADDING: f32 = 2.0;
/// Padding inside the bar's buttons, Photograph's Render and Edit (ADR-0028).
const BAR_BUTTON_PADDING: egui::Vec2 = vec2(10.0, 5.0);
/// The bar's buttons are at least this high (ADR-0022, ADR-0028).
const BAR_BUTTON_HEIGHT: f32 = 24.0;

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
    if custom {
        [0.0; 4]
    } else {
        visuals.panel_fill.to_normalized_gamma_f32()
    }
}

/// Compositors that can't minimise or maximise a window, by process name.
/// They ignore the requests, so the buttons would do nothing.
const WITHOUT_MIN_MAX: &[&str] = &["dwl"];

/// Whether the desktop can minimise and maximise windows. The header bar
/// leaves those buttons out where it can't; apps can ask too, for their own
/// shortcuts or menus.
///
/// Wayland tells each window this (`wm_capabilities`), but winit 0.30 drops
/// it, so for now this goes by which compositor it is, found as fastfetch
/// does: the process at the other end of the Wayland socket, looked up in a
/// table. Anything unknown is assumed able to.
pub fn can_minimise_and_maximise() -> bool {
    static CAN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CAN.get_or_init(|| !compositor().is_some_and(|name| WITHOUT_MIN_MAX.contains(&name.as_str())))
}

/// The Wayland compositor's process name, if there is one and it can be seen
/// (not from inside a sandbox with its own process IDs, say).
#[cfg(target_os = "linux")]
fn compositor() -> Option<String> {
    use std::os::fd::AsRawFd;
    let display = std::path::PathBuf::from(std::env::var_os("WAYLAND_DISPLAY")?);
    let socket = if display.is_absolute() {
        display
    } else {
        std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?).join(display)
    };
    let stream = std::os::unix::net::UnixStream::connect(socket).ok()?;
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `cred` and `len` describe a writable `ucred`, as SO_PEERCRED expects.
    let ok = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut cred).cast(),
            &mut len,
        )
    } == 0;
    if !ok || cred.pid <= 0 {
        return None;
    }
    let comm = std::fs::read_to_string(format!("/proc/{}/comm", cred.pid)).ok()?;
    Some(comm.trim_end().to_owned())
}

#[cfg(not(target_os = "linux"))]
fn compositor() -> Option<String> {
    None
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
        .inner_margin(HEADER_MARGIN)
        .corner_radius(egui::CornerRadius {
            nw: radius,
            ne: radius,
            sw: 0,
            se: 0,
        })
}

/// The header bar's full height, padding and all, from inside a panel
/// framed by [`header_frame`]: the span for [`TitleBar::span`] and
/// [`TabStrip::span`](crate::tabs::TabStrip::span).
pub fn header_span(ui: &egui::Ui) -> egui::Rangef {
    (ui.max_rect() + HEADER_MARGIN).y_range()
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
    minimise: bool,
    span: Option<egui::Rangef>,
}

impl<'a> TitleBar<'a> {
    pub fn new(title: &'a str, style: &'a FrameStyle) -> Self {
        Self {
            title,
            style,
            custom: true,
            menu_bar: false,
            minimise: true,
            span: None,
        }
    }

    /// The bar's full height, padding and all, as given to
    /// [`TabStrip::span`](crate::tabs::TabStrip::span): the menus in a menu
    /// bar fill it, as the tabs do. Without it they fill the row.
    pub fn span(mut self, span: egui::Rangef) -> Self {
        self.span = Some(span);
        self
    }

    /// Whether the window buttons include minimise (the default, where the
    /// desktop can). Off for a desktop that has nowhere to minimise to, as
    /// in the frames the Patina compositor draws for X11 windows.
    pub fn minimise(mut self, minimise: bool) -> Self {
        self.minimise = minimise;
        self
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
    pub fn show(
        self,
        ui: &mut egui::Ui,
        left: impl FnOnce(&mut egui::Ui),
        right: impl FnOnce(&mut egui::Ui),
    ) {
        let Self {
            title,
            style,
            custom,
            menu_bar,
            minimise,
            span,
        } = self;
        let ctx = ui.ctx().clone();
        let bar = ui.max_rect();
        if custom {
            // Registered before the controls and buttons, so they win where
            // they overlap and only empty space moves the window.
            let resp = ui.interact(bar, Id::new("title_bar_drag"), Sense::click_and_drag());
            if resp.drag_started_by(egui::PointerButton::Primary) {
                hand_to_compositor(&ctx, ViewportCommand::StartDrag);
            }
            if resp.double_clicked() && can_minimise_and_maximise() {
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
            if button(&mut buttons, &style.controls, Button::Close)
                .on_hover_text("Close")
                .clicked()
            {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            if can_minimise_and_maximise() {
                let max = maximized(&ctx);
                let (kind, tip) = if max {
                    (Button::Restore, "Restore")
                } else {
                    (Button::Maximize, "Maximise")
                };
                if button(&mut buttons, &style.controls, kind)
                    .on_hover_text(tip)
                    .clicked()
                {
                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
                }
                if minimise
                    && button(&mut buttons, &style.controls, Button::Minimize)
                        .on_hover_text("Minimise")
                        .clicked()
                {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            }
            buttons_left = buttons.min_rect().left() - CONTROLS_GAP;
        }
        let mut right_ui = child(
            ui,
            row.with_max_x(buttons_left),
            egui::Layout::right_to_left(egui::Align::Center),
        );
        bar_controls(&mut right_ui, style);
        steady_controls(&mut right_ui);
        bar_buttons(&mut right_ui);
        right(&mut right_ui);
        let right_width =
            (bar.right() - right_ui.min_rect().left()).max(bar.right() - buttons_left);

        let mut left_rect = row.with_max_x(row.right() - right_width - ui.spacing().item_spacing.x);
        if menu_bar {
            // The first menu's name stays where egui's menu bar put it, its
            // box reaching into the bar's padding.
            left_rect.min.x -= MENU_PADDING - EGUI_MENU_PADDING;
            if let Some(span) = span {
                left_rect = egui::Rect::from_x_y_ranges(left_rect.x_range(), span);
            }
        }
        let mut left_ui = child(
            ui,
            left_rect,
            egui::Layout::left_to_right(egui::Align::Center),
        );
        bar_controls(&mut left_ui, style);
        steady_controls(&mut left_ui);
        // Where the controls end, measured inside: a menu bar spans its area.
        let measured = |ui: &mut egui::Ui| {
            left(ui);
            ui.cursor().left()
        };
        let left_end = if menu_bar {
            let w = &left_ui.visuals().widgets;
            let lit = [w.hovered.weak_bg_fill, w.active.weak_bg_fill];
            let height = left_rect.height();
            // The menu bar's row takes this before its style applies.
            left_ui.spacing_mut().interact_size.y = height;
            let layer = left_ui.layer_id();
            let from = ctx.graphics(|g| g.get(layer).map_or(0, |l| l.next_idx().0));
            let end = egui::MenuBar::new()
                .style(move |s: &mut egui::Style| menu_bar_style(s, height))
                .ui(&mut left_ui, measured)
                .inner;
            if custom {
                round_corner(&ctx, layer, from, lit, corner_radius(&ctx, style.radius));
            }
            end
        } else {
            measured(&mut left_ui)
        };
        let left_width = left_end - bar.left();
        ui.advance_cursor_after_rect(row);

        // Centred on the window, not on the gap between the controls, and
        // shortened with an ellipsis rather than running into either side.
        let room = bar.width() - 2.0 * left_width.max(right_width) - 16.0;
        if custom && room > 40.0 {
            let galley =
                egui::WidgetText::from(egui::RichText::new(title).strong().color(style.title_text))
                    .into_galley(
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

/// Colours the bar's controls from the frame style when a skin gives the bar
/// a colour of its own (Calendar's red, Notebook's brown), so menus and
/// buttons read on it: text in `title_text`, fills from the window controls'
/// hover and pressed colours, and a wash of the title text at rest. Popups
/// build their own UI, so menus keep the skin's normal colours. A bar in the
/// panel colour, as in Default and Dark Orange, is left alone.
fn bar_controls(ui: &mut egui::Ui, style: &FrameStyle) {
    let v = ui.visuals_mut();
    if style.title_bar == v.panel_fill {
        return;
    }
    let ink = style.title_text;
    v.override_text_color = Some(ink);
    v.weak_text_color = Some(ink.gamma_multiply(0.75));
    v.widgets.noninteractive.bg_stroke.color = ink.gamma_multiply(0.35);
    for (w, fill) in [
        (&mut v.widgets.inactive, ink.gamma_multiply(0.14)),
        (&mut v.widgets.hovered, style.controls.hover),
        (&mut v.widgets.active, style.controls.pressed),
        (&mut v.widgets.open, style.controls.pressed),
    ] {
        w.weak_bg_fill = fill;
        w.fg_stroke.color = ink;
    }
    v.selection.bg_fill = ink.gamma_multiply(0.26);
    v.selection.stroke.color = ink;
}

/// Rounds the first menu's fill, lit in one of `lit` and painted on `layer` from
/// shape `from` on, as the window's top-left corner is rounded, so it
/// doesn't show past the corner. Starting inside the window, it stays
/// inside with the window's own radius.
fn round_corner(
    ctx: &egui::Context,
    layer: egui::LayerId,
    from: usize,
    lit: [Color32; 2],
    radius: u8,
) {
    if radius == 0 {
        return;
    }
    let corner = ctx.content_rect().min + egui::Vec2::splat(f32::from(radius));
    fn round(shape: &mut egui::Shape, corner: egui::Pos2, lit: [Color32; 2], radius: u8) {
        match shape {
            egui::Shape::Rect(r)
                if lit.contains(&r.fill) && r.rect.min.x < corner.x && r.rect.min.y < corner.y =>
            {
                r.corner_radius.nw = radius;
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    round(s, corner, lit, radius);
                }
            }
            _ => {}
        }
    }
    ctx.graphics_mut(|g| {
        let list = g.entry(layer);
        for i in from..list.next_idx().0 {
            list.mutate_shape(egui::layers::ShapeIdx(i), |c| {
                round(&mut c.shape, corner, lit, radius);
            });
        }
    });
}

/// The menu bar's style (ADR-0024): egui's menu style, with each menu
/// `height` tall, the bar's, and side by side with no gap, so the whole bar
/// above a name opens it, lit in the bar's hover colour as the tabs are, and
/// square like them.
fn menu_bar_style(style: &mut egui::Style, height: f32) {
    egui::containers::menu::menu_style(style);
    style.spacing.interact_size.y = height;
    style.spacing.button_padding = vec2(MENU_PADDING, 0.0);
    style.spacing.item_spacing.x = 0.0;
    let w = &mut style.visuals.widgets;
    for w in [&mut w.hovered, &mut w.active, &mut w.open] {
        w.corner_radius = egui::CornerRadius::ZERO;
        w.expansion = 0.0;
    }
}

/// Takes the strokes off the bar's controls. egui draws an idle toggle
/// (`selectable_label`) without a frame and a hovered one with a framed
/// stroke, and the stroke adds to the widget's size, so with a skin that
/// strokes its controls, hovering one widened it and pushed its
/// neighbours along. Without strokes, hovering changes only the fill.
/// Sizes the buttons on the bar's right, its output actions, as Photograph's
/// Render and Edit (ADR-0028): 10 px either side of the text and 5 above and
/// below, and never under 24 px high, where egui's are 4 by 1 and 18 high.
fn bar_buttons(ui: &mut egui::Ui) {
    let spacing = ui.spacing_mut();
    spacing.button_padding = BAR_BUTTON_PADDING;
    spacing.interact_size.y = spacing.interact_size.y.max(BAR_BUTTON_HEIGHT);
}

fn steady_controls(ui: &mut egui::Ui) {
    let w = &mut ui.visuals_mut().widgets;
    for state in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        state.bg_stroke.width = 0.0;
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
    if can_minimise_and_maximise() {
        if ui.button("Minimise").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        let max = maximized(ctx);
        if ui
            .button(if max { "Restore" } else { "Maximise" })
            .clicked()
        {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
        }
        ui.separator();
    }
    if ui.button("Close").clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn button(ui: &mut egui::Ui, colors: &WindowControlColors, kind: Button) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(BUTTON, Sense::click());
    let fg = if resp.hovered() || resp.has_focus() {
        colors.icon_hover
    } else {
        colors.icon
    };
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
            painter.rect_stroke(
                egui::Rect::from_center_size(c, vec2(2.0 * r, 2.0 * r)),
                0.0,
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        Button::Restore => {
            let back = egui::Rect::from_center_size(
                c + vec2(1.5, -1.5),
                vec2(2.0 * r - 1.5, 2.0 * r - 1.5),
            );
            let front = egui::Rect::from_center_size(
                c + vec2(-1.5, 1.5),
                vec2(2.0 * r - 1.5, 2.0 * r - 1.5),
            );
            painter.line_segment([back.left_top(), back.right_top()], stroke);
            painter.line_segment([back.right_top(), back.right_bottom()], stroke);
            painter.line_segment(
                [back.left_top(), egui::pos2(back.left(), front.top())],
                stroke,
            );
            painter.line_segment(
                [
                    back.right_bottom(),
                    egui::pos2(front.right(), back.bottom()),
                ],
                stroke,
            );
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
    let corner =
        |a: f32, b: f32| near(a, CORNER) && near(b, CORNER) && (near(a, EDGE) || near(b, EDGE));
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

/// epaint's own rounding added to a rounded corner, which the border has too.
const EPAINT_ROUNDING_TWEAK: f32 = 0.4;

/// Gives the panels holding the window's corners their full rounding where
/// epaint would cut it short. epaint rounds a corner by at most half the
/// rect's shorter side, so a status bar under 31 px high got its 15 px
/// corners cut to half its height and showed past the border, which keeps
/// the full radius. Such a fill is redrawn as a path, rounded as the border
/// is.
fn round_short_corners(ctx: &egui::Context, window: egui::Rect) {
    fn fix(shape: &mut egui::Shape, window: egui::Rect) {
        match shape {
            egui::Shape::Rect(r) if short_corner(r, window) => {
                *shape = rounded_fill(r.rect, r.corner_radius, r.fill);
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    fix(s, window);
                }
            }
            _ => {}
        }
    }
    ctx.graphics_mut(|g| {
        let list = g.entry(egui::LayerId::background());
        for i in 0..list.next_idx().0 {
            list.mutate_shape(egui::layers::ShapeIdx(i), |c| fix(&mut c.shape, window));
        }
    });
}

/// Whether `r` is a plain fill with a rounded corner on one of `window`'s
/// that epaint would round less than asked.
fn short_corner(r: &egui::epaint::RectShape, window: egui::Rect) -> bool {
    let cr = r.corner_radius;
    let plain = r.stroke.width == 0.0 && r.blur_width == 0.0 && r.brush.is_none() && r.angle == 0.0;
    let at = |a: egui::Pos2, b: egui::Pos2| (a - b).length() < 0.5;
    let (rect, half) = (r.rect, r.rect.size().min_elem() / 2.0);
    let short = |radius: u8, corner: egui::Pos2, window: egui::Pos2| {
        radius > 0 && f32::from(radius) + EPAINT_ROUNDING_TWEAK > half && at(corner, window)
    };
    plain
        && (short(cr.nw, rect.left_top(), window.left_top())
            || short(cr.ne, rect.right_top(), window.right_top())
            || short(cr.sw, rect.left_bottom(), window.left_bottom())
            || short(cr.se, rect.right_bottom(), window.right_bottom()))
}

/// `rect` filled with `cr` at its corners, as epaint would round it without
/// cutting the radius to the rect's size.
fn rounded_fill(rect: egui::Rect, cr: egui::CornerRadius, fill: Color32) -> egui::Shape {
    use egui::epaint::tessellator::path::add_circle_quadrant;
    // A corner can still be no larger than the rect itself.
    let r = |radius: u8| {
        if radius == 0 {
            0.0
        } else {
            (f32::from(radius) + EPAINT_ROUNDING_TWEAK).min(rect.size().min_elem())
        }
    };
    let (min, max) = (rect.min, rect.max);
    let mut points = Vec::new();
    for (centre, radius, quadrant) in [
        (
            egui::pos2(max.x - r(cr.se), max.y - r(cr.se)),
            r(cr.se),
            0.0,
        ),
        (
            egui::pos2(min.x + r(cr.sw), max.y - r(cr.sw)),
            r(cr.sw),
            1.0,
        ),
        (
            egui::pos2(min.x + r(cr.nw), min.y + r(cr.nw)),
            r(cr.nw),
            2.0,
        ),
        (
            egui::pos2(max.x - r(cr.ne), min.y + r(cr.ne)),
            r(cr.ne),
            3.0,
        ),
    ] {
        add_circle_quadrant(&mut points, centre, radius, quadrant);
    }
    points.dedup();
    egui::Shape::convex_polygon(points, fill, Stroke::NONE)
}

/// The border and resize handles. Call after everything else is drawn, so
/// the handles sit above the floating windows along the edges.
pub fn edges(ctx: &egui::Context, radius: u8, border: Color32) {
    if maximized(ctx) {
        return;
    }
    let window = ctx.content_rect();
    round_short_corners(ctx, window);
    let painter = ctx.layer_painter(egui::LayerId::new(
        Order::Foreground,
        Id::new("window_border"),
    ));
    painter.rect_stroke(
        window,
        radius,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );

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
            egui::Panel::top("bar").show(ui, |ui| {
                TitleBar::new("Title", &style).show(ui, |_| {}, |_| {})
            });
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
        assert_eq!(
            edge_at(w, egui::pos2(400.0, 2.0)),
            Some(ResizeDirection::North)
        );
        assert_eq!(
            edge_at(w, egui::pos2(400.0, 598.0)),
            Some(ResizeDirection::South)
        );
        assert_eq!(
            edge_at(w, egui::pos2(1.0, 300.0)),
            Some(ResizeDirection::West)
        );
        assert_eq!(
            edge_at(w, egui::pos2(799.0, 300.0)),
            Some(ResizeDirection::East)
        );
        assert_eq!(
            edge_at(w, egui::pos2(2.0, 10.0)),
            Some(ResizeDirection::NorthWest)
        );
        assert_eq!(
            edge_at(w, egui::pos2(790.0, 598.0)),
            Some(ResizeDirection::SouthEast)
        );
    }

    #[test]
    fn a_short_status_bar_keeps_the_window_s_rounding() {
        let w = window();
        let bar = |rect: egui::Rect, cr: egui::CornerRadius| {
            egui::epaint::RectShape::filled(rect, cr, Color32::RED)
        };
        let bottom = egui::CornerRadius {
            sw: WINDOW_RADIUS,
            se: WINDOW_RADIUS,
            ..Default::default()
        };
        let top = egui::CornerRadius {
            nw: WINDOW_RADIUS,
            ne: WINDOW_RADIUS,
            ..Default::default()
        };
        let status = egui::Rect::from_min_max(egui::pos2(0.0, 574.0), w.max);
        assert!(short_corner(&bar(status, bottom), w));
        // Tall enough for epaint to round it fully.
        let header = egui::Rect::from_min_max(w.min, egui::pos2(800.0, 41.0));
        assert!(!short_corner(&bar(header, top), w));
        // Not at the window's corners.
        let inside = status.translate(vec2(0.0, -100.0));
        assert!(!short_corner(&bar(inside, bottom), w));

        let egui::Shape::Path(path) = rounded_fill(status, bottom, Color32::RED) else {
            panic!("not a path");
        };
        assert!(path.points.iter().all(|p| status.expand(0.01).contains(*p)));
        // The bottom-left arc reaches the full radius up the side.
        let r = f32::from(WINDOW_RADIUS) + EPAINT_ROUNDING_TWEAK;
        assert!(path.points.contains(&egui::pos2(0.0, 600.0 - r)));
    }

    #[test]
    fn the_inside_and_the_outside_do_not_resize() {
        let w = window();
        assert_eq!(edge_at(w, egui::pos2(400.0, 300.0)), None);
        assert_eq!(edge_at(w, egui::pos2(10.0, 10.0)), None);
        assert_eq!(edge_at(w, egui::pos2(-1.0, 300.0)), None);
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    /// Draws a bar with menus on the left and toggles and buttons on the
    /// right, and returns every control's rect.
    fn rects(ctx: &egui::Context, t: f64, events: Vec<egui::Event>) -> Vec<egui::Rect> {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                vec2(1100.0, 700.0),
            )),
            time: Some(t),
            events,
            ..Default::default()
        };
        let (mut left, mut right) = (Vec::new(), Vec::new());
        let _ = ctx.run_ui(input, |ui| {
            let style = FrameStyle::from_visuals(ui.visuals());
            egui::Panel::top("bar")
                .frame(header_frame(ui.style(), &style, 0))
                .show(ui, |ui| {
                    TitleBar::new("Title", &style).menu_bar(true).show(
                        ui,
                        |ui| {
                            for m in ["File", "Edit", "View"] {
                                left.push(ui.menu_button(m, |_| {}).response.rect);
                            }
                        },
                        |ui| {
                            right.push(ui.selectable_label(false, "Markdown").rect);
                            right.push(ui.button("Insert").rect);
                            right.push(ui.selectable_label(false, "B").rect);
                            right.push(ui.selectable_label(true, "I").rect);
                            right.push(ui.menu_button("Aa", |_| {}).response.rect);
                        },
                    );
                });
        });
        left.extend(right);
        left
    }

    #[test]
    fn hovering_a_control_moves_nothing_even_with_stroked_skins() {
        let ctx = egui::Context::default();
        // Dark Orange strokes its controls, which is what used to shift them.
        crate::skin::Skin::parse(crate::skin::BUILT_IN[0].1)
            .unwrap()
            .apply(&ctx);
        let rest = rects(&ctx, 0.1, vec![]);
        let rest = rects(&ctx, 0.2, vec![egui::Event::PointerGone])
            .into_iter()
            .zip(rest)
            .map(|(a, _)| a)
            .collect::<Vec<_>>();
        for (i, r) in rest.iter().enumerate() {
            let t = 1.0 + i as f64;
            rects(&ctx, t, vec![egui::Event::PointerMoved(r.center())]);
            let hovered = rects(&ctx, t + 0.1, vec![]);
            assert_eq!(hovered, rest, "hovering control {i} moved the others");
        }
    }

    #[test]
    fn the_menus_fill_the_bar_side_by_side() {
        let ctx = egui::Context::default();
        let mut found = (egui::Rangef::NOTHING, Vec::new());
        for t in [0.1, 0.2] {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(1100.0, 700.0),
                )),
                time: Some(t),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                let style = FrameStyle::from_visuals(ui.visuals());
                let header = header_frame(ui.style(), &style, 0);
                egui::Panel::top("bar").frame(header).show(ui, |ui| {
                    let span = (ui.max_rect() + header.inner_margin).y_range();
                    assert_eq!(header_span(ui), span);
                    let mut menus = Vec::new();
                    TitleBar::new("", &style).menu_bar(true).span(span).show(
                        ui,
                        |ui| {
                            for m in ["File", "Edit", "View"] {
                                menus.push(ui.menu_button(m, |_| {}).response.rect);
                            }
                        },
                        |_| {},
                    );
                    found = (span, menus);
                });
            });
        }
        let (span, menus) = found;
        for m in &menus {
            assert_eq!(m.y_range(), span, "a menu doesn't fill the bar's height");
        }
        for pair in menus.windows(2) {
            assert_eq!(pair[0].right(), pair[1].left(), "menus with a gap between");
        }
    }

    #[test]
    fn bar_controls_only_change_a_coloured_bar() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let plain = FrameStyle::from_visuals(ui.visuals());
            let before = ui.visuals().clone();
            bar_controls(ui, &plain);
            assert_eq!(
                *ui.visuals(),
                before,
                "a bar in the panel colour is left alone"
            );
            let red = FrameStyle {
                title_bar: Color32::RED,
                title_text: Color32::WHITE,
                ..plain
            };
            bar_controls(ui, &red);
            assert_eq!(
                ui.visuals().widgets.inactive.fg_stroke.color,
                Color32::WHITE
            );
            assert_eq!(ui.visuals().override_text_color, Some(Color32::WHITE));
        });
    }
}
