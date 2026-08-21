use crate::assignment::Assignment;
use crate::theme::get_theme_colors;
use crate::ui_helpers::{add_button, remove_button};
use chrono::{Local, NaiveDate};
use eframe::egui::{self, Color32, Rounding, Stroke, Ui};

impl super::TrackerApp {
    pub fn show_assignments(&mut self, ui: &mut Ui) {
        ui.heading("✏ Manage Assignments");
        ui.add_space(10.0);

        let dark_mode = ui.ctx().style().visuals.dark_mode;
        let colors = get_theme_colors(dark_mode);
        let today = Local::now().date_naive();

        // Sorting buttons
        ui.horizontal(|ui| {
            if ui.button("📘 Sort by Subject").clicked() {
                self.assignments.sort_by(|a, b| a.subject.cmp(&b.subject));
            }
            if ui.button("📅 Sort by Due Date").clicked() {
                self.assignments.sort_by(|a, b| a.due_date.cmp(&b.due_date));
            }
        });

        ui.add_space(10.0);

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.search_query);
        });

        ui.separator();

        // Input fields
        ui.horizontal(|ui| {
            ui.label("📗 Subject:");
            ui.text_edit_singleline(&mut self.subject_input);
        });
        ui.horizontal(|ui| {
            ui.label("📄 Title:");
            ui.text_edit_singleline(&mut self.title_input);
        });

        //  Due date + time inputs
        ui.horizontal(|ui| {
            ui.label("🗓 Due (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.due_date_input);
            ui.label("⏰ Time (HH:MM):");
            ui.text_edit_singleline(&mut self.due_time_input);
        });

        // Add button
        if add_button(ui, "➕ Add Assignment", &colors) {
            if !self.subject_input.is_empty()
                && !self.title_input.is_empty()
                && !self.due_date_input.is_empty()
            {
                let parsed_date = NaiveDate::parse_from_str(&self.due_date_input, "%Y-%m-%d")
                    .map(|d| d.to_string())
                    .unwrap_or_else(|_| "Invalid date".into());

                self.assignments.push(Assignment {
                    subject: self.subject_input.clone(),
                    title: self.title_input.clone(),
                    due_date: parsed_date,
                    due_time: if self.due_time_input.is_empty() {
                        "23:59".into()
                    } else {
                        self.due_time_input.clone()
                    },
                    completed: false,
                });

                self.subject_input.clear();
                self.title_input.clear();
                self.due_date_input.clear();
                self.due_time_input = "23:59".to_string();

                Assignment::save_to_file(&self.assignments);
            }
        }

        ui.separator();

        // Search filter
        let query = self.search_query.to_lowercase();
        let filtered: Vec<_> = self
            .assignments
            .iter_mut()
            .filter(|a| {
                a.subject.to_lowercase().contains(&query) || a.title.to_lowercase().contains(&query)
            })
            .collect();

        if filtered.is_empty() {
            ui.label("No matching assignments.");
        } else {
            let mut changed = false;
            let mut remove_index: Option<usize> = None;

            for (i, a) in filtered.into_iter().enumerate() {
                let parsed = NaiveDate::parse_from_str(&a.due_date, "%Y-%m-%d");
                let (color, countdown) = if let Ok(date) = parsed {
                    let days_left = (date - today).num_days();
                    if a.completed {
                        (Color32::from_rgb(0, 200, 0), "✅ Completed".into())
                    } else if days_left < 0 {
                        (Color32::RED, format!("⏰ Overdue by {} days", -days_left))
                    } else if days_left == 0 {
                        (Color32::from_rgb(255, 180, 0), "⚠ Due today!".into())
                    } else {
                        (
                            Color32::from_rgb(120, 220, 120),
                            format!("🕒 Due in {} days", days_left),
                        )
                    }
                } else {
                    (colors.base_text, "Invalid date".into())
                };

                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.visuals_mut().widgets.noninteractive.rounding = Rounding::same(8.0);
                    ui.visuals_mut().widgets.noninteractive.bg_stroke =
                        Stroke::new(1.0, Color32::from_gray(120));

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut a.completed, "").changed() {
                            changed = true;
                        }

                        ui.label(
                            egui::RichText::new(format!(
                                "{} - {} (Due: {} {})",
                                a.subject, a.title, a.due_date, a.due_time
                            ))
                            .color(color)
                            .size(16.0)
                            .strong(),
                        );

                        ui.label(egui::RichText::new(countdown).color(color).size(14.0));

                        if remove_button(ui, &colors) {
                            remove_index = Some(i);
                        }
                    });
                });
            }

            if changed {
                Assignment::save_to_file(&self.assignments);
            }
            if let Some(i) = remove_index {
                self.assignments.remove(i);
                Assignment::save_to_file(&self.assignments);
            }
        }
    }
}
