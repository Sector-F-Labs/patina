//! Every Patina component on one screen: `cargo run --example gallery`.
//!
//! It's wired up the way an app would be, so it doubles as the reference for
//! how to use the frame (see the steps in `patina::frame`).
//!
//! With `PATINA_SCREENSHOT` set to a PNG path it saves a screenshot of its
//! window there and quits (`make screenshot`). `PATINA_SKIN` picks the
//! skin: `light`, `dark` or a skin's name. `PATINA_ROOT` sets the folder
//! tree's root (default: home).

use std::path::PathBuf;

use eframe::egui;
use patina::frame::{self, FrameStyle, TitleBar};
use patina::ime;
use patina::kinetic;
use patina::menu::{Entry, Menus, Role};
use patina::skin::{self, Skin};
use patina::tabs::{self, TabAction, TabStrip};
use patina::tree::FolderTree;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: frame::viewport(
            egui::ViewportBuilder::default()
                .with_title("Patina Gallery")
                .with_app_id("patina-gallery")
                .with_inner_size([1000.0, 680.0])
                .with_min_inner_size([600.0, 400.0]),
            true,
        ),
        ..Default::default()
    };
    eframe::run_native(
        "patina-gallery",
        options,
        Box::new(|cc| Ok(Box::new(Gallery::new(cc)))),
    )
}

struct Gallery {
    custom_frame: bool,
    /// Zoom in percent, and whether the sidebar shows, from the View menu.
    zoom: u32,
    sidebar: bool,
    /// Whether a maximised window squares its corners (off by default).
    square: bool,
    skins: Vec<Skin>,
    /// Index into `skins`.
    skin: usize,
    /// Light, dark or following the desktop, while the skin is Default.
    light_dark: egui::ThemePreference,
    /// Problems loading skins, shown in the status bar until they're fixed.
    skin_errors: Vec<String>,
    /// The latest message for the status bar (ADR-0016).
    status: Option<Status>,
    /// The title bar's tabs, in place of a title, and the one shown.
    tabs: Vec<String>,
    tab: usize,
    /// Tabs opened, for naming the next.
    opened: usize,
    tree: FolderTree,
    current: PathBuf,
    text: String,
    value: f32,
    checked: bool,
    choice: usize,
    /// Where to save a screenshot, and whether one has been asked for.
    screenshot: Option<(PathBuf, bool)>,
}

impl Gallery {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        patina::appearance::watch(&cc.egui_ctx);
        let (skins, skin_errors) = load_skins();
        let requested = std::env::var("PATINA_SKIN").unwrap_or_default();
        let light_dark = match requested.as_str() {
            "light" => egui::ThemePreference::Light,
            "dark" => egui::ThemePreference::Dark,
            _ => egui::ThemePreference::System,
        };
        let skin = skins.iter().position(|t| t.name == requested).unwrap_or(0);
        let root = std::env::var_os("PATINA_ROOT")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let label = root
            .file_name()
            .map_or("/".into(), |n| n.to_string_lossy().into_owned());
        let gallery = Self {
            custom_frame: true,
            zoom: 100,
            sidebar: true,
            square: false,
            skins,
            skin,
            light_dark,
            skin_errors,
            status: None,
            tabs: vec!["Patina Gallery".into(), "Tab 2".into()],
            tab: 0,
            opened: 2,
            tree: FolderTree::new(root.clone(), label),
            current: root,
            text: "Some text".into(),
            value: 0.4,
            checked: true,
            choice: 0,
            screenshot: std::env::var_os("PATINA_SCREENSHOT").map(|p| (PathBuf::from(p), false)),
        };
        gallery.apply_skin(&cc.egui_ctx);
        gallery
    }

    fn apply_skin(&self, ctx: &egui::Context) {
        let skin = &self.skins[self.skin];
        skin.apply(ctx);
        if skin.follows_desktop() {
            ctx.set_theme(self.light_dark);
        }
    }

    /// The skin menu every app has (ADR-0017): only controls, reporting to the status bar.
    /// The Skin menu's entries (ADR-0017): the skins, light or dark while Default is in use, and
    /// the skins folder.
    fn skin_entries(&self) -> Vec<Entry<MenuAction>> {
        let mut e = Vec::new();
        for (i, skin) in self.skins.iter().enumerate() {
            let label = match &skin.author {
                Some(author) if skin.path.is_some() => format!("{} — {author}", skin.name),
                _ => skin.name.clone(),
            };
            e.push(Entry::item(label, MenuAction::Skin(i)).radio(self.skin == i));
        }
        e.push(Entry::separator());
        let follows = self.skins[self.skin].follows_desktop();
        for (value, label) in [
            (egui::ThemePreference::System, "Follow desktop"),
            (egui::ThemePreference::Light, "Light"),
            (egui::ThemePreference::Dark, "Dark"),
        ] {
            e.push(
                Entry::item(label, MenuAction::Theme(value))
                    .radio(self.light_dark == value)
                    .enabled(follows),
            );
        }
        e.push(Entry::separator());
        e.push(Entry::item("Reload skins", MenuAction::ReloadSkins));
        if skin::app_dir(GALLERY).is_some() {
            e.push(Entry::item(
                "Copy built-in skins to folder",
                MenuAction::ExportSkins,
            ));
            e.push(Entry::item(
                "Open skins folder",
                MenuAction::OpenSkinsFolder,
            ));
        }
        e
    }

    /// Every menu, as data: Patina draws it in the window, or on macOS in the menu bar (ADR-0036).
    fn menus(&self) -> Menus<MenuAction> {
        use egui::{Key, KeyboardShortcut as K, Modifiers as M};
        let zoom = |z: u32| Entry::item(format!("{z}%"), MenuAction::Zoom(z)).radio(self.zoom == z);
        Menus::new()
            .menu(
                "File",
                [
                    Entry::item("New Tab", MenuAction::Tab(TabAction::New)),
                    Entry::item("Close Tab", MenuAction::Tab(TabAction::Close(self.tab)))
                        .enabled(self.tabs.len() > 1),
                    Entry::separator(),
                    Entry::item("About Gallery", MenuAction::About).role(Role::About),
                    Entry::item("Settings…", MenuAction::Settings).role(Role::Settings),
                    Entry::separator(),
                    Entry::item("Quit", MenuAction::Quit).role(Role::Quit),
                ],
            )
            .menu(
                "Edit",
                [
                    Entry::item("Undo", MenuAction::Say("Undo"))
                        .shortcut(K::new(M::COMMAND, Key::Z))
                        .enabled(false),
                    Entry::item("Redo", MenuAction::Say("Redo"))
                        .shortcut(K::new(M::COMMAND | M::SHIFT, Key::Z))
                        .enabled(false),
                    Entry::separator(),
                    Entry::item("Cut", MenuAction::Say("Cut")).shortcut(K::new(M::COMMAND, Key::X)),
                    Entry::item("Copy", MenuAction::Say("Copy"))
                        .shortcut(K::new(M::COMMAND, Key::C)),
                    Entry::item("Paste", MenuAction::Say("Paste"))
                        .shortcut(K::new(M::COMMAND, Key::V)),
                ],
            )
            .menu(
                "View",
                [
                    Entry::item("Patina frame", MenuAction::PatinaFrame).check(self.custom_frame),
                    Entry::separator(),
                    Entry::submenu("Zoom", [zoom(75), zoom(100), zoom(125), zoom(150)]),
                    Entry::item("Reset Zoom", MenuAction::Zoom(100))
                        .shortcut(K::new(M::COMMAND, Key::Num0))
                        .enabled(self.zoom != 100),
                    Entry::separator(),
                    Entry::submenu(
                        "Panels",
                        [
                            Entry::item("Sidebar", MenuAction::Sidebar).check(self.sidebar),
                            Entry::submenu(
                                "Window",
                                [Entry::item(
                                    "Square corners when maximised",
                                    MenuAction::SquareCorners,
                                )
                                .check(self.square)],
                            ),
                        ],
                    ),
                ],
            )
            .menu("Skin", self.skin_entries())
    }

    fn menu_action(&mut self, ctx: &egui::Context, action: MenuAction) {
        let mut changed = false;
        match action {
            MenuAction::Tab(a) => self.tab_action(a),
            MenuAction::About => {
                self.status = Some(Status::info(format!(
                    "Patina gallery {}",
                    env!("CARGO_PKG_VERSION")
                )));
            }
            MenuAction::Settings => self.status = Some(Status::info("Settings…")),
            MenuAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            MenuAction::Say(what) => self.status = Some(Status::info(what)),
            MenuAction::PatinaFrame => {
                self.custom_frame = !self.custom_frame;
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!self.custom_frame));
            }
            MenuAction::Zoom(z) => {
                self.zoom = z;
                ctx.set_zoom_factor(z as f32 / 100.0);
            }
            MenuAction::Sidebar => self.sidebar = !self.sidebar,
            MenuAction::SquareCorners => {
                self.square = !self.square;
                frame::set_square_when_maximised(ctx, self.square);
            }
            MenuAction::Skin(i) => {
                self.skin = i;
                changed = true;
            }
            MenuAction::Theme(t) => {
                self.light_dark = t;
                changed = true;
            }
            MenuAction::ReloadSkins => {
                self.reload_skins();
                changed = true;
            }
            MenuAction::ExportSkins => {
                if let Some(dir) = skin::app_dir(GALLERY) {
                    self.status = Some(match skin::export_built_ins(&dir, skin::BUILT_IN) {
                        Ok(export) => {
                            // The copies share the built-ins' names, so after a reload they're
                            // the versions in use.
                            self.reload_skins();
                            changed = true;
                            Status::info(export_summary(&export.copied, &export.skipped))
                        }
                        Err(e) => Status::error(format!("Couldn't copy skins: {e}")),
                    });
                }
            }
            MenuAction::OpenSkinsFolder => {
                if let Some(dir) = skin::app_dir(GALLERY) {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = patina::open::path(&dir);
                }
            }
        }
        if changed {
            self.apply_skin(ctx);
        }
    }

    /// Rescans the built-in and user skins, keeping the one in use by name.
    fn reload_skins(&mut self) {
        let name = self.skins[self.skin].name.clone();
        (self.skins, self.skin_errors) = load_skins();
        self.skin = self.skins.iter().position(|t| t.name == name).unwrap_or(0);
    }

    /// The status bar (ADR-0016): the version, then the latest message coloured by its level,
    /// then the app's own standing information on the right, which the gallery has none of.
    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Just the version: the app's name is already in the title bar and the window list,
            // so repeating it here adds nothing.
            ui.label(
                egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .weak()
                    .small(),
            );
            // Skins that failed to load stay until they load, ahead of any newer message.
            let skin_errors = (!self.skin_errors.is_empty()).then(|| {
                let n = self.skin_errors.len();
                Status::error(format!(
                    "Couldn't load {n} skin{}",
                    if n == 1 { "" } else { "s" }
                ))
            });
            if let Some(status) = skin_errors.as_ref().or(self.status.as_ref()) {
                ui.separator();
                let color = match status.level {
                    Level::Error => ui.visuals().error_fg_color,
                    Level::Warn => ui.visuals().warn_fg_color,
                    Level::Info => ui.visuals().weak_text_color(),
                };
                let resp = ui.label(egui::RichText::new(&status.text).color(color).small());
                if skin_errors.is_some() {
                    resp.on_hover_text(self.skin_errors.join("\n"));
                }
            }
        });
    }

    fn header(
        &mut self,
        ui: &mut egui::Ui,
        style: &FrameStyle,
        span: egui::Rangef,
    ) -> Option<TabAction> {
        let mut action = None;
        let mut menu = None;
        // No title: the tabs take its place.
        TitleBar::new("", style)
            .custom(self.custom_frame)
            .menu_bar(true)
            .span(span)
            .show(
                ui,
                |ui| {
                    // On macOS these are in the menu bar and nothing is drawn here.
                    menu = self.menus().show(ui);
                    action = TabStrip::new(&self.tabs, self.tab, style)
                        .span(span)
                        .show(ui);
                },
                |ui| {
                    let _ = ui.button("Right");
                    ui.label(egui::RichText::new("right-hand controls").weak());
                },
            );
        if let Some(menu) = menu {
            self.menu_action(ui.ctx(), menu);
        }
        action
    }

    /// Opens, shows or closes a tab, keeping at least one.
    fn tab_action(&mut self, action: TabAction) {
        match action {
            TabAction::New => {
                self.opened += 1;
                self.tabs.push(format!("Tab {}", self.opened));
                self.tab = self.tabs.len() - 1;
            }
            TabAction::Show(i) => self.tab = i,
            TabAction::Close(i) if self.tabs.len() > 1 => {
                self.tabs.remove(i);
                if i < self.tab || self.tab == self.tabs.len() {
                    self.tab -= 1;
                }
            }
            TabAction::Close(_) => {}
        }
    }

    fn widgets(&mut self, ui: &mut egui::Ui) {
        ui.heading("Widgets");
        ui.label("Stock egui widgets in the active visuals; a skin restyles all of these.");
        ui.add_space(8.0);
        egui::Grid::new("widgets")
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, |ui| {
                ui.label("Buttons");
                ui.horizontal(|ui| {
                    let _ = ui.button("Button");
                    let _ = ui.add(egui::Button::new("Selected").selected(true));
                    let _ = ui.add_enabled(false, egui::Button::new("Disabled"));
                });
                ui.end_row();
                ui.label("Text input");
                ui.horizontal(|ui| {
                    // Beside a row of dialog buttons, to compare their heights: an
                    // input is a button's size (ADR-0034).
                    ui.add(
                        patina::dialog::text_edit(ui, &mut self.text)
                            .hint_text("Text input")
                            .desired_width(200.0),
                    );
                    patina::dialog::buttons(ui);
                    let _ = ui.button("Button");
                    let _ = ui.add(patina::dialog::primary(ui, "Primary"));
                });
                ui.end_row();
                ui.label("Slider");
                ui.add(egui::Slider::new(&mut self.value, 0.0..=1.0));
                ui.end_row();
                ui.label("Check box");
                ui.checkbox(&mut self.checked, "Checked");
                ui.end_row();
                ui.label("Choice");
                ui.horizontal(|ui| {
                    for (i, label) in ["One", "Two", "Three"].iter().enumerate() {
                        ui.selectable_value(&mut self.choice, i, *label);
                    }
                });
                ui.end_row();
                ui.label("Progress");
                ui.add(egui::ProgressBar::new(self.value).show_percentage());
                ui.end_row();
                ui.label("Link");
                ui.hyperlink_to(
                    "Patina on GitHub",
                    "https://github.com/Sector-F-Labs/patina",
                );
                ui.end_row();
            });
        ui.add_space(16.0);
        ui.heading("Folder tree");
        ui.label(format!("Selected: {}", self.current.display()));
    }
}

impl Gallery {
    /// In screenshot mode: waits a second for the tree to load, captures the
    /// window, saves it and quits.
    fn take_screenshot(&mut self, ctx: &egui::Context) {
        let Some((path, requested)) = &mut self.screenshot else {
            return;
        };
        ctx.request_repaint();
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let [w, h] = image.size;
            let saved = image::RgbaImage::from_raw(w as u32, h as u32, image.as_raw().to_vec())
                .ok_or_else(|| "screenshot has the wrong size".to_string())
                .and_then(|buf| buf.save(&*path).map_err(|e| e.to_string()));
            match saved {
                Ok(()) => eprintln!("saved {}", path.display()),
                Err(e) => eprintln!("screenshot failed: {e}"),
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if !*requested && ctx.input(|i| i.time) > 1.0 {
            *requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
    }
}

/// The gallery's skins folder is `~/.config/patina-gallery/skins`.
const GALLERY: &str = "patina-gallery";

/// A message for the status bar.
struct Status {
    level: Level,
    text: String,
}

/// What the menus ask for.
#[derive(Clone)]
enum MenuAction {
    Tab(TabAction),
    About,
    Settings,
    Quit,
    /// A menu item with nothing behind it but a message in the status bar.
    Say(&'static str),
    PatinaFrame,
    Zoom(u32),
    Sidebar,
    SquareCorners,
    Skin(usize),
    Theme(egui::ThemePreference),
    ReloadSkins,
    ExportSkins,
    OpenSkinsFolder,
}

#[allow(dead_code)] // The gallery has nothing to warn about.
enum Level {
    Error,
    Warn,
    Info,
}

impl Status {
    fn error(text: impl Into<String>) -> Self {
        Status {
            level: Level::Error,
            text: text.into(),
        }
    }

    fn info(text: impl Into<String>) -> Self {
        Status {
            level: Level::Info,
            text: text.into(),
        }
    }
}

/// What Copy built-in skins did, for the status bar.
fn export_summary(copied: &[String], skipped: &[String]) -> String {
    let mut parts = Vec::new();
    if !copied.is_empty() {
        parts.push(format!("Copied {}", copied.join(", ")));
    }
    if !skipped.is_empty() {
        parts.push(format!("kept your existing {}", skipped.join(", ")));
    }
    parts.join("; ")
}

/// Patina's built-in skins and the gallery's own folder.
fn load_skins() -> (Vec<Skin>, Vec<String>) {
    skin::load_all(
        skin::BUILT_IN,
        &skin::app_dir(GALLERY).into_iter().collect::<Vec<_>>(),
    )
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, eframe_frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // The window draws its own frame, so the compositor is asked for its drop shadow (once).
        if self.custom_frame {
            frame::shadow::request(eframe_frame);
        }
        self.tree.poll();
        let style = self.skins[self.skin].frame_style(ui.visuals());
        let radius = if self.custom_frame {
            frame::corner_radius(&ctx, style.radius)
        } else {
            0
        };

        // Taken before the widgets see them.
        let mut action = tabs::keys(&ctx, self.tabs.len(), self.tab);
        let header = frame::header_frame(ui.style(), &style, radius);
        egui::Panel::top("header").frame(header).show(ui, |ui| {
            // The bar's full height, padding and all, for the menus and tabs to fill.
            let span = frame::header_span(ui);
            action = action.or(self.header(ui, &style, span));
        });
        if let Some(action) = action {
            self.tab_action(action);
        }

        // The status bar spans the window, so it comes before the sidebar, and holds both bottom
        // corners.
        egui::Panel::bottom("status")
            .frame(
                egui::Frame::side_top_panel(ui.style())
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .corner_radius(egui::CornerRadius {
                        sw: radius,
                        se: radius,
                        ..Default::default()
                    }),
            )
            .show(ui, |ui| self.status_bar(ui));
        if self.sidebar {
            egui::Panel::left("sidebar")
                .frame(egui::Frame::side_top_panel(ui.style()))
                .default_size(240.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(path) = self.tree.show(ui, &self.current.clone()) {
                            self.tree.reveal(&path);
                            self.current = path;
                        }
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()))
            .show(ui, |ui| self.widgets(ui));

        if self.custom_frame {
            frame::edges(&ctx, radius, style.border);
        }
        self.take_screenshot(&ctx);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        frame::clear_color(visuals, self.custom_frame)
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        frame::release_grab(ctx, raw_input);
        ime::drop_repeated_preedits(ctx, raw_input);
        kinetic::glide(ctx, raw_input);
    }
}
