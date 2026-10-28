//! The window: Patina's frame and header bar, then the app's own panels.

use patina::frame::{self, TitleBar};
use patina::skin::{self, Skin};

/// The skin used until the user picks one. Name a Patina built-in: an app
/// with a look of its own gets it added to Patina's skins, so every app
/// offers it. "Default" is stock egui, following the desktop.
const HOME_SKIN: &str = "Default";

/// What the header asks for, handled after it's drawn.
enum Action {
    Quit,
    SetSkin(String),
    SystemTitleBar(bool),
}

pub struct App {
    custom_frame: bool,
    skins: Vec<Skin>,
    skin_errors: Vec<String>,
    /// The chosen skin's name (from the app's config); `HOME_SKIN` when unset.
    skin: Option<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        patina::appearance::watch(&cc.egui_ctx);
        let dirs: Vec<_> = skin::app_dir("APP_BIN").into_iter().collect();
        let (skins, skin_errors) = skin::load_all(skin::BUILT_IN, &dirs);
        let app = Self { custom_frame: true, skins, skin_errors, skin: None };
        app.apply_skin(&cc.egui_ctx);
        app
    }

    /// The chosen skin, falling back to `HOME_SKIN` (a saved name may no
    /// longer exist), then Default.
    fn skin(&self) -> &Skin {
        let find = |name: &str| self.skins.iter().find(|s| s.name == name);
        find(self.skin.as_deref().unwrap_or(HOME_SKIN)).or_else(|| find(HOME_SKIN)).unwrap_or(&self.skins[0])
    }

    fn apply_skin(&self, ctx: &egui::Context) {
        // Replaces the whole egui style: re-apply any spacing of the app's own after it.
        self.skin().apply(ctx);
    }

    /// Left side of the title bar: word menus, never a hamburger
    /// (Patina's ADR-0006). Add Edit, View and the app's own before Skins.
    fn menus(&self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.menu_button("File", |ui| {
            if ui.add(egui::Button::new("Quit").shortcut_text("Ctrl+Q")).clicked() {
                actions.push(Action::Quit);
                ui.close();
            }
        });
        ui.menu_button("View", |ui| {
            let mut system = !self.custom_frame;
            if ui.checkbox(&mut system, "System Title Bar").changed() {
                actions.push(Action::SystemTitleBar(system));
            }
        });
        ui.menu_button("Skins", |ui| {
            let current = self.skin().name.clone();
            for skin in &self.skins {
                if ui.radio(skin.name == current, &skin.name).clicked() {
                    actions.push(Action::SetSkin(skin.name.clone()));
                }
            }
            for error in &self.skin_errors {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Every frame: Default's colours follow the desktop's light/dark setting.
        let style = self.skin().frame_style(ui.visuals());
        let radius = if self.custom_frame { frame::corner_radius(&ctx, style.radius) } else { 0 };

        let mut actions = Vec::new();
        egui::Panel::top("header").frame(frame::header_frame(ui.style(), &style, radius)).show(ui, |ui| {
            TitleBar::new("APP_NAME", &style).custom(self.custom_frame).menu_bar(true).span(frame::header_span(ui)).show(
                ui,
                |ui| self.menus(ui, &mut actions),
                |_ui| {
                    // Right side, next to the window buttons: the actions that make the
                    // app's main output (Photograph's Render and Edit). Laid out right to
                    // left. Empty for an app with no output, like a music player.
                },
            );
        });
        for action in actions {
            match action {
                Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                Action::SetSkin(name) => {
                    self.skin = Some(name); // and save it in the app's config
                    self.apply_skin(&ctx);
                }
                Action::SystemTitleBar(on) => {
                    self.custom_frame = !on; // and save it in the app's config
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(on));
                }
            }
        }

        // Whichever panels hold the bottom corners round them.
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).corner_radius(egui::CornerRadius {
                sw: radius,
                se: radius,
                ..Default::default()
            }))
            .show(ui, |ui| {
                ui.label("Hello from a Patina app");
            });

        if self.custom_frame {
            frame::edges(&ctx, radius, style.border);
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        frame::clear_color(visuals, self.custom_frame)
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        frame::release_grab(ctx, raw_input);
    }
}
