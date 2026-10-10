//! Every Patina component on one screen: `cargo run --example gallery`.
//!
//! It's wired up the way an app would be, so it doubles as the reference for
//! how to use the frame (see the steps in `patina::frame`).
//!
//! With `PATINA_SCREENSHOT` set to a PNG path it saves a screenshot of its
//! window there and quits (`make screenshot`). `PATINA_THEME=light|dark`
//! picks the theme, and `PATINA_ROOT` the folder tree's root (default: home).

use std::path::PathBuf;

use eframe::egui;
use patina::frame::{self, FrameStyle, TitleBar};
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
        match std::env::var("PATINA_THEME").as_deref() {
            Ok("light") => cc.egui_ctx.set_theme(egui::ThemePreference::Light),
            Ok("dark") => cc.egui_ctx.set_theme(egui::ThemePreference::Dark),
            _ => {}
        }
        let root = std::env::var_os("PATINA_ROOT")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        let label = root.file_name().map_or("/".into(), |n| n.to_string_lossy().into_owned());
        Self {
            custom_frame: true,
            tree: FolderTree::new(root.clone(), label),
            current: root,
            text: "Some text".into(),
            value: 0.4,
            checked: true,
            choice: 0,
            screenshot: std::env::var_os("PATINA_SCREENSHOT").map(|p| (PathBuf::from(p), false)),
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
                ui.menu_button("Theme", |ui| {
                    let mut pref = ui.options(|o| o.theme_preference);
                    for (value, label) in [
                        (egui::ThemePreference::System, "Follow desktop"),
                        (egui::ThemePreference::Light, "Light"),
                        (egui::ThemePreference::Dark, "Dark"),
                    ] {
                        if ui.radio_value(&mut pref, value, label).clicked() {
                            ctx.set_theme(pref);
                        }
                    }
                });
            },
            |ui| {
                let _ = ui.button("Right");
                ui.label(egui::RichText::new("right-hand controls").weak());
            },
        );
    }

    fn widgets(&mut self, ui: &mut egui::Ui) {
        ui.heading("Widgets");
        ui.label("Stock egui widgets in the active visuals; a skin restyles all of these.");
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

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.tree.poll();
        let style = FrameStyle::from_visuals(ui.visuals());
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
