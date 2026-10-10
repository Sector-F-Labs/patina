//! Every Patina component on one screen: `cargo run --example gallery`.
//!
//! It's wired up the way an app would be, so it doubles as the reference for
//! how to use the frame (see the steps in `patina::frame`).
//!
//! With `PATINA_SCREENSHOT` set to a PNG path it saves a screenshot of its
//! window there and quits (`make screenshot`). `PATINA_THEME` picks the
//! theme: `light`, `dark` or a theme's name. `PATINA_ROOT` sets the folder
//! tree's root (default: home).

use std::path::PathBuf;

use eframe::egui;
use patina::frame::{self, FrameStyle, TitleBar};
use patina::theme::{self, Theme};
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
    eframe::run_native("patina-gallery", options, Box::new(|cc| Ok(Box::new(Gallery::new(cc)))))
}

struct Gallery {
    custom_frame: bool,
    themes: Vec<Theme>,
    /// Index into `themes`.
    theme: usize,
    /// Light, dark or following the desktop, while the theme is Default.
    light_dark: egui::ThemePreference,
    /// Problems loading themes, shown in the Theme menu.
    theme_errors: Vec<String>,
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
        let (themes, theme_errors) = load_themes();
        let requested = std::env::var("PATINA_THEME").unwrap_or_default();
        let light_dark = match requested.as_str() {
            "light" => egui::ThemePreference::Light,
            "dark" => egui::ThemePreference::Dark,
            _ => egui::ThemePreference::System,
        };
        let theme = themes.iter().position(|t| t.name == requested).unwrap_or(0);
        let root = std::env::var_os("PATINA_ROOT")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let label = root.file_name().map_or("/".into(), |n| n.to_string_lossy().into_owned());
        let gallery = Self {
            custom_frame: true,
            themes,
            theme,
            light_dark,
            theme_errors,
            tree: FolderTree::new(root.clone(), label),
            current: root,
            text: "Some text".into(),
            value: 0.4,
            checked: true,
            choice: 0,
            screenshot: std::env::var_os("PATINA_SCREENSHOT").map(|p| (PathBuf::from(p), false)),
        };
        gallery.apply_theme(&cc.egui_ctx);
        gallery
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let theme = &self.themes[self.theme];
        theme.apply(ctx);
        if theme.follows_desktop() {
            ctx.set_theme(self.light_dark);
        }
    }

    fn theme_menu(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let mut changed = false;
        for (i, theme) in self.themes.iter().enumerate() {
            let resp = ui.radio_value(&mut self.theme, i, &theme.name);
            let resp = match (&theme.path, &theme.author) {
                (Some(path), _) => resp.on_hover_text(path.display().to_string()),
                (None, Some(author)) => resp.on_hover_text(format!("Built in, by {author}")),
                (None, None) => resp.on_hover_text("Stock egui"),
            };
            changed |= resp.clicked();
        }
        ui.separator();
        ui.add_enabled_ui(self.themes[self.theme].follows_desktop(), |ui| {
            for (value, label) in [
                (egui::ThemePreference::System, "Follow desktop"),
                (egui::ThemePreference::Light, "Light"),
                (egui::ThemePreference::Dark, "Dark"),
            ] {
                changed |= ui.radio_value(&mut self.light_dark, value, label).clicked();
            }
        });
        ui.separator();
        if ui.button("Reload themes").clicked() {
            let name = self.themes[self.theme].name.clone();
            (self.themes, self.theme_errors) = load_themes();
            self.theme = self.themes.iter().position(|t| t.name == name).unwrap_or(0);
            changed = true;
        }
        if let Some(dir) = theme::shared_dir() {
            let resp = ui.button("Copy built-in themes to folder").on_hover_text(dir.display().to_string());
            if resp.clicked() {
                match theme::export_built_ins(&dir, theme::BUILT_IN) {
                    Ok(export) => eprintln!("copied {:?}, kept {:?} in {}", export.copied, export.skipped, dir.display()),
                    Err(e) => self.theme_errors.push(e),
                }
            }
        }
        if !self.theme_errors.is_empty() {
            ui.separator();
            for error in &self.theme_errors {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
        }
        if changed {
            self.apply_theme(&ctx);
        }
    }

    fn header(&mut self, ui: &mut egui::Ui, style: &FrameStyle) {
        let ctx = ui.ctx().clone();
        TitleBar::new("Patina Gallery", style).custom(self.custom_frame).menu_bar(true).show(
            ui,
            |ui| {
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.custom_frame, "Patina frame").changed() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(!self.custom_frame));
                    }
                });
                ui.menu_button("Theme", |ui| self.theme_menu(ui));
            },
            |ui| {
                let _ = ui.button("Right");
                ui.label(egui::RichText::new("right-hand controls").weak());
            },
        );
    }

    fn widgets(&mut self, ui: &mut egui::Ui) {
        ui.heading("Widgets");
        ui.label("Stock egui widgets in the active visuals; a theme restyles all of these.");
        ui.add_space(8.0);
        egui::Grid::new("widgets").num_columns(2).spacing([24.0, 10.0]).show(ui, |ui| {
            ui.label("Buttons");
            ui.horizontal(|ui| {
                let _ = ui.button("Button");
                let _ = ui.add(egui::Button::new("Selected").selected(true));
                let _ = ui.add_enabled(false, egui::Button::new("Disabled"));
            });
            ui.end_row();
            ui.label("Text");
            ui.text_edit_singleline(&mut self.text);
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
            ui.hyperlink_to("Patina on GitHub", "https://github.com/Sector-F-Labs/patina");
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

/// Patina's built-in themes and the suite's shared folder.
fn load_themes() -> (Vec<Theme>, Vec<String>) {
    theme::load_all(theme::BUILT_IN, &theme::shared_dir().into_iter().collect::<Vec<_>>())
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.tree.poll();
        let style = self.themes[self.theme].frame_style(ui.visuals());
        let radius = if self.custom_frame { frame::corner_radius(&ctx, style.radius) } else { 0 };

        egui::Panel::top("header")
            .frame(frame::header_frame(ui.style(), &style, radius))
            .show(ui, |ui| self.header(ui, &style));

        // The panels holding the window's bottom corners round them.
        egui::Panel::left("sidebar")
            .frame(egui::Frame::side_top_panel(ui.style()).corner_radius(egui::CornerRadius { sw: radius, ..Default::default() }))
            .default_size(240.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(path) = self.tree.show(ui, &self.current.clone()) {
                        self.tree.reveal(&path);
                        self.current = path;
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).corner_radius(egui::CornerRadius { se: radius, ..Default::default() }))
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
    }
}
