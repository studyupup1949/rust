use crate::assignment::Assignment;
use eframe::egui::{Color32, ProgressBar, Ui};

impl super::TrackerApp {
    pub fn show_dashboard(&mut self, ui: &mut Ui) {
        ui.heading("📊 Dashboard Summary");
        let total = self.assignments.len();
        let done = self.assignments.iter().filter(|a| a.completed).count();
        let pending = total - done;

        ui.label(format!(
            "📚 Total: {total} | ✅ Done: {done} | ⏳ Pending: {pending}"
        ));
        ui.add_space(10.0);

        if total > 0 {
            let ratio = done as f32 / total as f32;

            ui.horizontal(|ui| {
                ui.add(
                    ProgressBar::new(ratio)
                        .desired_width(250.0)
                        .fill(Color32::from_rgb(80, 200, 120))
                        .text(format!("{:.0}% Completed", ratio * 100.0)),
                );
            });

            ui.add_space(15.0);

            if ui.button("💾 Export Summary ").clicked() {
                match Assignment::export_summary_txt(&self.assignments) {
                    Ok(_) => ui.label("✅ Exported → export/assignments_summary.txt"),
                    Err(e) => ui.label(format!("❌ Export failed: {e}")),
                };
            };
        } else {
            ui.label("No assignments yet.");
        }
    }
}
