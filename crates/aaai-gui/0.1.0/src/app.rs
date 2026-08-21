//! Top-level application state and message dispatcher.

use std::path::PathBuf;

use iced::{Subscription, Task};
use snora::{AppLayout, Toast, ToastIntent, ToastPosition, render};

use aaai_core::{
    AuditDefinition, AuditEngine, AuditResult, DiffEngine,
    config::{definition::{AuditEntry, AuditStrategy}, io as config_io},
};

use crate::views::{opening, main_view};

// ── Application state ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Opening,
    Main,
}

pub struct App {
    pub screen: Screen,

    // ── Opening screen fields
    pub before_path: String,
    pub after_path: String,
    pub definition_path: String,
    pub open_error: Option<String>,

    // ── Main screen state
    pub diffs: Vec<aaai_core::DiffEntry>,
    pub audit_result: Option<AuditResult>,
    pub definition: Option<AuditDefinition>,
    pub selected_index: Option<usize>,

    // ── Inspector / editing state
    pub inspector: InspectorState,

    // ── Unsaved changes
    pub dirty: bool,

    // ── Toasts
    pub toasts: Vec<Toast<Message>>,
    pub toast_id_counter: u64,

    // ── Dialog
    pub dialog: Option<DialogState>,

    // ── Status bar message
    pub status_msg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InspectorState {
    pub reason: String,
    pub strategy_label: String,
    pub strategy: AuditStrategy,
    pub note: String,
    pub validation_error: Option<String>,
}

impl Default for InspectorState {
    fn default() -> Self {
        InspectorState {
            reason: String::new(),
            strategy_label: "None".into(),
            strategy: AuditStrategy::None,
            note: String::new(),
            validation_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DialogState {
    ConfirmSave,
    ConfirmClose,
    ReportSaved(String),
}

impl Default for App {
    fn default() -> Self {
        App {
            screen: Screen::Opening,
            before_path: String::new(),
            after_path: String::new(),
            definition_path: String::new(),
            open_error: None,
            diffs: Vec::new(),
            audit_result: None,
            definition: None,
            selected_index: None,
            inspector: InspectorState::default(),
            dirty: false,
            toasts: Vec::new(),
            toast_id_counter: 0,
            dialog: None,
            status_msg: None,
        }
    }
}

// ── Messages ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    // Opening screen
    BeforePathChanged(String),
    AfterPathChanged(String),
    DefinitionPathChanged(String),
    StartAudit,
    NewDefinition,

    // File tree
    SelectEntry(usize),

    // Inspector
    ReasonChanged(String),
    NoteChanged(String),
    StrategySelected(String),
    // LineMatch rule editing
    AddLineRule,
    RemoveLineRule(usize),
    LineRuleActionChanged(usize, String),
    LineRuleLineChanged(usize, String),
    // Checksum field
    ChecksumChanged(String),
    // Regex fields
    RegexPatternChanged(String),
    RegexTargetChanged(String),
    // Exact field
    ExactContentChanged(String),

    // Actions
    ApproveEntry,
    RerunAudit,
    SaveDefinition,
    ExportReport(String), // "markdown" or "json"

    // Dialog / modal
    CloseModals,
    CloseMenus,

    // Toast
    DismissToast(u64),
    ToastTick,
}

// ── Update ────────────────────────────────────────────────────────────────

impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // ── Opening screen ─────────────────────────────────────────
            Message::BeforePathChanged(s) => { self.before_path = s; }
            Message::AfterPathChanged(s) => { self.after_path = s; }
            Message::DefinitionPathChanged(s) => { self.definition_path = s; }

            Message::StartAudit => {
                self.open_error = None;
                let before = PathBuf::from(&self.before_path);
                let after = PathBuf::from(&self.after_path);
                let def_path = PathBuf::from(&self.definition_path);

                // Validate
                if !before.is_dir() {
                    self.open_error = Some(format!(
                        "Before folder not found: {}", before.display()
                    ));
                    return Task::none();
                }
                if !after.is_dir() {
                    self.open_error = Some(format!(
                        "After folder not found: {}", after.display()
                    ));
                    return Task::none();
                }

                // Load or create definition
                let definition = if def_path.exists() {
                    match config_io::load(&def_path) {
                        Ok(d) => d,
                        Err(e) => {
                            self.open_error = Some(format!("Failed to load definition: {e}"));
                            return Task::none();
                        }
                    }
                } else {
                    AuditDefinition::new_empty()
                };

                // Run diff + audit
                match DiffEngine::compare(&before, &after) {
                    Ok(diffs) => {
                        let result = AuditEngine::evaluate(&diffs, &definition);
                        self.diffs = diffs;
                        self.audit_result = Some(result);
                        self.definition = Some(definition);
                        self.screen = Screen::Main;
                        self.selected_index = None;
                        self.dirty = false;
                    }
                    Err(e) => {
                        self.open_error = Some(format!("Diff failed: {e}"));
                    }
                }
            }

            Message::NewDefinition => {
                self.definition_path = String::new();
                self.open_error = None;
            }

            // ── File tree selection ────────────────────────────────────
            Message::SelectEntry(idx) => {
                self.selected_index = Some(idx);
                // Populate inspector from the selected entry.
                if let Some(result) = self.audit_result.as_ref() {
                    if let Some(far) = result.results.get(idx) {
                        if let Some(entry) = &far.entry {
                            self.inspector = InspectorState {
                                reason: entry.reason.clone(),
                                strategy_label: entry.strategy.label().into(),
                                strategy: entry.strategy.clone(),
                                note: entry.note.clone().unwrap_or_default(),
                                validation_error: None,
                            };
                        } else {
                            self.inspector = InspectorState::default();
                        }
                    }
                }
            }

            // ── Inspector edits ────────────────────────────────────────
            Message::ReasonChanged(s) => {
                self.inspector.reason = s;
                self.validate_inspector();
            }
            Message::NoteChanged(s) => { self.inspector.note = s; }

            Message::StrategySelected(label) => {
                self.inspector.strategy_label = label.clone();
                self.inspector.strategy = match label.as_str() {
                    "None"      => AuditStrategy::None,
                    "Checksum"  => AuditStrategy::Checksum { expected_sha256: String::new() },
                    "LineMatch" => AuditStrategy::LineMatch { rules: Vec::new() },
                    "Regex"     => AuditStrategy::Regex {
                        pattern: String::new(),
                        target: aaai_core::config::definition::RegexTarget::AddedLines,
                    },
                    "Exact"     => AuditStrategy::Exact { expected_content: String::new() },
                    _           => AuditStrategy::None,
                };
                self.validate_inspector();
            }

            Message::ChecksumChanged(s) => {
                if let AuditStrategy::Checksum { expected_sha256 } = &mut self.inspector.strategy {
                    *expected_sha256 = s;
                }
                self.validate_inspector();
            }

            Message::RegexPatternChanged(s) => {
                if let AuditStrategy::Regex { pattern, .. } = &mut self.inspector.strategy {
                    *pattern = s;
                }
                self.validate_inspector();
            }

            Message::RegexTargetChanged(s) => {
                if let AuditStrategy::Regex { target, .. } = &mut self.inspector.strategy {
                    use aaai_core::config::definition::RegexTarget;
                    *target = match s.as_str() {
                        "Added lines"       => RegexTarget::AddedLines,
                        "Removed lines"     => RegexTarget::RemovedLines,
                        "All changed lines" => RegexTarget::AllChangedLines,
                        _                   => RegexTarget::AddedLines,
                    };
                }
            }

            Message::AddLineRule => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    rules.push(aaai_core::config::definition::LineRule {
                        action: aaai_core::config::definition::LineAction::Added,
                        line: String::new(),
                    });
                }
            }

            Message::RemoveLineRule(i) => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    if i < rules.len() { rules.remove(i); }
                }
                self.validate_inspector();
            }

            Message::LineRuleActionChanged(i, s) => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    if let Some(r) = rules.get_mut(i) {
                        use aaai_core::config::definition::LineAction;
                        r.action = if s == "Removed" { LineAction::Removed } else { LineAction::Added };
                    }
                }
            }

            Message::LineRuleLineChanged(i, s) => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    if let Some(r) = rules.get_mut(i) { r.line = s; }
                }
                self.validate_inspector();
            }

            Message::ExactContentChanged(s) => {
                if let AuditStrategy::Exact { expected_content } = &mut self.inspector.strategy {
                    *expected_content = s;
                }
                self.validate_inspector();
            }

            // ── Approve ───────────────────────────────────────────────
            Message::ApproveEntry => {
                if let Some(idx) = self.selected_index {
                    if let Some(result) = &self.audit_result {
                        if let Some(far) = result.results.get(idx) {
                            // Validate
                            let entry = AuditEntry {
                                path: far.diff.path.clone(),
                                diff_type: far.diff.diff_type,
                                reason: self.inspector.reason.trim().to_string(),
                                strategy: self.inspector.strategy.clone(),
                                enabled: true,
                                note: {
                                    let n = self.inspector.note.trim().to_string();
                                    if n.is_empty() { None } else { Some(n) }
                                },
                            };
                            match entry.is_approvable() {
                                Ok(()) => {
                                    if let Some(def) = &mut self.definition {
                                        def.upsert_entry(entry);
                                        self.dirty = true;
                                        self.push_toast(
                                            ToastIntent::Success,
                                            "Approved",
                                            &format!("Entry approved: {}", far.diff.path),
                                        );
                                        // Re-run audit in place.
                                        self.rerun_audit();
                                    }
                                }
                                Err(e) => {
                                    self.inspector.validation_error = Some(e);
                                }
                            }
                        }
                    }
                }
            }

            // ── Re-run ────────────────────────────────────────────────
            Message::RerunAudit => {
                self.rerun_audit();
                self.push_toast(ToastIntent::Info, "Re-run", "Audit re-evaluated.");
            }

            // ── Save ──────────────────────────────────────────────────
            Message::SaveDefinition => {
                let path = PathBuf::from(&self.definition_path);
                if path.as_os_str().is_empty() {
                    self.push_toast(ToastIntent::Error, "Save failed", "No definition file path set.");
                    return Task::none();
                }
                if let Some(def) = &self.definition {
                    match config_io::save(def, &path, true) {
                        Ok(()) => {
                            self.dirty = false;
                            self.push_toast(
                                ToastIntent::Success,
                                "Saved",
                                &format!("Saved to {}", path.display()),
                            );
                        }
                        Err(e) => {
                            self.push_toast(
                                ToastIntent::Error,
                                "Save failed",
                                &e.to_string(),
                            );
                        }
                    }
                }
            }

            // ── Report export ─────────────────────────────────────────
            Message::ExportReport(fmt) => {
                if let Some(result) = &self.audit_result {
                    let before = PathBuf::from(&self.before_path);
                    let after = PathBuf::from(&self.after_path);
                    let def_path_str = &self.definition_path;
                    let def_path = if def_path_str.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(def_path_str))
                    };

                    let ext = if fmt == "json" { "json" } else { "md" };
                    let out = PathBuf::from(format!("aaai-report.{ext}"));

                    let res = match fmt.as_str() {
                        "json" => aaai_core::report::generator::ReportGenerator::write_json(
                            result, &before, &after, def_path.as_deref(), &out,
                        ),
                        _ => aaai_core::report::generator::ReportGenerator::write_markdown(
                            result, &before, &after, def_path.as_deref(), &out,
                        ),
                    };
                    match res {
                        Ok(()) => self.push_toast(
                            ToastIntent::Success,
                            "Report exported",
                            &format!("Saved to {}", out.display()),
                        ),
                        Err(e) => self.push_toast(
                            ToastIntent::Error,
                            "Export failed",
                            &e.to_string(),
                        ),
                    }
                }
            }

            // ── Modals / menus ────────────────────────────────────────
            Message::CloseModals => { self.dialog = None; }
            Message::CloseMenus  => {}

            // ── Toasts ────────────────────────────────────────────────
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, std::time::Instant::now());
            }
        }
        Task::none()
    }

    fn validate_inspector(&mut self) {
        self.inspector.validation_error = self.inspector.strategy.validate().err();
    }

    fn rerun_audit(&mut self) {
        if let (Some(def), diffs) = (&self.definition, &self.diffs) {
            let result = AuditEngine::evaluate(diffs, def);
            self.audit_result = Some(result);
        }
    }

    fn push_toast(&mut self, intent: ToastIntent, title: &str, body: &str) {
        let id = self.toast_id_counter;
        self.toast_id_counter += 1;
        self.toasts.push(Toast::new(
            id, intent,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        ));
    }

    // ── View ──────────────────────────────────────────────────────────────

    pub fn view(&self) -> iced::Element<'_, Message> {
        let body = match self.screen {
            Screen::Opening => opening::view(self),
            Screen::Main    => main_view::view(self),
        };

        let footer = self.view_footer();

        let layout = AppLayout::new(body)
            .footer(footer)
            .toasts(self.toasts.clone())
            .toast_position(ToastPosition::BottomEnd)
            .on_close_modals(Message::CloseModals)
            .on_close_menus(Message::CloseMenus);

        render(layout)
    }

    fn view_footer(&self) -> iced::Element<'_, Message> {
        use iced::{Alignment::Center, Length, widget::{container, row, space, text}};
        use crate::style::panel_style;

        let left: iced::Element<'_, Message> = if self.dirty {
            text("● Unsaved changes").size(12)
                .color(iced::Color::from_rgb(0.85, 0.45, 0.10))
                .into()
        } else {
            text("").size(12).into()
        };

        let right: iced::Element<'_, Message> =
            text("aaai v0.1.0").size(12).into();

        container(
            row![
                left,
                space().width(Length::Fill),
                right,
            ]
            .align_y(Center)
            .spacing(8),
        )
        .width(Length::Fill)
        .padding(iced::Padding::from([4.0, 16.0]))
        .style(panel_style)
        .into()
    }
}

impl App {
    /// Subscription for toast TTL ticks.
    #[allow(dead_code)]
    pub fn subscription(&self) -> Subscription<Message> {
        snora::toast::subscription(&self.toasts, || Message::ToastTick)
    }
}
// suppress false-positive dead_code for fields reserved for future use
