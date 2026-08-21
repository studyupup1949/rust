use eframe::egui::{self, Color32, Ui};

impl super::TrackerApp {
    pub fn show_settings(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.heading("⚙ Settings & About");

        // 🌙 Theme toggle
        let dark_mode = ctx.style().visuals.dark_mode;
        let theme_icon = if dark_mode {
            "☀ Light Mode"
        } else {
            "🌙 Dark Mode"
        };
        let theme_color = if dark_mode {
            Color32::from_rgb(255, 230, 100)
        } else {
            Color32::from_rgb(120, 180, 255)
        };

        if ui
            .add(egui::Button::new(theme_icon).fill(theme_color))
            .clicked()
        {
            let visuals = if dark_mode {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            };
            ctx.set_visuals(visuals);
        }

        ui.separator();
        ui.label("🖼 Change Banner Image:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.banner_path);
            if ui.button("Load").clicked() {
                let path = self.banner_path.clone();
                self.load_banner(ctx, &path);
            }
        });

        ui.separator();

        // ℹ About window
        if ui.button("ℹ About This App").clicked() {
            self.show_about = true;
        }

        if self.show_about {
            egui::Window::new("About This App")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("📘 Assignment Tracker GUI");
                    ui.label("👨‍💻 Developer: Srinath Reddy");
                    ui.label("🎓 Rowan University – Computer Science Major");
                    ui.label("🗓 2025 | Rust Project – Programming Languages Course");
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }
    }
}
