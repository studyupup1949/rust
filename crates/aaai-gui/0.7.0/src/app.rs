//! Top-level application state and message dispatcher (Phase 2).
//!
//! Changes from Phase 1:
//! * Toast subscription properly wired (`App::subscription`).
//! * `FilterMode` for file-tree filtering.
//! * `BatchApproveState` for bulk approval.
//! * `locale` field + `SwitchLocale` message.
//! * `Instant` passed to `sweep_expired` correctly.

use std::path::PathBuf;
use std::time::Instant;

use iced::{Element, Subscription, Task};
use snora::{
    AppLayout, Sheet, SheetEdge, SheetSize,
    Toast, ToastIntent, ToastPosition, render,
};

use aaai_core::{
    AuditDefinition, AuditEngine, AuditResult, DiffEngine, FileAuditResult,
    AuditStatus, DiffType,
    profile::store::{AuditProfile, ProfileStore},
    config::{
        definition::{AuditEntry, AuditStrategy, LineAction, LineRule, RegexTarget},
        io as config_io,
    },
};

use crate::views::{opening, main_view};
use rust_i18n::t;

// ── Screens ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen { Opening, Main }

// ── File-tree filter ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    ChangedOnly,
    PendingOnly,
    FailedAndError,
}

impl FilterMode {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            FilterMode::All           => "filter.all",
            FilterMode::ChangedOnly   => "filter.changed",
            FilterMode::PendingOnly   => "filter.pending",
            FilterMode::FailedAndError => "filter.errors",
        }
    }

    pub fn passes(self, far: &FileAuditResult) -> bool {
        match self {
            FilterMode::All => true,
            FilterMode::ChangedOnly =>
                far.diff.diff_type != DiffType::Unchanged,
            FilterMode::PendingOnly =>
                far.status == AuditStatus::Pending,
            FilterMode::FailedAndError =>
                matches!(far.status, AuditStatus::Failed | AuditStatus::Error),
        }
    }
}

// ── Batch approve state ──────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct BatchApproveState {
    pub selected: std::collections::HashSet<usize>,
    pub shared_reason: String,
    pub shared_strategy: AuditStrategy,
    pub validation_error: Option<String>,
}

// ── Inspector state ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InspectorState {
    pub reason: String,
    pub strategy_label: String,
    pub strategy: AuditStrategy,
    pub note: String,
    pub validation_error: Option<String>,
    // Phase 3
    pub ticket: String,
    pub approved_by: String,
    pub expires_at_str: String,
}

impl Default for InspectorState {
    fn default() -> Self {
        InspectorState {
            reason: String::new(),
            strategy_label: "None".into(),
            strategy: AuditStrategy::None,
            note: String::new(),
            validation_error: None,
            ticket: String::new(),
            approved_by: String::new(),
            expires_at_str: String::new(),
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,

    // Opening
    pub before_path: String,
    pub after_path: String,
    pub definition_path: String,
    pub open_error: Option<String>,

    // Main
    pub diffs: Vec<aaai_core::DiffEntry>,
    pub audit_result: Option<AuditResult>,
    pub definition: Option<AuditDefinition>,
    pub selected_index: Option<usize>,
    pub filter_mode: FilterMode,

    // Inspector
    pub inspector: InspectorState,

    // Batch
    pub batch: BatchApproveState,
    pub batch_sheet_open: bool,

    // Unsaved
    pub dirty: bool,

    // Toasts
    pub toasts: Vec<Toast<Message>>,
    pub toast_id: u64,

    // Locale
    pub locale: String,

    // Phase 3: profiles
    pub profiles: ProfileStore,
    pub profile_name_input: String,

    // Phase 3: ignore rules (loaded at audit start)
    pub ignore_path: String,

    // Phase 5: file tree search
    pub search_query: String,

    // Phase 6: undo stack (stores path of last upserted entry)
    pub undo_stack: Vec<String>,
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
            filter_mode: FilterMode::ChangedOnly,
            inspector: InspectorState::default(),
            batch: BatchApproveState::default(),
            batch_sheet_open: false,
            dirty: false,
            toasts: Vec::new(),
            toast_id: 0,
            locale: rust_i18n::locale().to_string(),
            profiles: ProfileStore::load().unwrap_or_default(),
            profile_name_input: String::new(),
            ignore_path: String::new(),
            search_query: String::new(),
            undo_stack: Vec::new(),
        }
    }
}

// ── Messages ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    // Opening
    BeforePathChanged(String),
    AfterPathChanged(String),
    DefinitionPathChanged(String),
    StartAudit,

    // File tree
    SelectEntry(usize),
    SetFilter(FilterMode),

    // Inspector
    ReasonChanged(String),
    NoteChanged(String),
    StrategySelected(String),
    ChecksumChanged(String),
    RegexPatternChanged(String),
    RegexTargetChanged(String),
    AddLineRule,
    RemoveLineRule(usize),
    LineRuleActionChanged(usize, String),
    LineRuleLineChanged(usize, String),
    ExactContentChanged(String),

    // Actions
    ApproveEntry,
    RerunAudit,
    SaveDefinition,
    ExportReport(String),

    // Batch
    ToggleBatchSelect(usize),
    BatchReasonChanged(String),
    BatchStrategySelected(String),
    OpenBatchSheet,
    CloseBatchSheet,
    CommitBatchApprove,

    // Phase 5: search
    SearchQueryChanged(String),

    // Phase 6: undo + keyboard navigation
    UndoApproval,
    SelectNext,
    SelectPrev,

    // Phase 3: inspector fields
    TicketChanged(String),
    ApprovedByChanged(String),
    ExpiresAtChanged(String),
    ApplyTemplate(String),

    // Phase 3: profiles
    IgnorePathChanged(String),
    ProfileNameChanged(String),
    SaveProfile,
    LoadProfile(usize),
    DeleteProfile(usize),

    // Locale
    SwitchLocale(String),

    // Overlays
    CloseModals,
    CloseMenus,

    // Toasts
    DismissToast(u64),
    ToastTick,
}

// ── Update ────────────────────────────────────────────────────────────────

impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // ── Opening ───────────────────────────────────────────────
            Message::BeforePathChanged(s) => { self.before_path = s; }
            Message::AfterPathChanged(s)  => { self.after_path = s; }
            Message::DefinitionPathChanged(s) => { self.definition_path = s; }

            Message::StartAudit => {
                self.open_error = None;
                let before = PathBuf::from(&self.before_path);
                let after  = PathBuf::from(&self.after_path);
                let def_path = PathBuf::from(&self.definition_path);

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

            // ── File tree ──────────────────────────────────────────────
            Message::SelectEntry(idx) => {
                self.selected_index = Some(idx);
                if let Some(far) = self.audit_result.as_ref()
                    .and_then(|r| r.results.get(idx))
                {
                    self.inspector = if let Some(entry) = &far.entry {
                        InspectorState {
                            reason: entry.reason.clone(),
                            strategy_label: entry.strategy.label().into(),
                            strategy: entry.strategy.clone(),
                            note: entry.note.clone().unwrap_or_default(),
                            validation_error: None,
                            ticket: entry.ticket.clone().unwrap_or_default(),
                            approved_by: entry.approved_by.clone().unwrap_or_default(),
                            expires_at_str: entry.expires_at.map(|d| d.to_string()).unwrap_or_default(),
                        }
                    } else {
                        InspectorState::default()
                    };
                }
            }

            Message::SetFilter(f) => {
                self.filter_mode = f;
                self.selected_index = None;
            }

            // ── Inspector ──────────────────────────────────────────────
            Message::ReasonChanged(s) => {
                self.inspector.reason = s;
                self.validate_inspector();
            }
            Message::NoteChanged(s) => { self.inspector.note = s; }

            Message::StrategySelected(label) => {
                self.inspector.strategy_label = label.clone();
                self.inspector.strategy = strategy_from_label(&label);
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
                    *target = regex_target_from_str(&s);
                }
            }
            Message::AddLineRule => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    rules.push(LineRule { action: LineAction::Added, line: String::new() });
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
                    if let Some(far) = self.audit_result.as_ref()
                        .and_then(|r| r.results.get(idx))
                    {
                        let expires_at = if self.inspector.expires_at_str.trim().is_empty() {
                                None
                            } else {
                                chrono::NaiveDate::parse_from_str(
                                    self.inspector.expires_at_str.trim(), "%Y-%m-%d"
                                ).ok()
                            };
                        let entry = AuditEntry {
                            path: far.diff.path.clone(),
                            diff_type: far.diff.diff_type,
                            reason: self.inspector.reason.trim().to_string(),
                            strategy: self.inspector.strategy.clone(),
                            enabled: true,
                            ticket: { let t = self.inspector.ticket.trim().to_string(); if t.is_empty() { None } else { Some(t) } },
                            approved_by: { let a = self.inspector.approved_by.trim().to_string(); if a.is_empty() { None } else { Some(a) } },
                            approved_at: Some(chrono::Utc::now()),
                            expires_at,
                            note: { let n = self.inspector.note.trim().to_string(); if n.is_empty() { None } else { Some(n) } },
                            created_at: None,
                            updated_at: None,
                        };
                        match entry.is_approvable() {
                            Ok(()) => {
                                let path = far.diff.path.clone();
                                if let Some(def) = &mut self.definition {
                                    let mut stamped = entry;
                                    stamped.stamp_now();
                                    let path_for_undo = stamped.path.clone();
                                    def.upsert_entry(stamped);
                                    self.undo_stack.push(path_for_undo);
                                    if self.undo_stack.len() > 20 {
                                        self.undo_stack.remove(0);
                                    }
                                    self.dirty = true;
                                    self.rerun_audit();
                                    self.push_toast(
                                        ToastIntent::Success,
                                        t!("toast.approved").as_ref(),
                                        &path,
                                    );
                                }
                            }
                            Err(e) => {
                                self.inspector.validation_error = Some(e);
                            }
                        }
                    }
                }
            }

            // ── Batch ─────────────────────────────────────────────────
            Message::ToggleBatchSelect(idx) => {
                if self.batch.selected.contains(&idx) {
                    self.batch.selected.remove(&idx);
                } else {
                    self.batch.selected.insert(idx);
                }
            }
            Message::BatchReasonChanged(s) => {
                self.batch.shared_reason = s;
            }
            Message::BatchStrategySelected(label) => {
                self.batch.shared_strategy = strategy_from_label(&label);
            }
            Message::OpenBatchSheet => {
                self.batch_sheet_open = true;
            }
            Message::CloseBatchSheet => {
                self.batch_sheet_open = false;
            }
            Message::CommitBatchApprove => {
                if self.batch.shared_reason.trim().is_empty() {
                    self.batch.validation_error =
                        Some("Reason must not be empty.".into());
                    return Task::none();
                }
                let indices: Vec<usize> =
                    self.batch.selected.iter().copied().collect();
                let mut count = 0usize;

                if let Some(result) = &self.audit_result {
                    let entries: Vec<AuditEntry> = indices
                        .iter()
                        .filter_map(|&i| result.results.get(i))
                        .map(|far| AuditEntry {
                            path: far.diff.path.clone(),
                            diff_type: far.diff.diff_type,
                            reason: self.batch.shared_reason.trim().to_string(),
                            strategy: self.batch.shared_strategy.clone(),
                            enabled: true,
                            ticket: None,
                            approved_by: None,
                            approved_at: Some(chrono::Utc::now()),
                            expires_at: None,
                            note: None,
                            created_at: None,
                            updated_at: None,
                        })
                        .collect();
                    count = entries.len();
                    if let Some(def) = &mut self.definition {
                        for entry in entries {
                            def.upsert_entry(entry);
                        }
                    }
                }

                self.dirty = true;
                self.batch.selected.clear();
                self.batch.shared_reason.clear();
                self.batch_sheet_open = false;
                self.rerun_audit();
                self.push_toast(
                    ToastIntent::Success,
                    t!("toast.batch_approved").as_ref(),
                    &format!("{} entries approved.", count),
                );
            }

            // ── Re-run / save / report ────────────────────────────────
            Message::RerunAudit => {
                self.rerun_audit();
                self.push_toast(ToastIntent::Info, t!("toast.rerun").as_ref(), "Audit re-evaluated.");
            }

            Message::SaveDefinition => {
                let path = PathBuf::from(&self.definition_path);
                if path.as_os_str().is_empty() {
                    self.push_toast(
                        ToastIntent::Error,
                        t!("toast.save_failed").as_ref(),
                        "No definition file path set.",
                    );
                    return Task::none();
                }
                if let Some(def) = &self.definition {
                    match config_io::save(def, &path, true) {
                        Ok(()) => {
                            self.dirty = false;
                            self.push_toast(
                                ToastIntent::Success,
                                t!("toast.saved").as_ref(),
                                &format!("Saved to {}", path.display()),
                            );
                        }
                        Err(e) => {
                            self.push_toast(
                                ToastIntent::Error,
                                t!("toast.save_failed").as_ref(),
                                &e.to_string(),
                            );
                        }
                    }
                }
            }

            Message::ExportReport(fmt) => {
                if let Some(result) = &self.audit_result {
                    let before = PathBuf::from(&self.before_path);
                    let after  = PathBuf::from(&self.after_path);
                    let def_path =
                        if self.definition_path.is_empty() { None }
                        else { Some(PathBuf::from(&self.definition_path)) };
                    let ext = if fmt == "json" { "json" } else { "md" };
                    let out = PathBuf::from(format!("aaai-report.{ext}"));
                    let res = match fmt.as_str() {
                        "json" => aaai_core::report::generator::ReportGenerator::write_json(
                            result, &before, &after, def_path.as_deref(), &out, None,
                        ),
                        _ => aaai_core::report::generator::ReportGenerator::write_markdown(
                            result, &before, &after, def_path.as_deref(), &out, None,
                        ),
                    };
                    match res {
                        Ok(()) => self.push_toast(
                            ToastIntent::Success,
                            t!("toast.export_ok").as_ref(),
                            &format!("Saved to {}", out.display()),
                        ),
                        Err(e) => self.push_toast(
                            ToastIntent::Error,
                            t!("toast.export_failed").as_ref(),
                            &e.to_string(),
                        ),
                    }
                }
            }

            // ── Phase 5: search ───────────────────────────────────────
            Message::SearchQueryChanged(s) => { self.search_query = s; }

            // ── Phase 6: undo + navigation ───────────────────────────
            Message::UndoApproval => {
                if let Some(path) = self.undo_stack.pop() {
                    if let Some(def) = &mut self.definition {
                        if let Some(idx) = def.entries.iter().position(|e| e.path == path) {
                            def.entries.remove(idx);
                            self.dirty = true;
                            self.rerun_audit();
                            self.push_toast(
                                ToastIntent::Info,
                                "Undo",
                                &format!("Removed approval for: {path}"),
                            );
                        }
                    }
                } else {
                    self.push_toast(ToastIntent::Info, "Undo", "Nothing to undo.");
                }
            }
            Message::SelectNext => {
                if let Some(result) = &self.audit_result {
                    let visible: Vec<usize> = result.results.iter().enumerate()
                        .filter(|(_, r)| self.filter_mode.passes(r)
                                      && r.diff.diff_type != aaai_core::DiffType::Unchanged
                                      && (self.search_query.is_empty()
                                          || r.diff.path.to_lowercase().contains(&self.search_query.to_lowercase())))
                        .map(|(i, _)| i)
                        .collect();
                    if !visible.is_empty() {
                        let next = match self.selected_index {
                            None => visible[0],
                            Some(cur) => {
                                let pos = visible.iter().position(|&i| i == cur).unwrap_or(0);
                                visible[(pos + 1) % visible.len()]
                            }
                        };
                        return self.update(Message::SelectEntry(next));
                    }
                }
            }
            Message::SelectPrev => {
                if let Some(result) = &self.audit_result {
                    let visible: Vec<usize> = result.results.iter().enumerate()
                        .filter(|(_, r)| self.filter_mode.passes(r)
                                      && r.diff.diff_type != aaai_core::DiffType::Unchanged
                                      && (self.search_query.is_empty()
                                          || r.diff.path.to_lowercase().contains(&self.search_query.to_lowercase())))
                        .map(|(i, _)| i)
                        .collect();
                    if !visible.is_empty() {
                        let prev = match self.selected_index {
                            None => *visible.last().unwrap(),
                            Some(cur) => {
                                let pos = visible.iter().position(|&i| i == cur).unwrap_or(0);
                                visible[(pos + visible.len() - 1) % visible.len()]
                            }
                        };
                        return self.update(Message::SelectEntry(prev));
                    }
                }
            }

            // ── Phase 3: inspector fields ─────────────────────────────
            Message::TicketChanged(s)     => { self.inspector.ticket = s; }
            Message::ApprovedByChanged(s) => { self.inspector.approved_by = s; }
            Message::ExpiresAtChanged(s)  => { self.inspector.expires_at_str = s; }
            Message::ApplyTemplate(id)    => {
                use aaai_core::templates::library as tmpl;
                if let Some(t) = tmpl::find(&id) {
                    self.inspector.strategy = (t.strategy)();
                    self.inspector.strategy_label = self.inspector.strategy.label().into();
                    self.validate_inspector();
                }
            }

            // ── Phase 3: profiles ─────────────────────────────────────
            Message::IgnorePathChanged(s)  => { self.ignore_path = s; }
            Message::ProfileNameChanged(s) => { self.profile_name_input = s; }
            Message::SaveProfile => {
                let name = self.profile_name_input.trim().to_string();
                if name.is_empty() {
                    self.push_toast(ToastIntent::Error, "Profile", "Profile name must not be empty.");
                    return Task::none();
                }
                let profile = AuditProfile {
                    name: name.clone(),
                    before: self.before_path.clone(),
                    after:  self.after_path.clone(),
                    definition: if self.definition_path.is_empty() { None } else { Some(self.definition_path.clone()) },
                    ignore_file: if self.ignore_path.is_empty() { None } else { Some(self.ignore_path.clone()) },
                };
                self.profiles.add(profile);
                if let Err(e) = self.profiles.save() {
                    self.push_toast(ToastIntent::Error, t!("toast.save_failed").as_ref(), &e.to_string());
                } else {
                    self.push_toast(ToastIntent::Success, t!("profile.saved").as_ref(), &name);
                    self.profile_name_input.clear();
                }
            }
            Message::LoadProfile(idx) => {
                if let Some(p) = self.profiles.profiles.get(idx).cloned() {
                    self.before_path     = p.before;
                    self.after_path      = p.after;
                    self.definition_path = p.definition.unwrap_or_default();
                    self.ignore_path     = p.ignore_file.unwrap_or_default();
                    self.push_toast(ToastIntent::Info, "Profile", "Profile loaded.");
                }
            }
            Message::DeleteProfile(idx) => {
                if let Some(p) = self.profiles.profiles.get(idx).cloned() {
                    self.profiles.remove(&p.name);
                    let _ = self.profiles.save();
                    self.push_toast(ToastIntent::Success, t!("profile.deleted").as_ref(), &p.name);
                }
            }

            // ── Locale ────────────────────────────────────────────────
            Message::SwitchLocale(code) => {
                rust_i18n::set_locale(&code);
                self.locale = code;
            }

            // ── Overlays ──────────────────────────────────────────────
            Message::CloseModals => { self.batch_sheet_open = false; }
            Message::CloseMenus  => {}

            // ── Toasts ────────────────────────────────────────────────
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
        }
        Task::none()
    }

    // ── Subscription ─────────────────────────────────────────────────────

    pub fn subscription(&self) -> Subscription<Message> {
        
        let toast_sub = snora::toast::subscription(&self.toasts, || Message::ToastTick);
        let kb_sub = iced::keyboard::listen().map(|event| {
            use iced::keyboard::{Event as KbEvent, Key, Modifiers};
            match event {
                KbEvent::KeyPressed { key, modifiers, .. } => {
                    match (key.as_ref(), modifiers) {
                        (Key::Character("s"), m) if m.contains(Modifiers::CTRL) =>
                            Message::SaveDefinition,
                        (Key::Character("r"), m) if m.contains(Modifiers::CTRL) =>
                            Message::RerunAudit,
                        (Key::Character("z"), m) if m.contains(Modifiers::CTRL) =>
                            Message::UndoApproval,
                        (Key::Named(iced::keyboard::key::Named::ArrowDown), _) =>
                            Message::SelectNext,
                        (Key::Named(iced::keyboard::key::Named::ArrowUp), _) =>
                            Message::SelectPrev,
                        _ => Message::CloseMenus, // no-op passthrough
                    }
                }
                _ => Message::CloseMenus, // no-op passthrough
            }
        });
        Subscription::batch([toast_sub, kb_sub])
    }

    // ── View ─────────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let body = match self.screen {
            Screen::Opening => opening::view(self),
            Screen::Main    => main_view::view(self),
        };

        let footer = self.view_footer();

        let mut layout = AppLayout::new(body)
            .footer(footer)
            .toasts(self.toasts.clone())
            .toast_position(ToastPosition::BottomEnd)
            .on_close_modals(Message::CloseModals)
            .on_close_menus(Message::CloseMenus);

        // Batch approve sheet
        if self.batch_sheet_open {
            let sheet_content = crate::views::batch::view(self);
            layout = layout.sheet(
                Sheet::new(sheet_content)
                    .at(SheetEdge::End)
                    .with_size(SheetSize::Pixels(380.0)),
            );
        }

        render(layout)
    }

    fn view_footer(&self) -> Element<'_, Message> {
        use iced::{Alignment::Center, Length, widget::{container, row, space, text}};
        use crate::style::panel_style;

        let locale_label = {
            use iced::widget::pick_list;
            let current = self.locale.as_str();
            let labels: Vec<&str> = crate::i18n::SUPPORTED_LOCALES.iter()
                .map(|(_, label)| *label)
                .collect();
            let current_label = crate::i18n::SUPPORTED_LOCALES
                .iter()
                .find(|(c, _)| *c == current)
                .map(|(_, l)| *l)
                .unwrap_or("English");
            pick_list(
                labels,
                Some(current_label),
                |label: &str| {
                    let code = crate::i18n::SUPPORTED_LOCALES
                        .iter()
                        .find(|(_, l)| *l == label)
                        .map(|(c, _)| c.to_string())
                        .unwrap_or_default();
                    Message::SwitchLocale(code)
                },
            )
            .text_size(11)
            .padding(2)
        };

        let left: Element<'_, Message> = if self.dirty {
            text(t!("footer.unsaved")).size(12)
                .color(iced::Color::from_rgb(0.85, 0.45, 0.10))
                .into()
        } else {
            text("").size(12).into()
        };

        container(
            row![
                left,
                space().width(Length::Fill),
                locale_label,
                text(t!("app.version")).size(11),
            ]
            .align_y(Center)
            .spacing(12),
        )
        .width(Length::Fill)
        .padding(iced::Padding::from([4.0, 16.0]))
        .style(panel_style)
        .into()
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn validate_inspector(&mut self) {
        self.inspector.validation_error = self.inspector.strategy.validate().err();
    }

    pub fn rerun_audit(&mut self) {
        if let Some(def) = &self.definition {
            let result = AuditEngine::evaluate(&self.diffs, def);
            self.audit_result = Some(result);
        }
    }

    pub fn push_toast(&mut self, intent: ToastIntent, title: &str, body: &str) {
        let id = self.toast_id;
        self.toast_id += 1;
        self.toasts.push(Toast::new(
            id, intent,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        ));
    }
}

// ── pure functions ────────────────────────────────────────────────────────

pub fn strategy_from_label(label: &str) -> AuditStrategy {
    match label {
        "Checksum"  => AuditStrategy::Checksum { expected_sha256: String::new() },
        "LineMatch" => AuditStrategy::LineMatch { rules: Vec::new() },
        "Regex"     => AuditStrategy::Regex { pattern: String::new(), target: RegexTarget::AddedLines },
        "Exact"     => AuditStrategy::Exact { expected_content: String::new() },
        _           => AuditStrategy::None,
    }
}

pub fn regex_target_from_str(s: &str) -> RegexTarget {
    match s {
        "Removed lines"     => RegexTarget::RemovedLines,
        "All changed lines" => RegexTarget::AllChangedLines,
        _                   => RegexTarget::AddedLines,
    }
}
