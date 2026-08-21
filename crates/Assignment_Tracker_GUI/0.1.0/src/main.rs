use eframe::{egui, NativeOptions};
mod app;
mod assignment;
mod theme;
mod ui_helpers;

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();

    eframe::run_native(
        "Assignment Tracker",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(app::TrackerApp::new())
        }),
    )
}

///  Use only Segoe UI Emoji for the entire interface (Windows)
fn setup_fonts(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;
    use std::path::Path;

    let mut fonts = egui::FontDefinitions::default();
    let emoji_path = "C:/Windows/Fonts/seguiemj.ttf";

    if Path::new(emoji_path).exists() {
        if let Ok(data) = std::fs::read(emoji_path) {
            fonts
                .font_data
                .insert("SegoeUIEmoji".to_owned(), egui::FontData::from_owned(data));
            fonts
                .families
                .insert(Proportional, vec!["SegoeUIEmoji".to_owned()]);
        }
    }

    ctx.set_fonts(fonts);

    // Optional: tweak layout spacing for clean look
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(6.0, 6.0);
        style.spacing.button_padding = egui::vec2(6.0, 4.0);
    });
}
