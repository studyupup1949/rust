use eframe::egui::{self, Color32};
use image;
use std::sync::Arc;

impl super::TrackerApp {
    pub fn load_banner(&mut self, ctx: &egui::Context, path: &str) {
        if let Ok(img_bytes) = std::fs::read(path) {
            if let Ok(img) = image::load_from_memory(&img_bytes) {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = ctx.load_texture("banner", color_image, Default::default());
                self.banner_texture = Some(Arc::new(texture));
            }
        }
    }

    pub fn draw_banner(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.banner_texture.is_none() {
            let path = self.banner_path.clone();
            self.load_banner(ctx, &path);
        }

        if let Some(tex) = &self.banner_texture {
            let screen_width = ui.available_width();
            let banner_height = screen_width / 5.5;

            let fade_alpha = (1.0 - (self.scroll_offset / 300.0)).clamp(0.3, 1.0);

            let rect = ui.available_rect_before_wrap();
            let pos = rect.min;

            ui.painter().image(
                tex.id(),
                egui::Rect::from_min_size(pos, egui::vec2(screen_width, banner_height)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::from_white_alpha((fade_alpha * 255.0) as u8),
            );

            ui.painter().text(
                pos + egui::vec2(screen_width / 2.0, banner_height / 2.0),
                egui::Align2::CENTER_CENTER,
                "📘 Assignment Tracker",
                egui::TextStyle::Heading.resolve(ui.style()),
                Color32::WHITE,
            );

            ui.add_space(banner_height);
        }
    }
}
