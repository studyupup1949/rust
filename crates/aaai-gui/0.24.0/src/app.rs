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
use iced::widget::pane_grid;
use snora::{
    AppLayout, Sheet, SheetEdge, SheetSize,
    Toast, ToastIntent, ToastPosition, render,
};

use regex::Regex as RegexCheck;
use aaai_core::{
    AuditDefinition, AuditEngine, AuditResult, DiffEngine, FileAuditResult,
    AuditStatus, DiffType, IgnoreRules,
    profile::store::{AuditProfile, ProfileStore},
    profile::prefs::{Theme as AppTheme, UserPrefs},
    config::{
        definition::{AuditEntry, AuditStrategy, LineAction, LineRule, RegexTarget},
        io as config_io,
    },
};

use crate::views::{opening, main_view};
use crate::util::StrategyKind;
use rust_i18n::t;

// ── Pane identifiers ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneKind { FileTree, Diff, Inspector }

// ── Diff view mode (RFC 011) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffViewMode {
    #[default]
    SideBySide,   // 左右差分
    Unified,      // 統合
    ChangedOnly,  // 変更のみ
}

// ── Keyboard focus (RFC 005) ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusTarget { #[default] FileTree, Search, Inspector }

// ── Opening screen validation ─────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct OpeningValidation {
    pub before_error: Option<String>,
    pub after_error:  Option<String>,
}

impl OpeningValidation {
    pub fn can_start(&self) -> bool {
        self.before_error.is_none() && self.after_error.is_none()
    }
}

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

// ── Inspector validation (RFC 002) ────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FieldError {
    pub field:   String,
    pub message: String,
    /// RFC 028 — optional next-action hint. Rendered beneath
    /// `message` in a muted style. `None` for errors where the
    /// message is self-explanatory (e.g. "cannot be empty");
    /// `Some` when the corrective action isn't trivially inferable
    /// from the message text itself.
    pub hint:    Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct InspectorValidation {
    pub reason_error:     Option<String>,
    pub strategy_errors:  Vec<FieldError>,
    pub expires_at_error: Option<String>,
}

impl InspectorValidation {
    pub fn can_approve(&self) -> bool {
        self.reason_error.is_none()
            && self.strategy_errors.is_empty()
            && self.expires_at_error.is_none()
    }
}

// ── Inspector state ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InspectorState {
    pub reason: String,
    /// RFC 012: index of the LineMatch rule currently in edit mode (None = all display mode)
    pub editing_rule: Option<usize>,
    /// RFC 009: multi-line text editor content backing the reason field.
    /// `reason` is kept in sync via `ReasonAction` handler.
    pub reason_content: iced::widget::text_editor::Content,
    pub strategy_kind: StrategyKind,
    pub strategy: AuditStrategy,
    pub note: String,
    pub validation: InspectorValidation,   // RFC 002: replaces validation_error
    // Phase 3
    pub ticket: String,
    pub approved_by: String,
    pub expires_at_str: String,
}

impl Default for InspectorState {
    fn default() -> Self {
        InspectorState {
            reason: String::new(),
            editing_rule: None,
            reason_content: iced::widget::text_editor::Content::new(),
            strategy_kind: StrategyKind::None,
            strategy: AuditStrategy::None,
            note: String::new(),
            validation: InspectorValidation::default(),
            ticket: String::new(),
            approved_by: String::new(),
            expires_at_str: String::new(),
        }
    }
}

// ── App state ─────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,

    // Phase 8: async diff state
    pub is_loading: bool,
    pub load_progress: Option<String>,

    // 最後に使用した IgnoreRules（rerun 時に再利用）
    pub active_ignore: IgnoreRules,

    // Opening
    /// RFC 015: optional settings (audit.yaml / .aaaiignore) section expansion
    pub optional_settings_expanded: bool,
    /// RFC 023: true while a drag is active over the window — flips
    /// `opening` into "drop here" hint mode.
    pub file_hovering: bool,
    pub before_path: String,
    pub after_path: String,
    pub definition_path: String,
    pub open_error: Option<crate::error::UserError>,
    pub opening_validation: OpeningValidation,

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

    // RFC 021 — screen navigation continuity
    /// True when the in-memory definition has changed since the last
    /// successful audit run, so the displayed `audit_result` may be
    /// stale. Set by handlers that mutate `self.definition` (approve,
    /// undo, inspector edits); cleared by `rerun_audit`.
    pub audit_dirty: bool,
    /// Wall-clock time of the last successful definition save. `None`
    /// until the first save. Used for the toolbar "Saved Nm ago" mark.
    pub last_saved_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Wall-clock time of the last successful report export. `None`
    /// until the first export. Used for the toolbar "Reported Nm ago" mark.
    pub last_reported_at: Option<chrono::DateTime<chrono::Utc>>,

    // Toasts
    pub toasts: Vec<Toast<Message>>,
    pub toast_id: u64,

    // Locale
    pub locale: String,

    // Phase 10: theme
    pub theme: AppTheme,

    // RFC 011: diff view tab selection
    pub diff_view_mode: DiffViewMode,

    // RFC 005: keyboard focus
    pub focus_target: FocusTarget,
    pub prefs: UserPrefs,

    // RFC 036: Settings dialog
    pub settings_open: bool,
    pub settings_draft: UserPrefs,

    // RFC 038: keyboard help overlay
    pub help_open: bool,

    // RFC 041: navigation guard (unsaved-changes confirmation)
    pub nav_guard_open: bool,

    /// RFC 046 — set when a save-as dialog is opened from NavGuardSaveAndLeave.
    /// Tells `DefinitionSavePathPicked` to call `do_leave_to_opening()` after saving.
    pub pending_leave_to_opening: bool,

    /// RFC 048 — progressive disclosure in the Inspector.
    /// `false` = show Reason + Strategy only (default for new users).
    /// `true`  = show all fields (expert mode, global across entries).
    pub advanced_inspector_expanded: bool,

    // Phase 10: resizable pane layout
    pub panes: pane_grid::State<PaneKind>,
    pub focus: Option<pane_grid::Pane>,

    // Phase 3: profiles
    pub profiles: ProfileStore,
    pub profile_name_input: String,

    // Phase 3: ignore rules (loaded at audit start)
    pub ignore_path: String,

    // Phase 5: file tree search
    pub search_query: String,

    // Phase 10: directory collapse state
    pub collapsed_dirs: std::collections::HashSet<String>,

    // Phase 6: undo stack (stores path of last upserted entry)
    pub undo_stack: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        App {
            screen: Screen::Opening,
            optional_settings_expanded: false,
            file_hovering: false,
            is_loading: false,
            load_progress: None,
            active_ignore: IgnoreRules::default(),
            before_path: String::new(),
            after_path: String::new(),
            definition_path: String::new(),
            open_error: None,
            opening_validation: OpeningValidation::default(),
            diffs: Vec::new(),
            audit_result: None,
            definition: None,
            selected_index: None,
            filter_mode: FilterMode::ChangedOnly,
            inspector: InspectorState::default(),
            batch: BatchApproveState::default(),
            batch_sheet_open: false,
            dirty: false,
            audit_dirty: false,
            last_saved_at: None,
            last_reported_at: None,
            toasts: Vec::new(),
            toast_id: 0,
            prefs: {
                // RFC 036 — load persisted settings; apply stored language immediately.
                let p = UserPrefs::load();
                if !p.language.is_empty() {
                    rust_i18n::set_locale(&p.language);
                }
                p
            },
            locale: rust_i18n::locale().to_string(),
            theme: UserPrefs::load().theme,
            settings_open: false,
            settings_draft: UserPrefs::default(),
            help_open: false,
            nav_guard_open: false,
            pending_leave_to_opening: false,
            advanced_inspector_expanded: false,
            diff_view_mode: DiffViewMode::default(),
            focus_target: FocusTarget::default(),
            profiles: ProfileStore::load().unwrap_or_default(),
            profile_name_input: String::new(),
            ignore_path: String::new(),
            search_query: String::new(),
            collapsed_dirs: std::collections::HashSet::new(),
            undo_stack: Vec::new(),
            panes: {
let (tree, _) = pane_grid::State::new(PaneKind::FileTree);
                // We'll rebuild panes in rerun_audit/DiffReady; use placeholder here.
                tree
            },
            focus: None,
        }
    }
}

// ── Messages ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
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
    ReasonAction(iced::widget::text_editor::Action),  // RFC 009
    NoteChanged(String),
    /// RFC 035 — payload changed from `String` to `StrategyKind`
    /// to support pick_list display/value separation.
    StrategySelected(StrategyKind),
    ChecksumChanged(String),
    RegexPatternChanged(String),
    /// RFC 033 — payload changed from `String` to `RegexTarget` to
    /// support pick_list display/value separation.
    RegexTargetChanged(RegexTarget),
    AddLineRule,
    EditRule(usize),   // RFC 012: toggle rule edit mode
    RemoveLineRule(usize),
    /// RFC 033 — payload changed from `String` to `LineAction`.
    LineRuleActionChanged(usize, LineAction),
    LineRuleLineChanged(usize, String),
    ExactContentChanged(String),

    // RFC 036: Settings dialog
    OpenSettings,
    CloseSettings,
    SaveSettings,
    SettingsLanguageChanged(String),
    SettingsIgnoreDirAdd,
    SettingsIgnoreDirEdit(usize, String),
    SettingsIgnoreDirRemove(usize),

    /// RFC 037 — carries the diff result from a background rerun started
    /// by `start_async_rerun()`. On Ok: re-evaluates audit + clears dirty.
    RerunDiffReady(Result<Vec<aaai_core::DiffEntry>, String>),

    // RFC 038: keyboard help overlay
    ToggleHelp,
    CloseHelp,
    /// RFC 038 — routes Escape: closes open overlays before falling through to deselect.
    EscapeKey,

    /// RFC 048 — toggle the expert fields section in the Inspector.
    ToggleAdvancedInspector,

    /// RFC 039 — removes the currently-selected OK entry from the definition,
    /// reverting it to Pending status. Triggers an async rerun.
    RevertSelectedEntry,

    // RFC 041: navigation guard messages
    NavGuardSaveAndLeave,
    NavGuardDiscardAndLeave,
    NavGuardCancel,

    // Actions
    /// Internal approval step used by [`Message::ApproveAndSave`] and batch approval.
    /// Prefer `ApproveAndSave` for direct user actions.
    ApproveEntry,
    ApproveAndSave,  // RFC 002: approve + save in one action
    RerunAudit,
    SaveDefinition,
    /// RFC 040 — opens the native save-file dialog; format derived from extension.
    ExportReport,
    ReportPathPicked(Option<std::path::PathBuf>),

    /// RFC 046 — result of the save-file dialog opened when `definition_path` is empty.
    DefinitionSavePathPicked(Option<std::path::PathBuf>),

    // Batch
    ToggleBatchSelect(usize),
    BatchReasonChanged(String),
    /// RFC 035 — payload changed from `String` to `StrategyKind`.
    BatchStrategySelected(StrategyKind),
    OpenBatchSheet,
    CloseBatchSheet,
    CommitBatchApprove,

    // Phase 5: search
    SearchQueryChanged(String),

    // Phase 10: directory collapse
    ToggleDir(String),

    // Phase 8: async diff loading
    DiffLoading(String),   // progress message (reserved for future channel-based progress)
    DiffReady(Vec<aaai_core::DiffEntry>, aaai_core::AuditDefinition, IgnoreRules),
    DiffFailed(String),

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

    // Phase 10: theme
    SetTheme(AppTheme),

    // Phase 10: pane resize
    PaneResized(pane_grid::ResizeEvent),
    PaneFocused(pane_grid::Pane),

    // RFC 015: Opening screen folder/file picker messages
    PickBeforeFolder,
    PickAfterFolder,
    PickDefinitionFile,
    PickIgnoreFile,
    BeforeFolderPicked(Option<std::path::PathBuf>),
    AfterFolderPicked(Option<std::path::PathBuf>),
    DefinitionFilePicked(Option<std::path::PathBuf>),
    IgnoreFilePicked(Option<std::path::PathBuf>),
    ToggleOptionalSettings,

    // RFC 023: drag-and-drop folder onto the Opening screen
    /// A file/folder is currently being hovered over the window. Used to
    /// switch the Opening view into "drop hint" mode.
    FileHoverEnter,
    /// The drag left the window without dropping. Clear hover state.
    FileHoverLeave,
    /// A file or folder was dropped on the window. The path may be a
    /// folder (routed to the first empty card) or a file (rejected with
    /// an inline error).
    FileDropped(std::path::PathBuf),

    // RFC 007: navigation
    BackToOpening,

    // RFC 011: diff view tab
    SetDiffViewMode(DiffViewMode),

    // RFC 005: keyboard focus messages
    DeselectEntry,
    FocusNext,
    FocusPrev,
    FocusSearch,
    FocusInspectorReason,
    Noop,

    // Locale
    SwitchLocale(String),

    // Overlays
    CloseModals,
    /// Fired by the snora ToastLayer when an outside click should close open overlays.
    /// Kept as a distinct variant (rather than aliasing `Noop`) so that
    /// snora's `on_close_menus()` callback type is self-documenting.
    CloseMenus,

    // Toasts
    DismissToast(u64),
    ToastTick,

    /// RFC 021 §3.5 — 30-second wall-clock tick. Used to refresh
    /// "Saved Nm ago" / "Reported Nm ago" relative-time labels. A
    /// no-op at the state level (it just causes a re-render).
    RelativeTimeTick,
}

// ── Update ────────────────────────────────────────────────────────────────

impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // ── Opening ───────────────────────────────────────────────
            Message::BeforePathChanged(s) => { self.before_path = s; self.validate_opening(); }
            Message::AfterPathChanged(s)  => { self.after_path = s; self.validate_opening(); }
            Message::DefinitionPathChanged(s) => { self.definition_path = s; }

            Message::StartAudit => {
                self.open_error = None;
                let before = PathBuf::from(&self.before_path);
                let after  = PathBuf::from(&self.after_path);
                let def_path = PathBuf::from(&self.definition_path);

                if !before.is_dir() {
                    self.open_error = Some(crate::error::UserError::new(
                        t!(
                            "error.opening.before_not_found.message",
                            path = before.display().to_string()
                        ),
                        t!("error.opening.before_not_found.hint"),
                    ));
                    return Task::none();
                }
                if !after.is_dir() {
                    self.open_error = Some(crate::error::UserError::new(
                        t!(
                            "error.opening.after_not_found.message",
                            path = after.display().to_string()
                        ),
                        t!("error.opening.after_not_found.hint"),
                    ));
                    return Task::none();
                }

                let definition = if def_path.exists() {
                    match config_io::load(&def_path) {
                        Ok(d) => d,
                        Err(e) => {
                            self.open_error = Some(crate::error::UserError::new(
                                t!(
                                    "error.opening.definition_load_failed.message",
                                    reason = e.to_string()
                                ),
                                t!("error.opening.definition_load_failed.hint"),
                            ));
                            return Task::none();
                        }
                    }
                } else {
                    AuditDefinition::new_empty()
                };

                // RFC 036 — Build merged ignore rules:
                // 1. Global directory ignores from app settings (always applied)
                // 2. Per-project .aaaiignore rules appended after
                let mut ignore_text = String::new();
                for dir in &self.prefs.global_ignored_dirs {
                    let dir = dir.trim();
                    if !dir.is_empty() {
                        ignore_text.push_str(&format!("{}/**\n", dir));
                    }
                }
                let ignore_path_str = self.ignore_path.trim().to_string();
                let project_file = if ignore_path_str.is_empty() {
                    before.join(".aaaiignore")
                } else {
                    std::path::PathBuf::from(&ignore_path_str)
                };
                if project_file.exists() {
                    if let Ok(project_text) = std::fs::read_to_string(&project_file) {
                        ignore_text.push('\n');
                        ignore_text.push_str(&project_text);
                    }
                }
                let ignore = IgnoreRules::from_str(&ignore_text).unwrap_or_default();

                // RFC 042 — auto-save a profile for this session so Recent
                // Projects is always current without requiring an explicit
                // "Save Profile" action.
                self.auto_save_profile();

                // Phase 8: run diff on a background thread to keep the GUI responsive.
                self.is_loading = true;
                // RFC 031 — i18n'd.
                self.load_progress = Some(t!("progress.comparing_folders").to_string());

                let ignore_for_msg = ignore.clone();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            DiffEngine::compare_with_ignore(&before, &after, &ignore)
                                .map(|diffs| (diffs, definition))
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r.map_err(|e| e.to_string()))
                    },
                    |result| match result {
                        Ok((diffs, def)) => Message::DiffReady(diffs, def, ignore_for_msg),
                        Err(e) => Message::DiffFailed(e),
                    },
                );
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
                            editing_rule: None,
                            reason_content: iced::widget::text_editor::Content::with_text(&entry.reason),
                            strategy_kind: StrategyKind::from_strategy(&entry.strategy),
                            strategy: entry.strategy.clone(),
                            note: entry.note.clone().unwrap_or_default(),
                            validation: InspectorValidation::default(),
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
                // Keep reason_content in sync when set programmatically
                self.inspector.reason_content =
                    iced::widget::text_editor::Content::with_text(&self.inspector.reason);
                self.validate_inspector();
            }

            Message::ReasonAction(action) => {
                // RFC 009: multi-line text editor for reason field
                self.inspector.reason_content.perform(action);
                self.inspector.reason = self.inspector.reason_content.text()
                    .trim_end_matches('\n').to_string();
                self.validate_inspector();
            }
            Message::NoteChanged(s) => { self.inspector.note = s; }

            Message::StrategySelected(kind) => {
                // RFC 035 — payload is already `StrategyKind`; construct the
                // default `AuditStrategy` for that variant.
                self.inspector.strategy_kind = kind;
                self.inspector.strategy = kind.to_default_strategy();
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
            Message::RegexTargetChanged(new_target) => {
                // RFC 033 — payload is already `RegexTarget`; no string parsing needed.
                if let AuditStrategy::Regex { target, .. } = &mut self.inspector.strategy {
                    *target = new_target;
                }
            }
            Message::AddLineRule => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    rules.push(LineRule { action: LineAction::Added, line: String::new() });
                }
            }
            Message::EditRule(idx) => {
                // RFC 012: toggle rule edit mode
                self.inspector.editing_rule = if self.inspector.editing_rule == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
            }

            Message::RemoveLineRule(i) => {
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    if i < rules.len() { rules.remove(i); }
                }
                self.validate_inspector();
            }
            Message::LineRuleActionChanged(i, new_action) => {
                // RFC 033 — payload is already `LineAction`; no string parsing
                // and no silent-drop on unknown variant.
                if let AuditStrategy::LineMatch { rules } = &mut self.inspector.strategy {
                    if let Some(r) = rules.get_mut(i) {
                        r.action = new_action;
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
            Message::ApproveAndSave => {
                // RFC 002: approve + save in one action.
                // Both sub-handlers currently return Task::none(); batch them
                // so that if either is ever made async the chain stays correct.
                let t1 = self.update(Message::ApproveEntry);
                let t2 = self.update(Message::SaveDefinition);
                return Task::batch([t1, t2]);
            }

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
                                    self.audit_dirty = true;
                                    self.push_toast(
                                        ToastIntent::Success,
                                        t!("toast.approved").as_ref(),
                                        &path,
                                    );

                                    // RFC 050 — auto-advance to next Pending entry so
                                    // the user can keep approving without manual navigation.
                                    let approved_path = path.clone();
                                    let next_pending: Option<usize> =
                                        self.audit_result.as_ref().and_then(|result| {
                                            let n = result.results.len();
                                            let start = (idx + 1) % n;
                                            (0..n)
                                                .map(|i| (start + i) % n)
                                                .find(|&i| {
                                                    let r = &result.results[i];
                                                    r.status == AuditStatus::Pending
                                                        && r.diff.path != approved_path
                                                })
                                        });

                                    let rerun = self.start_async_rerun();
                                    return if let Some(next_idx) = next_pending {
                                        Task::batch([
                                            rerun,
                                            Task::perform(
                                                async move { next_idx },
                                                Message::SelectEntry,
                                            ),
                                        ])
                                    } else {
                                        rerun
                                    };
                                }
                            }
                            Err(e) => {
                                self.inspector.validation.strategy_errors.push(FieldError { field: "expires_at".into(), message: e, hint: None });
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
            Message::BatchStrategySelected(kind) => {
                // RFC 035 — payload is already `StrategyKind`.
                self.batch.shared_strategy = kind.to_default_strategy();
            }
            Message::OpenBatchSheet => {
                self.batch_sheet_open = true;
            }
            Message::CloseBatchSheet => {
                self.batch_sheet_open = false;
            }
            Message::CommitBatchApprove => {
                if self.batch.shared_reason.trim().is_empty() {
                    // RFC 031 — i18n'd.
                    self.batch.validation_error =
                        Some(t!("error.batch.reason_required.message").to_string());
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
                self.audit_dirty = true;
                self.batch.selected.clear();
                self.batch.shared_reason.clear();
                self.batch_sheet_open = false;
                self.push_toast(
                    ToastIntent::Success,
                    t!("toast.batch_approved").as_ref(),
                    t!("toast.batch_approved_count", count = count.to_string()).as_ref(),
                );
                // RFC 037 — async rerun.
                return self.start_async_rerun();
            }

            // ── Re-run / save / report ────────────────────────────────
            Message::RerunAudit => {
                // RFC 037 — converted to async; toast fires from RerunDiffReady.
                return self.start_async_rerun();
            }

            Message::SaveDefinition => {
                let path = PathBuf::from(&self.definition_path);
                if path.as_os_str().is_empty() {
                    // RFC 046 — open save-as dialog instead of showing a dead-end error.
                    let title = t!("dialog.save_approvals_file").to_string();
                    return Task::perform(
                        async move {
                            rfd::AsyncFileDialog::new()
                                .set_title(title)
                                .set_file_name("audit.yaml")
                                .add_filter("YAML", &["yaml", "yml"])
                                .save_file()
                                .await
                                .map(|h| h.path().to_path_buf())
                        },
                        Message::DefinitionSavePathPicked,
                    );
                }
                if let Some(def) = &self.definition {
                    match config_io::save(def, &path, true) {
                        Ok(()) => {
                            self.dirty = false;
                            // RFC 021 §3.2 — stamp save time so toolbar
                            // can show "Saved Nm ago" until next mutation.
                            self.last_saved_at = Some(chrono::Utc::now());
                            self.push_toast(
                                ToastIntent::Success,
                                t!("toast.saved").as_ref(),
                                t!("toast.saved_to_path", path = path.display().to_string()).as_ref(),
                            );
                        }
                        Err(e) => {
                            // RFC 026 — use message+hint pattern. The
                            // raw `e.to_string()` is appended to the
                            // localized message so the user sees both
                            // a user-friendly description and the
                            // concrete OS error.
                            let user_err = crate::error::UserError::from_i18n("error.save.failed");
                            let full_message = format!("{}\n({})", user_err.message, e);
                            self.push_toast_with_hint(
                                ToastIntent::Error,
                                t!("toast.save_failed").as_ref(),
                                &full_message,
                                &user_err.hint,
                            );
                        }
                    }
                }
            }

            Message::ExportReport => {
                // RFC 040 — open native save-file dialog; format derived from extension.
                if self.audit_result.is_none() {
                    self.push_toast(
                        ToastIntent::Info,
                        t!("toast.export_failed").as_ref(),
                        t!("toast.no_audit_result").as_ref(),
                    );
                    return Task::none();
                }
                let title = t!("dialog.save_report").to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .set_file_name("aaai-report.md")
                            .add_filter("Markdown", &["md"])
                            .add_filter("JSON",     &["json"])
                            .save_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::ReportPathPicked,
                );
            }

            // RFC 046 — save-as dialog result ──────────────────────────
            Message::DefinitionSavePathPicked(None) => {
                // User cancelled the dialog — clear any pending-leave flag, no toast.
                self.pending_leave_to_opening = false;
            }

            Message::DefinitionSavePathPicked(Some(chosen)) => {
                self.definition_path = chosen.display().to_string();
                // RFC 047 — make the newly-saved path visible in Optional settings.
                self.optional_settings_expanded = true;
                if let Some(def) = &self.definition {
                    match config_io::save(def, &chosen, true) {
                        Ok(()) => {
                            self.dirty = false;
                            self.last_saved_at = Some(chrono::Utc::now());
                            self.push_toast(
                                ToastIntent::Success,
                                t!("toast.saved").as_ref(),
                                t!("toast.saved_to_path",
                                   path = chosen.display().to_string()).as_ref(),
                            );
                            if self.pending_leave_to_opening {
                                self.pending_leave_to_opening = false;
                                self.do_leave_to_opening();
                            }
                        }
                        Err(e) => {
                            let user_err = crate::error::UserError::from_i18n("error.save.failed");
                            let full_message = format!("{}\n({})", user_err.message, e);
                            self.push_toast_with_hint(
                                ToastIntent::Error,
                                t!("toast.save_failed").as_ref(),
                                &full_message,
                                &user_err.hint,
                            );
                            self.pending_leave_to_opening = false;
                        }
                    }
                }
            }

            Message::ReportPathPicked(None) => { /* user cancelled */ }

            Message::ReportPathPicked(Some(out)) => {
                if let Some(result) = &self.audit_result {
                    let before   = PathBuf::from(&self.before_path);
                    let after    = PathBuf::from(&self.after_path);
                    let def_path = if self.definition_path.is_empty() { None }
                                   else { Some(PathBuf::from(&self.definition_path)) };

                    // Derive format from chosen extension.
                    let use_json = out.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("json"))
                        .unwrap_or(false);

                    let res = if use_json {
                        aaai_core::report::generator::ReportGenerator::write_json(
                            result, &before, &after, def_path.as_deref(), &out, None,
                        )
                    } else {
                        aaai_core::report::generator::ReportGenerator::write_markdown(
                            result, &before, &after, def_path.as_deref(), &out, None,
                        )
                    };

                    match res {
                        Ok(()) => {
                            self.last_reported_at = Some(chrono::Utc::now());
                            self.push_toast(
                                ToastIntent::Success,
                                t!("toast.export_ok").as_ref(),
                                t!("toast.saved_to_path", path = out.display().to_string()).as_ref(),
                            );
                        }
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

            // ── Phase 10: directory collapse ──────────────────────────────
            Message::ToggleDir(dir) => {
                if self.collapsed_dirs.contains(&dir) {
                    self.collapsed_dirs.remove(&dir);
                } else {
                    self.collapsed_dirs.insert(dir);
                }
            }

            // ── Phase 8: async diff results ───────────────────────────────
            Message::DiffLoading(msg) => {
                self.load_progress = Some(msg);
            }
            Message::DiffReady(diffs, definition, ignore) => {
                self.is_loading = false;
                self.load_progress = None;
                // (Re)initialize 3-pane layout: FileTree | Diff | Inspector
                let (pane_state, pane_file_tree) = pane_grid::State::new(PaneKind::FileTree);
                self.panes = pane_state;
                // Split FileTree | right-column (Diff + Inspector)
                if let Some((right_pane, _)) = self.panes.split(
                    pane_grid::Axis::Vertical, pane_file_tree, PaneKind::Diff
                ) {
                    // Split Diff | Inspector
                    let _ = self.panes.split(
                        pane_grid::Axis::Vertical, right_pane, PaneKind::Inspector
                    );
                }
                let result = aaai_core::AuditEngine::evaluate(&diffs, &definition);
                self.diffs = diffs;
                self.audit_result = Some(result);
                self.definition = Some(definition);
                self.active_ignore = ignore;
                self.screen = Screen::Main;
                self.selected_index = None;
                self.dirty = false;

                // RFC 052 — auto-select the first Pending entry so the user
                // can start approving immediately without clicking in the tree.
                let first_pending: Option<usize> = self.audit_result.as_ref().and_then(|r| {
                    r.results.iter()
                        .enumerate()
                        .find(|(_, far)| far.status == AuditStatus::Pending)
                        .map(|(idx, _)| idx)
                });
                if let Some(idx) = first_pending {
                    return Task::perform(async move { idx }, Message::SelectEntry);
                }
            }
            Message::DiffFailed(err) => {
                self.is_loading = false;
                self.load_progress = None;
                self.open_error = Some(crate::error::UserError::new(
                    t!("error.diff.failed.message", reason = err),
                    t!("error.diff.failed.hint"),
                ));
            }

            // ── Phase 6: undo + navigation ───────────────────────────
            Message::UndoApproval => {
                if let Some(path) = self.undo_stack.pop() {
                    if let Some(def) = &mut self.definition {
                        if let Some(idx) = def.entries.iter().position(|e| e.path == path) {
                            def.entries.remove(idx);
                            self.dirty = true;
                            self.audit_dirty = true;
                            self.push_toast(
                                ToastIntent::Info,
                                t!("toast.undo").as_ref(),
                                t!("toast.removed_approval", path = path.clone()).as_ref(),
                            );
                            // RFC 037 — async rerun.
                            return self.start_async_rerun();
                        }
                    }
                } else {
                    self.push_toast(
                        ToastIntent::Info,
                        t!("toast.undo").as_ref(),
                        t!("toast.nothing_to_undo").as_ref(),
                    );
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

            // ── Phase 10: theme ───────────────────────────────────────
            Message::SetTheme(t) => {
                self.theme = t;
                self.prefs.theme = t;
                self.prefs.save();
            }

            // ── Phase 10: pane resize ─────────────────────────────────
            Message::PaneResized(e) => { self.panes.resize(e.split, e.ratio); }
            Message::PaneFocused(p) => { self.focus = Some(p); }

            // ── RFC 005: keyboard focus ───────────────────────────────────
            Message::Noop => {}
            Message::DeselectEntry => { self.selected_index = None; }

            // ── RFC 011: diff view mode ───────────────────────────────
            Message::SetDiffViewMode(mode) => {
                self.diff_view_mode = mode;
            }


            // ── RFC 015: Opening picker handlers ─────────────────────
            Message::PickBeforeFolder => {
                let title = t!("dialog.pick_before").to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::BeforeFolderPicked,
                );
            }
            Message::PickAfterFolder => {
                let title = t!("dialog.pick_after").to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .pick_folder()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::AfterFolderPicked,
                );
            }
            Message::PickDefinitionFile => {
                let title = t!("dialog.pick_audit_yaml").to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter("YAML", &["yaml", "yml"])
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::DefinitionFilePicked,
                );
            }
            Message::PickIgnoreFile => {
                let title = t!("dialog.pick_aaaiignore").to_string();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .pick_file()
                            .await
                            .map(|h| h.path().to_path_buf())
                    },
                    Message::IgnoreFilePicked,
                );
            }
            Message::BeforeFolderPicked(opt) => {
                if let Some(path) = opt {
                    self.before_path = path.display().to_string();
                    self.validate_opening();
                }
            }
            Message::AfterFolderPicked(opt) => {
                if let Some(path) = opt {
                    self.after_path = path.display().to_string();
                    self.validate_opening();
                }
            }
            Message::DefinitionFilePicked(opt) => {
                if let Some(path) = opt {
                    self.definition_path = path.display().to_string();
                    self.validate_opening();
                }
            }
            Message::IgnoreFilePicked(opt) => {
                if let Some(path) = opt {
                    self.ignore_path = path.display().to_string();
                    self.validate_opening();
                }
            }
            Message::ToggleOptionalSettings => {
                self.optional_settings_expanded = !self.optional_settings_expanded;
            }

            // ── RFC 023: drag-and-drop on the Opening screen ──────────
            Message::FileHoverEnter => {
                // Only meaningful while the user is on Opening. We don't
                // restrict by `self.screen` here because the iced event
                // arrives globally; the Opening view itself ignores the
                // `file_hovering` flag on other screens.
                self.file_hovering = true;
            }
            Message::FileHoverLeave => {
                self.file_hovering = false;
            }
            Message::FileDropped(path) => {
                self.file_hovering = false;
                // Only act on Opening — on the Main screen the drop is
                // ignored to avoid surprising the user mid-audit.
                if self.screen != Screen::Opening {
                    return Task::none();
                }
                if !path.is_dir() {
                    // RFC 023 FR-3: non-folder drops surface as inline
                    // error via the open_error banner (RFC 020 pattern).
                    self.open_error = Some(crate::error::UserError::new(
                        t!(
                            "error.opening.drop_invalid_kind.message",
                            path = path.display().to_string()
                        ),
                        t!("error.opening.drop_invalid_kind.hint"),
                    ));
                    return Task::none();
                }
                // Route to the first empty card; if both filled, route to
                // Before (the user can re-drag for After). This is the
                // simplest mapping that satisfies RFC 023 FR-1 without
                // needing layout-coordinate hit-testing.
                let target = path.display().to_string();
                if self.before_path.trim().is_empty() {
                    self.before_path = target;
                } else if self.after_path.trim().is_empty() {
                    self.after_path = target;
                } else {
                    // Both are set — overwrite Before by convention.
                    self.before_path = target;
                }
                self.validate_opening();
            }

            // ── RFC 007: navigation ───────────────────────────────────
            Message::BackToOpening => {
                if self.dirty {
                    // RFC 041 — open confirmation dialog instead of passive toast.
                    self.nav_guard_open = true;
                } else {
                    self.do_leave_to_opening();
                }
            }
            Message::FocusNext => {
                self.focus_target = match self.focus_target {
                    FocusTarget::FileTree  => FocusTarget::Inspector,
                    FocusTarget::Inspector => FocusTarget::FileTree,
                    FocusTarget::Search    => FocusTarget::FileTree,
                };
            }
            Message::FocusPrev => {
                self.focus_target = match self.focus_target {
                    FocusTarget::FileTree  => FocusTarget::Inspector,
                    FocusTarget::Inspector => FocusTarget::FileTree,
                    FocusTarget::Search    => FocusTarget::Inspector,
                };
            }
            Message::FocusSearch => {
                // RFC 005: update logical focus; visual focus ring shown by search input
                self.focus_target = FocusTarget::Search;
            }
            Message::FocusInspectorReason => {
                // RFC 005: update logical focus; inspector reason input highlighted
                self.focus_target = FocusTarget::Inspector;
            }

            // ── Phase 3: inspector fields ─────────────────────────────
            Message::TicketChanged(s)     => { self.inspector.ticket = s; }
            Message::ApprovedByChanged(s) => { self.inspector.approved_by = s; }
            Message::ExpiresAtChanged(s)  => { self.inspector.expires_at_str = s; }
            Message::ApplyTemplate(id)    => {
                use aaai_core::templates::library as tmpl;
                if let Some(t) = tmpl::find(&id) {
                    self.inspector.strategy = (t.strategy)();
                    self.inspector.strategy_kind = StrategyKind::from_strategy(&self.inspector.strategy);
                    self.validate_inspector();
                }
            }

            // ── Phase 3: profiles ─────────────────────────────────────
            Message::IgnorePathChanged(s)  => { self.ignore_path = s; }
            Message::ProfileNameChanged(s) => { self.profile_name_input = s; }
            Message::SaveProfile => {
                let name = self.profile_name_input.trim().to_string();
                if name.is_empty() {
                    self.push_toast(
                        ToastIntent::Error,
                        t!("toast.profile").as_ref(),
                        t!("toast.profile_name_empty").as_ref(),
                    );
                    return Task::none();
                }
                let profile = AuditProfile {
                    name: name.clone(),
                    before: self.before_path.clone(),
                    after:  self.after_path.clone(),
                    definition: if self.definition_path.is_empty() { None } else { Some(self.definition_path.clone()) },
                    ignore_file: if self.ignore_path.is_empty() { None } else { Some(self.ignore_path.clone()) },
                    // RFC 023 §3.2: new profiles start un-touched. The
                    // first LoadProfile or explicit re-save will stamp this.
                    last_used_at: None,
                };
                self.profiles.add(profile);
                if let Err(e) = self.profiles.save() {
                    // RFC 026 — message+hint pattern. The same
                    // "couldn't write" hint applies: the user's
                    // recourse is the same.
                    let user_err = crate::error::UserError::from_i18n("error.save.failed");
                    let full_message = format!("{}\n({})", user_err.message, e);
                    self.push_toast_with_hint(
                        ToastIntent::Error,
                        t!("toast.save_failed").as_ref(),
                        &full_message,
                        &user_err.hint,
                    );
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
                    // RFC 047 — auto-expand so the user can see which approvals file loaded.
                    if !self.definition_path.is_empty() {
                        self.optional_settings_expanded = true;
                    }
                    // RFC 023 FR-6: stamp last_used_at when loading so the
                    // Recent list re-orders on next view. We swallow the
                    // I/O error: failing to persist the timestamp must not
                    // block the user from continuing into the audit.
                    let _ = self.profiles.touch(&p.name);
                    self.push_toast(
                        ToastIntent::Info,
                        t!("toast.profile").as_ref(),
                        t!("toast.profile_loaded").as_ref(),
                    );
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

            // RFC 037 — async rerun completion ────────────────────────
            Message::RerunDiffReady(result) => {
                self.is_loading = false;
                self.load_progress = None;
                match result {
                    Ok(fresh_diffs) => {
                        self.diffs = fresh_diffs;
                        if let Some(def) = self.definition.clone() {
                            self.audit_result = Some(AuditEngine::evaluate(&self.diffs, &def));
                        }
                        self.audit_dirty = false;
                        self.push_toast(
                            ToastIntent::Info,
                            t!("toast.rerun").as_ref(),
                            t!("toast.rerun_complete").as_ref(),
                        );
                    }
                    Err(e) => {
                        self.push_toast(
                            ToastIntent::Error,
                            t!("toast.rerun").as_ref(),
                            &e,
                        );
                    }
                }
            }

            // ── RFC 041: navigation guard ─────────────────────────────
            Message::NavGuardCancel => { self.nav_guard_open = false; }

            Message::NavGuardDiscardAndLeave => {
                self.nav_guard_open = false;
                self.dirty = false;
                self.do_leave_to_opening();
            }

            Message::NavGuardSaveAndLeave => {
                // Inline save; navigate on success, show error and stay on failure.
                let path = PathBuf::from(&self.definition_path);
                if path.as_os_str().is_empty() {
                    // RFC 046 — open save-as dialog; navigate after a successful pick+save.
                    self.nav_guard_open = false;
                    self.pending_leave_to_opening = true;
                    let title = t!("dialog.save_approvals_file").to_string();
                    return Task::perform(
                        async move {
                            rfd::AsyncFileDialog::new()
                                .set_title(title)
                                .set_file_name("audit.yaml")
                                .add_filter("YAML", &["yaml", "yml"])
                                .save_file()
                                .await
                                .map(|h| h.path().to_path_buf())
                        },
                        Message::DefinitionSavePathPicked,
                    );
                }
                if let Some(def) = &self.definition {
                    match config_io::save(def, &path, true) {
                        Ok(()) => {
                            self.dirty = false;
                            self.last_saved_at = Some(chrono::Utc::now());
                            self.nav_guard_open = false;
                            self.do_leave_to_opening();
                        }
                        Err(e) => {
                            let user_err = crate::error::UserError::from_i18n("error.save.failed");
                            let full_message = format!("{}\n({})", user_err.message, e);
                            self.push_toast_with_hint(
                                ToastIntent::Error,
                                t!("toast.save_failed").as_ref(),
                                &full_message,
                                &user_err.hint,
                            );
                            self.nav_guard_open = false;
                            // Do NOT navigate — user must resolve the save error.
                        }
                    }
                }
            }

            // ── RFC 038: keyboard help overlay ────────────────────────
            Message::ToggleHelp  => { self.help_open = !self.help_open; }
            Message::CloseHelp   => { self.help_open = false; }
            Message::EscapeKey   => {
                // Prioritise overlay-close before falling through to deselect.
                if self.help_open        { self.help_open = false; }
                else if self.nav_guard_open { self.nav_guard_open = false; }
                else if self.settings_open  { self.settings_open = false; }
                else { self.selected_index = None; }
            }

            // RFC 048 — Inspector progressive disclosure ───────────────
            Message::ToggleAdvancedInspector => {
                self.advanced_inspector_expanded = !self.advanced_inspector_expanded;
            }

            // RFC 039 — Revert selected OK entry to Pending ───────────
            Message::RevertSelectedEntry => {
                if let (Some(idx), Some(def)) = (self.selected_index, &mut self.definition) {
                    if let Some(diff) = self.diffs.get(idx) {
                        let path = diff.path.clone();
                        if let Some(pos) = def.entries.iter().position(|e| e.path == path) {
                            def.entries.remove(pos);
                            self.dirty = true;
                            self.push_toast(
                                ToastIntent::Info,
                                t!("toast.reverted").as_ref(),
                                t!("toast.reverted_path", path = path).as_ref(),
                            );
                            return self.start_async_rerun();
                        }
                    }
                }
            }

            // ── RFC 036: Settings dialog ──────────────────────────────
            Message::OpenSettings => {
                self.settings_draft = self.prefs.clone();
                self.settings_open = true;
            }
            Message::CloseSettings => {
                self.settings_open = false;
                // draft is abandoned; prefs remain unchanged
            }
            Message::SaveSettings => {
                self.prefs = self.settings_draft.clone();
                // Trim empty entries before saving
                self.prefs.global_ignored_dirs.retain(|d| !d.trim().is_empty());
                // Apply language change immediately
                if !self.prefs.language.is_empty() {
                    rust_i18n::set_locale(&self.prefs.language);
                    self.locale = self.prefs.language.clone();
                }
                self.prefs.save();
                self.settings_open = false;
            }
            Message::SettingsLanguageChanged(code) => {
                self.settings_draft.language = code;
            }
            Message::SettingsIgnoreDirAdd => {
                self.settings_draft.global_ignored_dirs.push(String::new());
            }
            Message::SettingsIgnoreDirEdit(i, s) => {
                if let Some(entry) = self.settings_draft.global_ignored_dirs.get_mut(i) {
                    *entry = s;
                }
            }
            Message::SettingsIgnoreDirRemove(i) => {
                let dirs = &mut self.settings_draft.global_ignored_dirs;
                if i < dirs.len() { dirs.remove(i); }
            }

            // ── Overlays ──────────────────────────────────────────────
            Message::CloseModals => { self.batch_sheet_open = false; }
            Message::CloseMenus  => { /* snora overlay close — no state change needed */ }

            // ── Toasts ────────────────────────────────────────────────
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
            Message::RelativeTimeTick => {
                // No-op at the state level — receiving this message
                // causes iced to re-render, which is enough to refresh
                // the "Saved Nm ago" labels through humanize_since.
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
                        (Key::Character("z"), m) if m.contains(Modifiers::CTRL) && m.contains(Modifiers::SHIFT) =>
                            Message::RevertSelectedEntry,
                        (Key::Character("z"), m) if m.contains(Modifiers::CTRL) =>
                            Message::UndoApproval,
                        // RFC 005: Ctrl+E → export report
                        (Key::Character("e"), m) if m.contains(Modifiers::CTRL) =>
                            Message::ExportReport,
                        (Key::Named(iced::keyboard::key::Named::ArrowDown), _) =>
                            Message::SelectNext,
                        (Key::Named(iced::keyboard::key::Named::ArrowUp), _) =>
                            Message::SelectPrev,
                        // RFC 005: Tab / Shift+Tab for pane focus cycling
                        (Key::Named(iced::keyboard::key::Named::Tab), m)
                            if m.contains(Modifiers::SHIFT) =>
                            Message::FocusPrev,
                        (Key::Named(iced::keyboard::key::Named::Tab), _) =>
                            Message::FocusNext,
                        // RFC 005: / key → focus search
                        (Key::Character("/"), m)
                            if !m.contains(Modifiers::CTRL) && !m.contains(Modifiers::ALT) =>
                            Message::FocusSearch,
                        // RFC 051 — Ctrl+Enter submits approval (the reason text is
                        // trimmed in the handler, so an accidental trailing newline
                        // from the text_editor is harmless).
                        (Key::Named(iced::keyboard::key::Named::Enter), m)
                            if m.contains(Modifiers::CTRL) =>
                            Message::ApproveAndSave,
                        // RFC 005: Enter → focus inspector reason
                        (Key::Named(iced::keyboard::key::Named::Enter), _) =>
                            Message::FocusInspectorReason,
                        // RFC 038: ? key → toggle keyboard help overlay
                        (Key::Character("?"), _) => Message::ToggleHelp,
                        // Escape — handled via EscapeKey to avoid capturing self in the closure
                        (Key::Named(iced::keyboard::key::Named::Escape), _) =>
                            Message::EscapeKey,
                        _ => Message::Noop,
                    }
                }
                _ => Message::Noop,
            }
        });
        // RFC 021 §3.5 — 30-second wall-clock tick. Only enabled when at
        // least one timestamp is present, so we don't burn CPU re-rendering
        // until the user has actually saved or exported once.
        let needs_tick =
            self.last_saved_at.is_some() || self.last_reported_at.is_some();
        let time_sub: Subscription<Message> = if needs_tick {
            iced::time::every(std::time::Duration::from_secs(30))
                .map(|_| Message::RelativeTimeTick)
        } else {
            Subscription::none()
        };

        Subscription::batch([toast_sub, kb_sub, dnd_sub(), time_sub])
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

        let base: Element<'_, Message> = render(layout);

        // RFC 036 — Settings dialog modal overlay
        if self.settings_open {
            use iced::{Color, Length};
            use iced::widget::{container, mouse_area, stack};

            let backdrop = mouse_area(
                container(iced::widget::space().width(Length::Fill).height(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(
                            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 }
                        )),
                        ..Default::default()
                    })
            )
            .on_press(Message::CloseSettings);

            let dialog = iced::widget::center(
                crate::views::settings_dialog::view(&self.settings_draft, &self.locale)
            );

            stack![base, backdrop, dialog].into()

        // RFC 038 — Keyboard help overlay (only on Main screen)
        } else if self.help_open && matches!(self.screen, Screen::Main) {
            use iced::{Color, Length};
            use iced::widget::{container, mouse_area, stack};

            let backdrop = mouse_area(
                container(iced::widget::space().width(Length::Fill).height(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(
                            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 }
                        )),
                        ..Default::default()
                    })
            )
            .on_press(Message::CloseHelp);

            let dialog = iced::widget::center(
                crate::views::help_overlay::view()
            );

            stack![base, backdrop, dialog].into()

        // RFC 041 — Navigation guard (only on Main screen)
        } else if self.nav_guard_open && matches!(self.screen, Screen::Main) {
            use iced::{Color, Length};
            use iced::widget::{container, mouse_area, stack};

            let backdrop = mouse_area(
                container(iced::widget::space().width(Length::Fill).height(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(
                            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.50 }
                        )),
                        ..Default::default()
                    })
            )
            .on_press(Message::NavGuardCancel);

            let dialog = iced::widget::center(
                crate::views::nav_guard::view()
            );

            stack![base, backdrop, dialog].into()

        } else {
            base
        }
    }

    fn view_footer(&self) -> Element<'_, Message> {
        use iced::{Alignment::Center, Length, widget::{button, container, row, space, text, tooltip}};
        use iced::widget::tooltip::Position;
        use crate::style::panel_style;

        // RFC 036 — language picker moved to Settings dialog.
        // RFC 038 — ? button (help overlay) + ⚙ settings button.
        let help_btn = tooltip(
            button(text("?").size(13))
                .on_press(Message::ToggleHelp)
                .padding(iced::Padding::from([2.0, 6.0]))
                .style(iced::widget::button::text),
            text(t!("help.title").to_string()).size(11),
            Position::Top,
        );

        let settings_btn = tooltip(
            button(text("⚙").size(13))
                .on_press(Message::OpenSettings)
                .padding(iced::Padding::from([2.0, 6.0]))
                .style(iced::widget::button::text),
            text(t!("settings.button_tooltip").to_string()).size(11),
            Position::Top,
        );

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
                help_btn,
                settings_btn,
                text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(11),
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

    /// RFC 002: per-field real-time validation for the inspector.
    fn validate_inspector(&mut self) {
        use aaai_core::config::definition::AuditStrategy;
        let ins = &self.inspector;
        let mut v = InspectorValidation::default();

        // Reason (required)
        if ins.reason.trim().is_empty() {
            // RFC 031 — i18n'd.
            v.reason_error = Some(t!("error.inspector.reason_required.message").to_string());
        }

        // ExpiresAt format
        if !ins.expires_at_str.trim().is_empty() {
            if chrono::NaiveDate::parse_from_str(&ins.expires_at_str, "%Y-%m-%d").is_err() {
                // RFC 031 — i18n-migrated; this was the last
                // hardcoded user-facing string in app.rs.
                v.expires_at_error = Some(t!("error.inspector.expires_at_format.message").to_string());
            }
        }

        // Strategy-specific validation
        match &ins.strategy {
            AuditStrategy::Checksum { expected_sha256 } => {
                let s = expected_sha256.trim();
                if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
                    // RFC 030 — message + actionable hint. The
                    // raw "64 hex chars" message assumes SHA-256
                    // familiarity; the hint says where the value
                    // comes from for users new to the tool.
                    let err = crate::error::UserError::from_i18n("error.inspector.invalid_sha256");
                    v.strategy_errors.push(FieldError {
                        field: "expected_sha256".into(),
                        message: err.message,
                        hint: Some(err.hint),
                    });
                }
            }
            AuditStrategy::LineMatch { rules } => {
                if rules.is_empty() {
                    // RFC 030 — message + actionable hint. New
                    // users may not realise the `+ Add rule`
                    // button below the empty rules list is where
                    // they go next.
                    let err = crate::error::UserError::from_i18n("error.inspector.empty_rules");
                    v.strategy_errors.push(FieldError {
                        field: "rules".into(),
                        message: err.message,
                        hint: Some(err.hint),
                    });
                }
                for (i, rule) in rules.iter().enumerate() {
                    if rule.line.trim().is_empty() {
                        // RFC 029 — hint stays None: the message ("rule line
                        // cannot be empty") already points at the action.
                        v.strategy_errors.push(FieldError {
                            field: format!("rule[{}].line", i),
                            message: t!("error.inspector.empty_rule_line.message").to_string(),
                            hint: None,
                        });
                    }
                }
            }
            AuditStrategy::Regex { pattern, .. } => {
                if let Err(e) = RegexCheck::new(pattern) {
                    // RFC 028 — hint is now a structural field, not
                    // composed into the message. The message line
                    // carries the localized description + the concrete
                    // syntax error from the regex compiler; the hint
                    // line carries the actionable next step
                    // (i.e. "test at regex101.com").
                    let err = crate::error::UserError::from_i18n("error.inspector.invalid_regex");
                    v.strategy_errors.push(FieldError {
                        field: "pattern".into(),
                        message: format!("{} ({})", err.message, e),
                        hint: Some(err.hint),
                    });
                }
            }
            AuditStrategy::Exact { expected_content } => {
                if expected_content.trim().is_empty() {
                    // RFC 029.
                    v.strategy_errors.push(FieldError {
                        field: "expected_content".into(),
                        message: t!("error.inspector.empty_expected.message").to_string(),
                        hint: None,
                    });
                }
            }
            AuditStrategy::None => {}
        }

        self.inspector.validation = v;
    }

    pub fn validate_opening(&mut self) {
        let mut v = OpeningValidation::default();
        let before_s = self.before_path.trim().to_string();
        let after_s  = self.after_path.trim().to_string();

        // RFC 031 — all 6 inline validation messages migrated to t!().
        // Distinct from the RFC 020 banner path's
        // error.opening.{before,after}_not_found.* keys, which carry
        // path interpolation. Inline versions are terse.
        if before_s.is_empty() {
            v.before_error = Some(t!("error.opening.before_required.message").to_string());
        } else {
            let p = std::path::Path::new(&before_s);
            if !p.exists() {
                v.before_error = Some(t!("error.opening.folder_missing.message").to_string());
            } else if !p.is_dir() {
                v.before_error = Some(t!("error.opening.not_a_directory.message").to_string());
            }
        }

        if after_s.is_empty() {
            v.after_error = Some(t!("error.opening.after_required.message").to_string());
        } else {
            let p = std::path::Path::new(&after_s);
            if !p.exists() {
                v.after_error = Some(t!("error.opening.folder_missing.message").to_string());
            } else if !p.is_dir() {
                v.after_error = Some(t!("error.opening.not_a_directory.message").to_string());
            }
        }
        self.opening_validation = v;
    }

    /// RFC 042 — silently upsert a profile for the current paths.
    ///
    /// Called at audit start so the Recent Projects list stays current
    /// without requiring explicit "Save Profile" actions. Profile name is
    /// derived from the definition file stem or before-folder name.
    /// I/O errors are swallowed — a failing auto-save must never block the audit.
    fn auto_save_profile(&mut self) {
        let name = {
            let from_def = (!self.definition_path.is_empty()).then(|| {
                std::path::Path::new(&self.definition_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            }).flatten();

            let from_before = (!self.before_path.is_empty()).then(|| {
                std::path::Path::new(&self.before_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            }).flatten();

            from_def.or(from_before).unwrap_or_else(|| "untitled".to_string())
        };

        let profile = aaai_core::profile::store::AuditProfile {
            name,
            before:      self.before_path.clone(),
            after:       self.after_path.clone(),
            definition:  if self.definition_path.is_empty() { None }
                         else { Some(self.definition_path.clone()) },
            ignore_file: if self.ignore_path.is_empty() { None }
                         else { Some(self.ignore_path.clone()) },
            last_used_at: Some(chrono::Utc::now()),
        };

        self.profiles.add(profile);
        let _ = self.profiles.save();
    }

    /// RFC 041 — centralised navigation to the Opening screen.
    /// Clears all main-screen state and closes any open overlays.
    fn do_leave_to_opening(&mut self) {
        self.screen = Screen::Opening;
        self.audit_result = None;
        self.diffs.clear();
        self.definition = None;
        self.selected_index = None;
        self.inspector = InspectorState::default();
        self.audit_dirty = false;
        self.help_open = false;
        self.nav_guard_open = false;
    }

    /// RFC 037 — Non-blocking rerun helper.
    /// Sets `is_loading = true`, `audit_dirty` stays true until the background
    /// diff completes and `RerunDiffReady` fires.  Callers should push any
    /// "what just happened" toast *before* returning this task so that the
    /// toast is immediately visible while the diff runs.
    fn start_async_rerun(&mut self) -> Task<Message> {
        let before = std::path::PathBuf::from(&self.before_path);
        let after  = std::path::PathBuf::from(&self.after_path);
        let ignore = self.active_ignore.clone();

        if !before.is_dir() || !after.is_dir() {
            return Task::none();
        }

        self.is_loading = true;
        self.load_progress = Some(t!("progress.rerunning").to_string());

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    DiffEngine::compare_with_ignore(&before, &after, &ignore)
                        .map_err(|e| e.to_string())
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r)
            },
            Message::RerunDiffReady,
        )
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

    /// Push a toast with a two-line body following RFC 020's
    /// message + hint pattern (RFC 026 §3). The first line names what
    /// happened in user-facing terms; the second line — prefixed with
    /// a 💡 marker for visual distinction — names what to do next.
    ///
    /// Use this for *actionable* errors. For purely informational
    /// success / info toasts, use [`Self::push_toast`] directly.
    pub fn push_toast_with_hint(
        &mut self,
        intent: ToastIntent,
        title: &str,
        message: &str,
        hint: &str,
    ) {
        let body = format!("{message}\n\n💡 {hint}");
        self.push_toast(intent, title, &body);
    }

    /// Push a toast carrying an already-built [`crate::error::UserError`].
    /// Convenience over [`Self::push_toast_with_hint`] for call-sites
    /// that constructed the error elsewhere (e.g. via
    /// `UserError::from_i18n("error.save.failed")`).
    ///
    /// Currently no internal site uses this — the two existing save_failed
    /// sites and the inspector regex site each build the toast inline. This
    /// method is part of RFC 026's public surface for future error sites
    /// (e.g. when DiffFailed, profile delete failure, or export failure
    /// gain proper UserError plumbing).
    #[allow(dead_code)]
    pub fn push_user_error_toast(
        &mut self,
        intent: ToastIntent,
        title: &str,
        err: &crate::error::UserError,
    ) {
        self.push_toast_with_hint(intent, title, &err.message, &err.hint);
    }
}

// ── pure functions ────────────────────────────────────────────────────────

/// RFC 023 §3.1 — subscription source for drag-and-drop window events.
/// We listen for the three iced window events that bracket a drag:
/// `FileHovered` / `FilesHoveredLeft` / `FileDropped`. The handler in
/// [`App`] decides what to do with the payload (it ignores events while
/// the user is on a screen other than Opening).
fn dnd_sub() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| {
        use iced::event::Event;
        use iced::window::Event as WinEvent;
        match event {
            Event::Window(WinEvent::FileHovered(_)) => Some(Message::FileHoverEnter),
            Event::Window(WinEvent::FilesHoveredLeft) => Some(Message::FileHoverLeave),
            Event::Window(WinEvent::FileDropped(p)) => Some(Message::FileDropped(p)),
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 028 — verify the new `hint` field is populated when the
    /// construction site passes `Some(...)`, alongside the existing
    /// `field` and `message` fields.
    #[test]
    fn field_error_with_hint_holds_all_three_fields() {
        let fe = FieldError {
            field: "pattern".into(),
            message: "Pattern parse failed".into(),
            hint: Some("Test at regex101.com".into()),
        };
        assert_eq!(fe.field, "pattern");
        assert_eq!(fe.message, "Pattern parse failed");
        assert_eq!(fe.hint.as_deref(), Some("Test at regex101.com"));
    }

    /// RFC 028 — verify `hint: None` is a valid construction and
    /// behaves identically to the pre-RFC-028 `FieldError` for
    /// errors where a hint would just repeat the message
    /// (e.g. "cannot be empty" validations).
    #[test]
    fn field_error_without_hint_remains_valid() {
        let fe = FieldError {
            field: "expected_content".into(),
            message: "Expected content cannot be empty.".into(),
            hint: None,
        };
        assert_eq!(fe.field, "expected_content");
        assert!(fe.hint.is_none());
    }
}
