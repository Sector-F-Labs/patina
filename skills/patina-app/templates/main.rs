//! APP_NAME: one line saying what the app is. Built on egui and Patina.

mod app;

fn main() -> eframe::Result {
    let custom_frame = true; // from the app's config: `!config.system_title_bar`
    let options = eframe::NativeOptions {
        viewport: patina::frame::viewport(
            egui::ViewportBuilder::default()
                .with_title("APP_NAME")
                .with_app_id("APP_ID")
                .with_inner_size([1100.0, 740.0])
                .with_min_inner_size([560.0, 360.0]),
            custom_frame,
        ),
        ..Default::default()
    };
    eframe::run_native("APP_BIN", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
