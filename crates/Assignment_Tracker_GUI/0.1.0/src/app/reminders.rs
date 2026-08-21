#![allow(dead_code)]
use chrono::{Duration, Local, NaiveDate};
use eframe::egui;

impl super::TrackerApp {
    /// Refresh reminders based on due date (today or tomorrow)
    pub fn refresh_reminders(&mut self) {
        let today = Local::now().date_naive();
        let mut new_queue = vec![];

        for a in &self.assignments {
            if let Ok(due) = NaiveDate::parse_from_str(&a.due_date, "%Y-%m-%d") {
                // ✅ Trigger for today or tomorrow
                if !a.completed && (due == today || due == today + Duration::days(1)) {
                    if !self.dismissed_reminders.contains(&a.title) {
                        new_queue.push(a.title.clone());
                    }
                }
            }
        }

        // If we found new reminders, mark popup to show
        if !new_queue.is_empty() {
            self.show_reminder_popup = true;
        }

        self.reminder_queue = new_queue;
    }

    /// Generate message for current reminder
    pub fn current_reminder_text(&self) -> Option<String> {
        let today = Local::now().date_naive();
        if let Some(title) = self.reminder_queue.first() {
            if let Some(a) = self.assignments.iter().find(|x| &x.title == title) {
                if let Ok(due) = NaiveDate::parse_from_str(&a.due_date, "%Y-%m-%d") {
                    let day_text = if due == today { "today" } else { "tomorrow" };
                    return Some(format!(
                        "⚠ '{}' is due {} at {}",
                        a.title, day_text, a.due_time
                    ));
                }
            }
        }
        None
    }

    /// Display popup reminder window (runs every frame)
    pub fn show_reminder_popup(&mut self, ctx: &egui::Context) {
        ctx.request_repaint(); // 🔁 ensures the popup stays rendered

        if self.show_reminder_popup {
            if let Some(msg) = self.current_reminder_text() {
                egui::Window::new("📅 Reminder")
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(msg);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Got it").clicked() {
                                if !self.reminder_queue.is_empty() {
                                    let done = self.reminder_queue.remove(0);
                                    self.dismissed_reminders.push(done);
                                }
                                if self.reminder_queue.is_empty() {
                                    self.show_reminder_popup = false;
                                }
                            }

                            if ui.button("Close").clicked() {
                                if let Some(title) = self.reminder_queue.first().cloned() {
                                    self.dismissed_reminders.push(title);
                                }
                                self.reminder_queue.clear();
                                self.show_reminder_popup = false;
                            }
                        });
                    });
            }
        }
    }
}
