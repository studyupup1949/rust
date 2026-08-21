use crate::theme::ThemeColors;
use eframe::egui::{self, RichText, Ui};

pub fn add_button(ui: &mut Ui, text: &str, colors: &ThemeColors) -> bool {
    ui.add(
        egui::Button::new(RichText::new(text).color(colors.add_text).strong())
            .fill(colors.add_btn)
            .min_size(egui::vec2(150.0, 30.0)),
    )
    .clicked()
}

pub fn remove_button(ui: &mut Ui, colors: &ThemeColors) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new("❌ Remove")
                .color(colors.remove_text)
                .strong(),
        )
        .fill(colors.remove_btn)
        .min_size(egui::vec2(85.0, 25.0)),
    )
    .clicked()
}
