pub mod assignments_tab;
pub mod banner;
pub mod dashboard;
pub mod reminders;
pub mod settings_tab;

use eframe::{egui, App, Frame};
use egui::TextureHandle;
use std::sync::Arc;

use crate::assignment::Assignment;

pub struct TrackerApp {
    pub assignments: Vec<Assignment>,
    pub subject_input: String,
    pub title_input: String,
    pub due_date_input: String,
    pub due_time_input: String,
    pub show_about: bool,
    pub active_tab: String,
    pub banner_texture: Option<Arc<TextureHandle>>,
    pub banner_path: String,

    pub show_reminder_popup: bool,
    pub reminder_queue: Vec<String>,
    pub dismissed_reminders: Vec<String>,

    pub scroll_offset: f32,
    pub search_query: String,
}

impl TrackerApp {
    pub fn new() -> Self {
        Self {
            assignments: Assignment::load_from_file(),
            subject_input: String::new(),
            title_input: String::new(),
            due_date_input: String::new(),
            due_time_input: "23:59".to_string(),
            show_about: false,
            active_tab: "🏠 Dashboard".into(),
            banner_texture: None,
            banner_path: "static/banner.jpg".into(),
            show_reminder_popup: false,
            reminder_queue: vec![],
            dismissed_reminders: vec![],
            scroll_offset: 0.0,
            search_query: String::new(),
        }
    }
}

impl App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut Frame) {
        self.refresh_reminders();
        self.show_reminder_popup(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.draw_banner(ui, ctx);
                ui.separator();

                // Tabs
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 20.0;
                    for tab in ["🏠 Dashboard", "📋 Assignments", "⚙ Settings"] {
                        let active = self.active_tab == tab;
                        if ui.selectable_label(active, tab).clicked() {
                            self.active_tab = tab.to_string();
                        }
                    }
                });

                ui.separator();

                match self.active_tab.as_str() {
                    "🏠 Dashboard" => self.show_dashboard(ui),
                    "📋 Assignments" => self.show_assignments(ui),
                    "⚙ Settings" => self.show_settings(ui, ctx),
                    _ => {}
                }
            });
        });
    }
}
