use crate::app::StartupPlan;

pub const MAX_AGENTIC_KANBAN_TASKS: usize = 200;
pub const DEFAULT_AGENTIC_LOOP_MAX_TURNS: u16 = 100;
pub const MAX_AGENTIC_LOOP_TURNS: u16 = 250;
pub const MAX_AGENTIC_LOOP_OBSERVATIONS: usize = 24;
pub const AGENTIC_LOOP_WARN_TURN_PERCENT: usize = 80;
pub const ACROPOLIS_AGENT_RUN_MARKER: &str = "ACROPOLIS_AGENT_RUN:";
pub const ACROPOLIS_AGENT_DONE_MARKER: &str = "ACROPOLIS_AGENT_DONE:";
pub const ACROPOLIS_AGENT_BLOCKED_MARKER: &str = "ACROPOLIS_AGENT_BLOCKED:";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentMode {
    Build,
    Plan,
}

impl AgentMode {
    pub fn id(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Plan => "plan",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Plan => "Plan",
        }
    }

    pub fn prompt_prefix(self) -> &'static str {
        match self {
            Self::Build => "build> ",
            Self::Plan => "plan> ",
        }
    }

    pub fn contract(self) -> &'static str {
        match self {
            Self::Build => "inspect, patch small, verify, repair",
            Self::Plan => "read-only inspect/search/ask only",
        }
    }

    pub fn allows_workspace_edits(self) -> bool {
        matches!(self, Self::Build)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThinkingMode {
    Off,
    Low,
    Medium,
    High,
    XHigh,
}

impl ThinkingMode {
    pub fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
        }
    }

    pub fn effort_label(self) -> &'static str {
        match self {
            Self::Off => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High | Self::XHigh => "high",
        }
    }

    pub fn from_alias(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" | "none" => Some(Self::Off),
            "low" => Some(Self::Low),
            "medium" | "med" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" | "x-high" | "max" | "ultra" => Some(Self::XHigh),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentLifecyclePhase {
    UnderstandGoal,
    InspectRepository,
    BuildContext,
    CreateEditPlan,
    ExecuteChanges,
    ValidateChanges,
    RepairFailures,
    Revalidate,
    CompleteTask,
}

impl AgentLifecyclePhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::UnderstandGoal => "Understand Goal",
            Self::InspectRepository => "Inspect Repository",
            Self::BuildContext => "Build Context",
            Self::CreateEditPlan => "Create Edit Plan",
            Self::ExecuteChanges => "Execute Changes",
            Self::ValidateChanges => "Validate Changes",
            Self::RepairFailures => "Repair Failures",
            Self::Revalidate => "Revalidate",
            Self::CompleteTask => "Complete Task",
        }
    }
}

pub const AGENT_LIFECYCLE: [AgentLifecyclePhase; 9] = [
    AgentLifecyclePhase::UnderstandGoal,
    AgentLifecyclePhase::InspectRepository,
    AgentLifecyclePhase::BuildContext,
    AgentLifecyclePhase::CreateEditPlan,
    AgentLifecyclePhase::ExecuteChanges,
    AgentLifecyclePhase::ValidateChanges,
    AgentLifecyclePhase::RepairFailures,
    AgentLifecyclePhase::Revalidate,
    AgentLifecyclePhase::CompleteTask,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentRailKind {
    Human,
    Agentic,
    Verification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KanbanLane {
    Todo,
    Doing,
    Done,
}

impl KanbanLane {
    pub fn id(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Done => "done",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "Todo",
            Self::Doing => "Doing",
            Self::Done => "Done",
        }
    }

    pub fn from_status(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "doing" | "active" | "progress" | "in-progress" | "in_progress" => Self::Doing,
            "done" | "complete" | "completed" | "finished" => Self::Done,
            _ => Self::Todo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRail {
    pub kind: AgentRailKind,
    pub name: &'static str,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowKanbanCard {
    pub id: &'static str,
    pub lane: KanbanLane,
    pub title: &'static str,
    pub context_labels: Vec<&'static str>,
    pub agent_notes: Vec<&'static str>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KanbanCardDraft {
    pub title: String,
    pub context_labels: Vec<String>,
    pub agent_notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticKanbanCard {
    pub id: String,
    pub lane: KanbanLane,
    pub title: String,
    pub context_labels: Vec<String>,
    pub agent_notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DelegatedWorkCounts {
    pub total: usize,
    pub active: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticKanbanBoard {
    pub cards: Vec<AgenticKanbanCard>,
    pub cap: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgenticKanbanError {
    EmptyTitle,
    EmptyBoardCap,
    CardNotFound(String),
}

impl std::fmt::Display for AgenticKanbanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTitle => write!(f, "kanban task title must be non-empty"),
            Self::EmptyBoardCap => write!(f, "kanban task board cap must be non-zero"),
            Self::CardNotFound(id) => write!(f, "kanban task not found: {id}"),
        }
    }
}

impl std::error::Error for AgenticKanbanError {}

impl AgenticKanbanCard {
    pub fn from_draft(id: impl Into<String>, lane: KanbanLane, draft: KanbanCardDraft) -> Self {
        Self {
            id: id.into(),
            lane,
            title: draft.title,
            context_labels: draft.context_labels,
            agent_notes: draft.agent_notes,
        }
    }
}

impl AgenticKanbanBoard {
    pub fn new() -> Self {
        Self::with_cap(MAX_AGENTIC_KANBAN_TASKS).expect("default kanban cap is non-zero")
    }

    pub fn with_cap(cap: usize) -> Result<Self, AgenticKanbanError> {
        if cap == 0 {
            return Err(AgenticKanbanError::EmptyBoardCap);
        }
        Ok(Self {
            cards: Vec::new(),
            cap,
        })
    }

    pub fn from_cards(
        cards: Vec<AgenticKanbanCard>,
        cap: usize,
    ) -> Result<Self, AgenticKanbanError> {
        if cap == 0 {
            return Err(AgenticKanbanError::EmptyBoardCap);
        }
        let mut board = Self { cards, cap };
        board.trim_to_cap();
        Ok(board)
    }

    pub fn add_spec(&mut self, spec: &str, lane: KanbanLane) -> Result<String, AgenticKanbanError> {
        let draft = parse_kanban_card_spec(spec)?;
        self.add_draft(draft, lane)
    }

    pub fn add_draft(
        &mut self,
        draft: KanbanCardDraft,
        lane: KanbanLane,
    ) -> Result<String, AgenticKanbanError> {
        if draft.title.trim().is_empty() {
            return Err(AgenticKanbanError::EmptyTitle);
        }
        let id = self.next_card_id();
        self.cards
            .push(AgenticKanbanCard::from_draft(id.clone(), lane, draft));
        self.trim_to_cap();
        Ok(id)
    }

    pub fn move_card(&mut self, id: &str, lane: KanbanLane) -> Result<(), AgenticKanbanError> {
        let card = self
            .cards
            .iter_mut()
            .find(|card| card.id == id)
            .ok_or_else(|| AgenticKanbanError::CardNotFound(id.to_string()))?;
        card.lane = lane;
        Ok(())
    }

    pub fn remove_card(&mut self, id: &str) -> Result<AgenticKanbanCard, AgenticKanbanError> {
        let index = self
            .cards
            .iter()
            .position(|card| card.id == id)
            .ok_or_else(|| AgenticKanbanError::CardNotFound(id.to_string()))?;
        Ok(self.cards.remove(index))
    }

    pub fn lane_count(&self, lane: KanbanLane) -> usize {
        self.cards.iter().filter(|card| card.lane == lane).count()
    }

    pub fn delegated_work_counts(&self) -> DelegatedWorkCounts {
        let mut counts = DelegatedWorkCounts::default();
        for card in self
            .cards
            .iter()
            .filter(|card| is_delegated_kanban_card(card))
        {
            counts.total += 1;
            if card.lane != KanbanLane::Done {
                counts.active += 1;
            }
        }
        counts
    }

    pub fn render_text(&self) -> String {
        let delegated = self.delegated_work_counts();
        let mut lines = vec![
            "Kanban tasks".to_string(),
            "Add with title | ctx: <ref,path> | note: <agent instruction>. Move with todo/doing/done lanes.".to_string(),
            format!("cap={} cards={}", self.cap, self.cards.len()),
        ];
        if delegated.total > 0 {
            lines.push(format!(
                "Delegated work: {} active / {} total",
                delegated.active, delegated.total
            ));
        }
        for lane in [KanbanLane::Todo, KanbanLane::Doing, KanbanLane::Done] {
            lines.push(String::new());
            lines.push(format!("{} ({})", lane.label(), self.lane_count(lane)));
            let matching = self
                .cards
                .iter()
                .filter(|card| card.lane == lane)
                .collect::<Vec<_>>();
            if matching.is_empty() {
                lines.push("- none".to_string());
            } else {
                for card in matching {
                    lines.push(format!("- #{} {}", card.id, card.title));
                    if !card.context_labels.is_empty() {
                        lines.push(format!("  context: {}", card.context_labels.join(", ")));
                    }
                    if !card.agent_notes.is_empty() {
                        lines.push(format!("  notes: {}", card.agent_notes.join(" | ")));
                    }
                }
            }
        }
        lines.join("\n")
    }

    fn next_card_id(&self) -> String {
        let next = self
            .cards
            .iter()
            .filter_map(|card| card.id.strip_prefix('K'))
            .filter_map(|id| id.parse::<usize>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        format!("K{next}")
    }

    fn trim_to_cap(&mut self) {
        if self.cards.len() > self.cap {
            let remove_count = self.cards.len() - self.cap;
            self.cards.drain(0..remove_count);
        }
    }
}

impl Default for AgenticKanbanBoard {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_kanban_card_spec(spec: &str) -> Result<KanbanCardDraft, AgenticKanbanError> {
    let mut parts = spec
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let title = parts.next().unwrap_or_default().trim();
    if title.is_empty() {
        return Err(AgenticKanbanError::EmptyTitle);
    }
    let mut draft = KanbanCardDraft {
        title: title.to_string(),
        context_labels: Vec::new(),
        agent_notes: Vec::new(),
    };
    for part in parts {
        if let Some(value) = tagged_value(part, &["ctx", "use"]) {
            push_unique_all(&mut draft.context_labels, split_card_list(value));
        } else if let Some(value) = tagged_value(part, &["note", "agent"]) {
            push_unique_all(&mut draft.agent_notes, split_card_list(value));
        } else {
            push_unique(&mut draft.agent_notes, part.to_string());
        }
    }
    Ok(draft)
}

pub fn is_delegated_kanban_card(card: &AgenticKanbanCard) -> bool {
    card.title
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("delegated:")
        || card.agent_notes.iter().any(|note| {
            let note = note.to_ascii_lowercase();
            note.contains("delegate_task") || note.contains("delegated work")
        })
}

fn tagged_value<'a>(part: &'a str, tags: &[&str]) -> Option<&'a str> {
    let (tag, value) = part.split_once(':')?;
    let tag = tag.trim().to_ascii_lowercase();
    if tags.iter().any(|candidate| *candidate == tag) {
        Some(value.trim())
    } else {
        None
    }
}

fn split_card_list(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn push_unique_all(values: &mut Vec<String>, incoming: Vec<String>) {
    for value in incoming {
        push_unique(values, value);
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    let value = value.trim();
    if value.is_empty() || values.iter().any(|existing| existing == value) {
        return;
    }
    values.push(value.to_string());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgenticLoopTurnLimit {
    pub requested: u32,
    pub applied: u16,
    pub capped: bool,
}

impl AgenticLoopTurnLimit {
    pub fn from_requested(requested: u32) -> Result<Self, AgenticLoopError> {
        if requested == 0 {
            return Err(AgenticLoopError::ZeroTurnLimit);
        }
        let max = u32::from(MAX_AGENTIC_LOOP_TURNS);
        Ok(Self {
            requested,
            applied: requested.min(max) as u16,
            capped: requested > max,
        })
    }

    pub fn default_limit() -> Self {
        Self {
            requested: u32::from(DEFAULT_AGENTIC_LOOP_MAX_TURNS),
            applied: DEFAULT_AGENTIC_LOOP_MAX_TURNS,
            capped: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgenticLoopStatus {
    Running,
    Done,
    Blocked,
    Stopped,
}

impl AgenticLoopStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticLoopState {
    pub id: String,
    pub goal: String,
    pub status: AgenticLoopStatus,
    pub phase: AgentLifecyclePhase,
    pub turn: u16,
    pub max_turns: u16,
    pub min_turns: u16,
    pub completion_marker: &'static str,
    pub done_reason: Option<String>,
    pub blocked_reason: Option<String>,
    pub observations: Vec<String>,
    pub validation_required: bool,
    pub validation_passed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgenticLoopError {
    EmptyGoal,
    ZeroTurnLimit,
    LoopNotRunning,
    TurnLimitReached { turn: u16, max_turns: u16 },
    ValidationRequired,
}

impl std::fmt::Display for AgenticLoopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyGoal => write!(f, "agentic loop goal must be non-empty"),
            Self::ZeroTurnLimit => write!(f, "agentic loop max turns must be non-zero"),
            Self::LoopNotRunning => write!(f, "agentic loop is not running"),
            Self::TurnLimitReached { turn, max_turns } => write!(
                f,
                "agentic loop turn limit reached: turn {turn} of {max_turns}"
            ),
            Self::ValidationRequired => write!(f, "agentic loop completion requires validation"),
        }
    }
}

impl std::error::Error for AgenticLoopError {}

impl AgenticLoopState {
    pub fn new(
        id: impl Into<String>,
        goal: impl Into<String>,
        limit: AgenticLoopTurnLimit,
    ) -> Result<Self, AgenticLoopError> {
        let goal = goal.into();
        if goal.trim().is_empty() {
            return Err(AgenticLoopError::EmptyGoal);
        }
        Ok(Self {
            id: id.into(),
            goal: goal.trim().to_string(),
            status: AgenticLoopStatus::Running,
            phase: AgentLifecyclePhase::UnderstandGoal,
            turn: 0,
            max_turns: limit.applied,
            min_turns: 1,
            completion_marker: ACROPOLIS_AGENT_DONE_MARKER,
            done_reason: None,
            blocked_reason: None,
            observations: Vec::new(),
            validation_required: false,
            validation_passed: false,
        })
    }

    pub fn with_default_limit(
        id: impl Into<String>,
        goal: impl Into<String>,
    ) -> Result<Self, AgenticLoopError> {
        Self::new(id, goal, AgenticLoopTurnLimit::default_limit())
    }

    pub fn record_turn(
        &mut self,
        phase: AgentLifecyclePhase,
        observation: impl Into<String>,
    ) -> Result<(), AgenticLoopError> {
        self.assert_running()?;
        if self.turn >= self.max_turns {
            self.status = AgenticLoopStatus::Blocked;
            self.blocked_reason = Some("turn limit reached".to_string());
            return Err(AgenticLoopError::TurnLimitReached {
                turn: self.turn,
                max_turns: self.max_turns,
            });
        }
        self.turn += 1;
        self.phase = phase;
        self.push_observation(observation.into());
        Ok(())
    }

    pub fn require_validation(&mut self) {
        self.validation_required = true;
        self.validation_passed = false;
    }

    pub fn record_validation_result(&mut self, passed: bool) {
        self.validation_required = true;
        self.validation_passed = passed;
        if passed {
            self.phase = AgentLifecyclePhase::Revalidate;
        } else {
            self.phase = AgentLifecyclePhase::RepairFailures;
        }
    }

    pub fn can_accept_done(&self) -> bool {
        !self.validation_required || self.validation_passed
    }

    pub fn mark_done(&mut self, reason: impl Into<String>) -> Result<(), AgenticLoopError> {
        self.assert_running()?;
        if !self.can_accept_done() {
            return Err(AgenticLoopError::ValidationRequired);
        }
        self.status = AgenticLoopStatus::Done;
        self.phase = AgentLifecyclePhase::CompleteTask;
        self.done_reason = Some(reason.into());
        Ok(())
    }

    pub fn mark_blocked(&mut self, reason: impl Into<String>) -> Result<(), AgenticLoopError> {
        self.assert_running()?;
        self.status = AgenticLoopStatus::Blocked;
        self.blocked_reason = Some(reason.into());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), AgenticLoopError> {
        self.assert_running()?;
        self.status = AgenticLoopStatus::Stopped;
        Ok(())
    }

    pub fn render_status(&self) -> String {
        let mut lines = vec![
            format!("Loop {}", self.id),
            format!("status: {}", self.status.label()),
            format!("goal: {}", self.goal),
            format!("phase: {}", self.phase.label()),
            format!(
                "turns: {}/{} min={}",
                self.turn, self.max_turns, self.min_turns
            ),
            format!(
                "validation_required={} validation_passed={}",
                self.validation_required, self.validation_passed
            ),
        ];
        if let Some(reason) = &self.done_reason {
            lines.push(format!("done: {reason}"));
        }
        if let Some(reason) = &self.blocked_reason {
            lines.push(format!("blocked: {reason}"));
        }
        if !self.observations.is_empty() {
            lines.push("observations".to_string());
            lines.extend(
                self.observations
                    .iter()
                    .rev()
                    .take(6)
                    .map(|observation| format!("- {observation}")),
            );
        }
        lines.join("\n")
    }

    fn assert_running(&self) -> Result<(), AgenticLoopError> {
        if self.status == AgenticLoopStatus::Running {
            Ok(())
        } else {
            Err(AgenticLoopError::LoopNotRunning)
        }
    }

    fn push_observation(&mut self, observation: String) {
        let observation = observation.trim();
        if observation.is_empty() {
            return;
        }
        self.observations.push(observation.to_string());
        if self.observations.len() > MAX_AGENTIC_LOOP_OBSERVATIONS {
            let remove_count = self.observations.len() - MAX_AGENTIC_LOOP_OBSERVATIONS;
            self.observations.drain(0..remove_count);
        }
    }
}

pub fn loop_done_reason(message: &str) -> Option<String> {
    marker_reason(message, ACROPOLIS_AGENT_DONE_MARKER)
}

pub fn loop_blocked_reason(message: &str) -> Option<String> {
    marker_reason(message, ACROPOLIS_AGENT_BLOCKED_MARKER)
}

pub fn loop_run_observation(message: &str) -> Option<String> {
    let trimmed = message.trim_start();
    let observation = trimmed.strip_prefix(ACROPOLIS_AGENT_RUN_MARKER)?.trim();
    if observation.is_empty() {
        None
    } else {
        Some(
            observation
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string(),
        )
    }
}

fn marker_reason(message: &str, marker: &str) -> Option<String> {
    let trimmed = message.trim_start();
    let reason = trimmed.strip_prefix(marker)?.trim();
    if reason.is_empty() {
        Some("no reason provided".to_string())
    } else {
        Some(reason.lines().next().unwrap_or_default().trim().to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticLoopDiagnostics {
    pub loops: usize,
    pub running: usize,
    pub done: usize,
    pub blocked: usize,
    pub stopped: usize,
    pub turns_used: usize,
    pub turn_capacity: usize,
    pub observations_retained: usize,
    pub observation_capacity: usize,
    pub validation_pending: usize,
    pub validation_failed: usize,
    pub near_turn_cap: usize,
    pub blocked_at_turn_cap: usize,
    pub local_only: bool,
    pub sidecar_spawn: bool,
    pub provider_calls: bool,
    pub filesystem_writes: bool,
    pub sockets_opened: bool,
    pub background_agents: bool,
}

impl AgenticLoopDiagnostics {
    pub fn from_loops(loops: &[AgenticLoopState]) -> Self {
        let mut diagnostics = Self {
            loops: loops.len(),
            running: 0,
            done: 0,
            blocked: 0,
            stopped: 0,
            turns_used: 0,
            turn_capacity: 0,
            observations_retained: 0,
            observation_capacity: loops.len() * MAX_AGENTIC_LOOP_OBSERVATIONS,
            validation_pending: 0,
            validation_failed: 0,
            near_turn_cap: 0,
            blocked_at_turn_cap: 0,
            local_only: true,
            sidecar_spawn: false,
            provider_calls: false,
            filesystem_writes: false,
            sockets_opened: false,
            background_agents: false,
        };

        for loop_state in loops {
            match loop_state.status {
                AgenticLoopStatus::Running => diagnostics.running += 1,
                AgenticLoopStatus::Done => diagnostics.done += 1,
                AgenticLoopStatus::Blocked => diagnostics.blocked += 1,
                AgenticLoopStatus::Stopped => diagnostics.stopped += 1,
            }

            diagnostics.turns_used += usize::from(loop_state.turn);
            diagnostics.turn_capacity += usize::from(loop_state.max_turns);
            diagnostics.observations_retained += loop_state.observations.len();

            if loop_state.validation_required && !loop_state.validation_passed {
                diagnostics.validation_pending += 1;
                if loop_state.phase == AgentLifecyclePhase::RepairFailures {
                    diagnostics.validation_failed += 1;
                }
            }

            let turn_pressure = percent(
                usize::from(loop_state.turn),
                usize::from(loop_state.max_turns),
            );
            if loop_state.status == AgenticLoopStatus::Running
                && turn_pressure >= AGENTIC_LOOP_WARN_TURN_PERCENT
            {
                diagnostics.near_turn_cap += 1;
            }
            if loop_state.status == AgenticLoopStatus::Blocked
                && (loop_state.turn >= loop_state.max_turns
                    || loop_state.blocked_reason.as_deref() == Some("turn limit reached"))
            {
                diagnostics.blocked_at_turn_cap += 1;
            }
        }

        diagnostics
    }

    pub fn turn_pressure_percent(&self) -> usize {
        percent(self.turns_used, self.turn_capacity)
    }

    pub fn observation_pressure_percent(&self) -> usize {
        percent(self.observations_retained, self.observation_capacity)
    }

    pub fn safety_clear(&self) -> bool {
        self.local_only
            && !self.sidecar_spawn
            && !self.provider_calls
            && !self.filesystem_writes
            && !self.sockets_opened
            && !self.background_agents
    }

    pub fn render_text(&self) -> String {
        let mut lines = vec![
            "Agentic loop diagnostics".to_string(),
            format!(
                "local_only={} sidecar_spawn={} provider_calls={} filesystem_writes={} sockets_opened={} background_agents={}",
                self.local_only,
                self.sidecar_spawn,
                self.provider_calls,
                self.filesystem_writes,
                self.sockets_opened,
                self.background_agents
            ),
            format!(
                "loops total={} running={} done={} blocked={} stopped={}",
                self.loops, self.running, self.done, self.blocked, self.stopped
            ),
            format!(
                "turns used={} capacity={} pressure={} near_cap={} blocked_at_cap={}",
                self.turns_used,
                self.turn_capacity,
                self.turn_pressure_percent(),
                self.near_turn_cap,
                self.blocked_at_turn_cap
            ),
            format!(
                "observations retained={} capacity={} pressure={}",
                self.observations_retained,
                self.observation_capacity,
                self.observation_pressure_percent()
            ),
            format!(
                "validation pending={} failed={}",
                self.validation_pending, self.validation_failed
            ),
            format!(
                "safety={}",
                if self.safety_clear() {
                    "clear"
                } else {
                    "blocked"
                }
            ),
        ];
        if self.near_turn_cap > 0 {
            lines.push(format!(
                "warning: {} running loop(s) at or above {}% turn budget",
                self.near_turn_cap, AGENTIC_LOOP_WARN_TURN_PERCENT
            ));
        }
        if self.blocked_at_turn_cap > 0 {
            lines.push(format!(
                "blocked: {} loop(s) reached the local turn cap",
                self.blocked_at_turn_cap
            ));
        }
        lines.join("\n")
    }
}

pub fn agentic_loop_diagnostics(loops: &[AgenticLoopState]) -> AgenticLoopDiagnostics {
    AgenticLoopDiagnostics::from_loops(loops)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticLoopReplayReport {
    pub messages: usize,
    pub observations: usize,
    pub done_markers: usize,
    pub blocked_markers: usize,
    pub ignored: usize,
    pub errors: usize,
    pub final_status: AgenticLoopStatus,
    pub final_turn: u16,
    pub local_only: bool,
    pub sidecar_spawn: bool,
    pub provider_calls: bool,
    pub filesystem_writes: bool,
    pub sockets_opened: bool,
    pub background_agents: bool,
}

impl AgenticLoopReplayReport {
    pub fn empty() -> Self {
        Self {
            messages: 0,
            observations: 0,
            done_markers: 0,
            blocked_markers: 0,
            ignored: 0,
            errors: 0,
            final_status: AgenticLoopStatus::Running,
            final_turn: 0,
            local_only: true,
            sidecar_spawn: false,
            provider_calls: false,
            filesystem_writes: false,
            sockets_opened: false,
            background_agents: false,
        }
    }

    fn new(loop_state: &AgenticLoopState) -> Self {
        let mut report = Self::empty();
        report.final_status = loop_state.status;
        report.final_turn = loop_state.turn;
        report
    }

    pub fn safety_clear(&self) -> bool {
        self.local_only
            && !self.sidecar_spawn
            && !self.provider_calls
            && !self.filesystem_writes
            && !self.sockets_opened
            && !self.background_agents
    }

    pub fn render_text(&self) -> String {
        [
            "Agentic loop replay".to_string(),
            format!(
                "local_only={} sidecar_spawn={} provider_calls={} filesystem_writes={} sockets_opened={} background_agents={}",
                self.local_only,
                self.sidecar_spawn,
                self.provider_calls,
                self.filesystem_writes,
                self.sockets_opened,
                self.background_agents
            ),
            format!(
                "messages={} observations={} done_markers={} blocked_markers={} ignored={} errors={}",
                self.messages,
                self.observations,
                self.done_markers,
                self.blocked_markers,
                self.ignored,
                self.errors
            ),
            format!(
                "final_status={} final_turn={} safety={}",
                self.final_status.label(),
                self.final_turn,
                if self.safety_clear() { "clear" } else { "blocked" }
            ),
        ]
        .join("\n")
    }
}

pub fn replay_agentic_loop_messages(
    loop_state: &mut AgenticLoopState,
    phase: AgentLifecyclePhase,
    messages: &[&str],
) -> AgenticLoopReplayReport {
    let mut report = AgenticLoopReplayReport::new(loop_state);
    for message in messages {
        report.messages += 1;
        if loop_state.status != AgenticLoopStatus::Running {
            report.ignored += 1;
            continue;
        }

        if let Some(reason) = loop_done_reason(message) {
            report.done_markers += 1;
            if loop_state.mark_done(reason).is_err() {
                report.errors += 1;
            }
            continue;
        }

        if let Some(reason) = loop_blocked_reason(message) {
            report.blocked_markers += 1;
            if loop_state.mark_blocked(reason).is_err() {
                report.errors += 1;
            }
            continue;
        }

        if let Some(observation) = loop_run_observation(message) {
            if loop_state.record_turn(phase, observation).is_ok() {
                report.observations += 1;
            } else {
                report.errors += 1;
            }
            continue;
        }

        report.ignored += 1;
    }
    report.final_status = loop_state.status;
    report.final_turn = loop_state.turn;
    report
}

fn percent(used: usize, capacity: usize) -> usize {
    used.saturating_mul(100)
        .checked_div(capacity)
        .unwrap_or(0)
        .min(100)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowSwarmAgent {
    pub role: &'static str,
    pub kind: &'static str,
    pub stance: &'static str,
    pub weight: u8,
    pub status: &'static str,
    pub phase: AgentLifecyclePhase,
    pub handoff_to: Vec<&'static str>,
    pub goal: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowRigProfile {
    pub name: &'static str,
    pub command: &'static str,
    pub detail: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentFlowSnapshot {
    pub active_mode: AgentMode,
    pub thinking_mode: ThinkingMode,
    pub modes: Vec<AgentMode>,
    pub lifecycle: Vec<AgentLifecyclePhase>,
    pub rails: Vec<AgentRail>,
    pub kanban_cards: Vec<FlowKanbanCard>,
    pub swarm_agents: Vec<FlowSwarmAgent>,
    pub rig_profiles: Vec<FlowRigProfile>,
    pub loop_diagnostics: AgenticLoopDiagnostics,
    pub loop_replay: AgenticLoopReplayReport,
    pub local_only: bool,
    pub sidecar_spawn: bool,
    pub provider_calls: bool,
    pub live_agents_running: bool,
    pub agent_runtime_state_writes: bool,
}

impl AgentFlowSnapshot {
    pub fn from_startup_plan(plan: &StartupPlan) -> Self {
        agent_flow_snapshot(plan)
    }

    pub fn kanban_count(&self, lane: KanbanLane) -> usize {
        self.kanban_cards
            .iter()
            .filter(|card| card.lane == lane)
            .count()
    }

    pub fn active_swarm_agent(&self) -> Option<&FlowSwarmAgent> {
        self.swarm_agents
            .iter()
            .find(|agent| agent.status == "active")
    }

    pub fn render_text(&self) -> String {
        let safety_gate = agentic_safety_gate(self);
        let safety_actions = safety_gate.action_items();
        let mut lines = vec![
            "Agentic flow snapshot".to_string(),
            format!(
                "mode={} thinking={} effort={}",
                self.active_mode.id(),
                self.thinking_mode.id(),
                self.thinking_mode.effort_label()
            ),
            format!(
                "local_only={} sidecar_spawn={} provider_calls={} live_agents_running={} agent_runtime_state_writes={}",
                self.local_only,
                self.sidecar_spawn,
                self.provider_calls,
                self.live_agents_running,
                self.agent_runtime_state_writes
            ),
            format!(
                "kanban todo={} doing={} done={}",
                self.kanban_count(KanbanLane::Todo),
                self.kanban_count(KanbanLane::Doing),
                self.kanban_count(KanbanLane::Done)
            ),
            format!(
                "loops total={} running={} blocked={} safety={}",
                self.loop_diagnostics.loops,
                self.loop_diagnostics.running,
                self.loop_diagnostics.blocked,
                if self.loop_diagnostics.safety_clear() {
                    "clear"
                } else {
                    "blocked"
                }
            ),
            format!(
                "replay messages={} observations={} errors={} safety={}",
                self.loop_replay.messages,
                self.loop_replay.observations,
                self.loop_replay.errors,
                if self.loop_replay.safety_clear() {
                    "clear"
                } else {
                    "blocked"
                }
            ),
            format!(
                "safety_gate={} blockers={}",
                if safety_gate.is_clear() {
                    "clear"
                } else {
                    "blocked"
                },
                safety_gate.blockers.len()
            ),
            format!(
                "safety_actions={}",
                if safety_actions.is_empty() {
                    "none".to_string()
                } else {
                    safety_actions.len().to_string()
                }
            ),
            format!("swarm_agents={}", self.swarm_agents.len()),
            "rails".to_string(),
        ];
        lines.extend(self.rails.iter().map(|rail| {
            format!(
                "- {:?} {} [{}] {}",
                rail.kind, rail.name, rail.status, rail.detail
            )
        }));
        lines.push("lifecycle".to_string());
        lines.extend(
            self.lifecycle
                .iter()
                .enumerate()
                .map(|(index, phase)| format!("{}. {}", index + 1, phase.label())),
        );
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgenticSafetyGateReport {
    pub blockers: Vec<String>,
    pub local_only: bool,
    pub sidecar_spawn: bool,
    pub provider_calls: bool,
    pub live_agents_running: bool,
    pub agent_runtime_state_writes: bool,
    pub loop_diagnostics_clear: bool,
    pub loop_replay_clear: bool,
}

impl AgenticSafetyGateReport {
    pub fn from_snapshot(snapshot: &AgentFlowSnapshot) -> Self {
        let mut blockers = Vec::new();
        if !snapshot.local_only {
            blockers.push("agent flow snapshot is not local-only".to_string());
        }
        if snapshot.sidecar_spawn {
            blockers.push("sidecar spawn is enabled".to_string());
        }
        if snapshot.provider_calls {
            blockers.push("provider calls are enabled".to_string());
        }
        if snapshot.live_agents_running {
            blockers.push("live agents are running".to_string());
        }
        if snapshot.agent_runtime_state_writes {
            blockers.push("agent runtime state writes are enabled".to_string());
        }
        push_surface_blockers(
            &mut blockers,
            "loop diagnostics",
            [
                ("is not local-only", !snapshot.loop_diagnostics.local_only),
                (
                    "sidecar spawn is enabled",
                    snapshot.loop_diagnostics.sidecar_spawn,
                ),
                (
                    "provider calls are enabled",
                    snapshot.loop_diagnostics.provider_calls,
                ),
                (
                    "filesystem writes are enabled",
                    snapshot.loop_diagnostics.filesystem_writes,
                ),
                (
                    "sockets are opened",
                    snapshot.loop_diagnostics.sockets_opened,
                ),
                (
                    "background agents are enabled",
                    snapshot.loop_diagnostics.background_agents,
                ),
            ],
        );
        push_surface_blockers(
            &mut blockers,
            "loop replay",
            [
                ("is not local-only", !snapshot.loop_replay.local_only),
                (
                    "sidecar spawn is enabled",
                    snapshot.loop_replay.sidecar_spawn,
                ),
                (
                    "provider calls are enabled",
                    snapshot.loop_replay.provider_calls,
                ),
                (
                    "filesystem writes are enabled",
                    snapshot.loop_replay.filesystem_writes,
                ),
                ("sockets are opened", snapshot.loop_replay.sockets_opened),
                (
                    "background agents are enabled",
                    snapshot.loop_replay.background_agents,
                ),
            ],
        );

        Self {
            blockers,
            local_only: snapshot.local_only,
            sidecar_spawn: snapshot.sidecar_spawn,
            provider_calls: snapshot.provider_calls,
            live_agents_running: snapshot.live_agents_running,
            agent_runtime_state_writes: snapshot.agent_runtime_state_writes,
            loop_diagnostics_clear: snapshot.loop_diagnostics.safety_clear(),
            loop_replay_clear: snapshot.loop_replay.safety_clear(),
        }
    }

    pub fn is_clear(&self) -> bool {
        self.blockers.is_empty()
    }

    pub fn action_items(&self) -> Vec<String> {
        if self.is_clear() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        if !self.local_only {
            actions.push(
                "restore local-only Agent Flow snapshots before runtime integration".to_string(),
            );
        }
        if self.sidecar_spawn {
            actions.push("disable sidecar spawning before runtime integration".to_string());
        }
        if self.provider_calls {
            actions.push("disable provider calls before runtime integration".to_string());
        }
        if self.live_agents_running {
            actions.push("stop live agents before runtime integration".to_string());
        }
        if self.agent_runtime_state_writes {
            actions
                .push("disable agent runtime state writes before runtime integration".to_string());
        }
        if !self.loop_diagnostics_clear {
            actions.push(
                "clear loop diagnostics side-effect blockers before runtime integration"
                    .to_string(),
            );
        }
        if !self.loop_replay_clear {
            actions.push(
                "clear loop replay side-effect blockers before runtime integration".to_string(),
            );
        }
        actions
    }

    pub fn render_text(&self) -> String {
        let actions = self.action_items();
        let mut lines = vec![
            "Agentic safety gate".to_string(),
            format!(
                "safety_gate={} blockers={}",
                if self.is_clear() { "clear" } else { "blocked" },
                self.blockers.len()
            ),
            format!(
                "local_only={} sidecar_spawn={} provider_calls={} live_agents_running={} agent_runtime_state_writes={}",
                self.local_only,
                self.sidecar_spawn,
                self.provider_calls,
                self.live_agents_running,
                self.agent_runtime_state_writes
            ),
            format!(
                "loop_diagnostics_clear={} loop_replay_clear={}",
                self.loop_diagnostics_clear, self.loop_replay_clear
            ),
        ];
        if self.blockers.is_empty() {
            lines.push("blockers: none".to_string());
        } else {
            lines.push("blockers".to_string());
            lines.extend(self.blockers.iter().map(|blocker| format!("- {blocker}")));
        }
        if actions.is_empty() {
            lines.push("actions: none".to_string());
        } else {
            lines.push("actions".to_string());
            lines.extend(actions.iter().map(|action| format!("- {action}")));
        }
        lines.join("\n")
    }
}

pub fn agentic_safety_gate(snapshot: &AgentFlowSnapshot) -> AgenticSafetyGateReport {
    AgenticSafetyGateReport::from_snapshot(snapshot)
}

fn push_surface_blockers(blockers: &mut Vec<String>, surface: &str, checks: [(&str, bool); 6]) {
    for (reason, blocked) in checks {
        if blocked {
            blockers.push(format!("{surface} {reason}"));
        }
    }
}

pub fn agent_flow_snapshot(plan: &StartupPlan) -> AgentFlowSnapshot {
    let closed = plan.steps.iter().filter(|step| !step.enabled).count();
    AgentFlowSnapshot {
        active_mode: AgentMode::Build,
        thinking_mode: ThinkingMode::Medium,
        modes: vec![AgentMode::Build, AgentMode::Plan],
        lifecycle: AGENT_LIFECYCLE.to_vec(),
        rails: vec![
            AgentRail {
                kind: AgentRailKind::Human,
                name: "Human Review Rail",
                status: "gated",
                detail: "approves live paths, destructive actions, visual pass, and release gates"
                    .to_string(),
            },
            AgentRail {
                kind: AgentRailKind::Agentic,
                name: "Agentic Build Rail",
                status: "local",
                detail: format!(
                    "{} startup steps modeled; implementation slices stay small and verified",
                    plan.steps.len()
                ),
            },
            AgentRail {
                kind: AgentRailKind::Verification,
                name: "Verification Rail",
                status: "offline",
                detail: format!(
                    "{} closed path surfaces remain pinned by tests and dry-run output",
                    closed
                ),
            },
        ],
        kanban_cards: vec![
            FlowKanbanCard {
                id: "K1",
                lane: KanbanLane::Todo,
                title: "Keep Acropolis network activation behind conformance gates",
                context_labels: Vec::new(),
                agent_notes: vec!["require explicit bounded testnet configuration"],
            },
            FlowKanbanCard {
                id: "K2",
                lane: KanbanLane::Doing,
                title: "Expose Acropolis local agent runtime state in the dashboard",
                context_labels: Vec::new(),
                agent_notes: vec!["display local state without starting agents"],
            },
            FlowKanbanCard {
                id: "K3",
                lane: KanbanLane::Done,
                title: "Verify Acropolis Cardano handshake decisions offline",
                context_labels: Vec::new(),
                agent_notes: vec!["keep all protocol sockets closed"],
            },
        ],
        swarm_agents: vec![
            FlowSwarmAgent {
                role: "Scout",
                kind: "explore",
                stance: "map",
                weight: 1,
                status: "ready",
                phase: AgentLifecyclePhase::InspectRepository,
                handoff_to: vec!["Strategist"],
                goal: "map runtime modes, state transitions, and safety gaps",
            },
            FlowSwarmAgent {
                role: "Strategist",
                kind: "plan",
                stance: "balance",
                weight: 2,
                status: "ready",
                phase: AgentLifecyclePhase::CreateEditPlan,
                handoff_to: vec!["Builder", "RedTeam"],
                goal: "sequence small verified implementation epochs",
            },
            FlowSwarmAgent {
                role: "Builder",
                kind: "build",
                stance: "patch",
                weight: 3,
                status: "active",
                phase: AgentLifecyclePhase::ExecuteChanges,
                handoff_to: vec!["Verifier"],
                goal: "apply minimal local-only code changes",
            },
            FlowSwarmAgent {
                role: "RedTeam",
                kind: "review",
                stance: "skeptic",
                weight: 2,
                status: "ready",
                phase: AgentLifecyclePhase::RepairFailures,
                handoff_to: vec!["Builder"],
                goal: "look for unsafe path opening or fake readiness",
            },
            FlowSwarmAgent {
                role: "Verifier",
                kind: "rig",
                stance: "evidence",
                weight: 3,
                status: "queued",
                phase: AgentLifecyclePhase::ValidateChanges,
                handoff_to: vec!["Human"],
                goal: "run offline checks and report good or bad",
            },
        ],
        rig_profiles: vec![
            FlowRigProfile {
                name: "fmt",
                command: "cargo fmt --check",
                detail: "format gate",
            },
            FlowRigProfile {
                name: "check",
                command: "cargo check --offline",
                detail: "offline compile gate",
            },
            FlowRigProfile {
                name: "test",
                command: "cargo test --offline",
                detail: "offline unit/golden gate",
            },
            FlowRigProfile {
                name: "dry-run",
                command: "cargo run --offline -- testnet-plan --network testnet",
                detail: "no-live-contact gate",
            },
            FlowRigProfile {
                name: "tui",
                command: "cargo test --offline --features tui",
                detail: "dashboard render gate",
            },
        ],
        loop_diagnostics: agentic_loop_diagnostics(&[]),
        loop_replay: AgenticLoopReplayReport::empty(),
        local_only: true,
        sidecar_spawn: false,
        provider_calls: false,
        live_agents_running: false,
        agent_runtime_state_writes: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, NodeConfig};

    fn startup_plan() -> StartupPlan {
        Node::new(NodeConfig::default()).unwrap().startup_plan()
    }

    #[test]
    fn agent_modes_follow_acropolis_runtime_contracts_without_side_effects() {
        assert_eq!(AgentMode::Build.id(), "build");
        assert_eq!(AgentMode::Build.prompt_prefix(), "build> ");
        assert!(AgentMode::Build.allows_workspace_edits());
        assert_eq!(AgentMode::Plan.id(), "plan");
        assert_eq!(AgentMode::Plan.prompt_prefix(), "plan> ");
        assert!(!AgentMode::Plan.allows_workspace_edits());
    }

    #[test]
    fn thinking_mode_aliases_support_acropolis_runtime() {
        assert_eq!(ThinkingMode::from_alias("none"), Some(ThinkingMode::Off));
        assert_eq!(ThinkingMode::from_alias("med"), Some(ThinkingMode::Medium));
        assert_eq!(ThinkingMode::from_alias("ultra"), Some(ThinkingMode::XHigh));
        assert_eq!(ThinkingMode::from_alias("bad"), None);
        assert_eq!(ThinkingMode::XHigh.effort_label(), "high");
    }

    #[test]
    fn lifecycle_keeps_disciplined_runtime_order() {
        assert_eq!(AGENT_LIFECYCLE[0], AgentLifecyclePhase::UnderstandGoal);
        assert_eq!(AGENT_LIFECYCLE[3], AgentLifecyclePhase::CreateEditPlan);
        assert_eq!(AGENT_LIFECYCLE[5], AgentLifecyclePhase::ValidateChanges);
        assert_eq!(AGENT_LIFECYCLE[8], AgentLifecyclePhase::CompleteTask);
    }

    #[test]
    fn agent_flow_snapshot_is_local_only_and_counts_lanes() {
        let snapshot = agent_flow_snapshot(&startup_plan());

        assert!(snapshot.local_only);
        assert!(!snapshot.sidecar_spawn);
        assert!(!snapshot.provider_calls);
        assert!(!snapshot.live_agents_running);
        assert!(!snapshot.agent_runtime_state_writes);
        assert_eq!(snapshot.active_mode, AgentMode::Build);
        assert_eq!(snapshot.thinking_mode, ThinkingMode::Medium);
        assert_eq!(snapshot.kanban_count(KanbanLane::Todo), 1);
        assert_eq!(snapshot.kanban_count(KanbanLane::Doing), 1);
        assert_eq!(snapshot.kanban_count(KanbanLane::Done), 1);
        assert_eq!(
            snapshot.active_swarm_agent().map(|agent| agent.role),
            Some("Builder")
        );
        assert!(snapshot
            .rig_profiles
            .iter()
            .any(|profile| profile.name == "tui"));
        assert_eq!(snapshot.loop_diagnostics.loops, 0);
        assert!(snapshot.loop_diagnostics.safety_clear());
        assert_eq!(snapshot.loop_replay.messages, 0);
        assert!(snapshot.loop_replay.safety_clear());
    }

    #[test]
    fn agent_flow_snapshot_renders_text_report() {
        let report = agent_flow_snapshot(&startup_plan()).render_text();

        assert!(report.contains("Agentic flow snapshot"));
        assert!(report.contains("mode=build"));
        assert!(report.contains("provider_calls=false"));
        assert!(report.contains("kanban todo=1 doing=1 done=1"));
        assert!(report.contains("loops total=0 running=0 blocked=0 safety=clear"));
        assert!(report.contains("replay messages=0 observations=0 errors=0 safety=clear"));
        assert!(report.contains("safety_gate=clear blockers=0"));
        assert!(report.contains("safety_actions=none"));
        assert!(report.contains("Validate Changes"));
    }

    #[test]
    fn agentic_safety_gate_allows_default_local_snapshot() {
        let snapshot = agent_flow_snapshot(&startup_plan());
        let gate = agentic_safety_gate(&snapshot);

        assert!(gate.is_clear());
        assert!(gate.blockers.is_empty());
        assert!(gate.loop_diagnostics_clear);
        assert!(gate.loop_replay_clear);
        assert!(gate.action_items().is_empty());
        let report = gate.render_text();
        assert!(report.contains("Agentic safety gate"));
        assert!(report.contains("safety_gate=clear blockers=0"));
        assert!(report.contains("blockers: none"));
        assert!(report.contains("actions: none"));
    }

    #[test]
    fn agentic_safety_gate_reports_side_effect_blockers() {
        let mut snapshot = agent_flow_snapshot(&startup_plan());
        snapshot.provider_calls = true;
        snapshot.live_agents_running = true;
        snapshot.agent_runtime_state_writes = true;
        snapshot.loop_diagnostics.sockets_opened = true;
        snapshot.loop_replay.filesystem_writes = true;

        let gate = agentic_safety_gate(&snapshot);

        assert!(!gate.is_clear());
        assert!(gate
            .blockers
            .iter()
            .any(|blocker| blocker == "provider calls are enabled"));
        assert!(gate
            .blockers
            .iter()
            .any(|blocker| blocker == "live agents are running"));
        assert!(gate
            .blockers
            .iter()
            .any(|blocker| blocker == "agent runtime state writes are enabled"));
        assert!(gate
            .blockers
            .iter()
            .any(|blocker| blocker == "loop diagnostics sockets are opened"));
        assert!(gate
            .blockers
            .iter()
            .any(|blocker| blocker == "loop replay filesystem writes are enabled"));
        let actions = gate.action_items();
        assert!(actions
            .iter()
            .any(|action| action == "disable provider calls before runtime integration"));
        assert!(actions
            .iter()
            .any(|action| action == "stop live agents before runtime integration"));
        assert!(actions.iter().any(|action| {
            action == "disable agent runtime state writes before runtime integration"
        }));
        assert!(actions.iter().any(|action| {
            action == "clear loop diagnostics side-effect blockers before runtime integration"
        }));
        assert!(actions
            .iter()
            .any(|action| action
                == "clear loop replay side-effect blockers before runtime integration"));
        let report = gate.render_text();
        assert!(report.contains("safety_gate=blocked"));
        assert!(report.contains("- provider calls are enabled"));
        assert!(report.contains("actions"));
        assert!(report.contains("- disable provider calls before runtime integration"));
    }

    #[test]
    fn kanban_lane_status_aliases_support_acropolis_tasks() {
        assert_eq!(KanbanLane::from_status("todo"), KanbanLane::Todo);
        assert_eq!(KanbanLane::from_status("active"), KanbanLane::Doing);
        assert_eq!(KanbanLane::from_status("in-progress"), KanbanLane::Doing);
        assert_eq!(KanbanLane::from_status("completed"), KanbanLane::Done);
        assert_eq!(KanbanLane::Done.id(), "done");
    }

    #[test]
    fn kanban_card_parser_extracts_context_labels_and_agent_notes() {
        let draft = parse_kanban_card_spec(
            "Review Acropolis startup | ctx: startup plan, local defaults | note: preserve local-only defaults; test parser | agent: delegate_task verifier",
        )
        .unwrap();

        assert_eq!(draft.title, "Review Acropolis startup");
        assert_eq!(draft.context_labels, vec!["startup plan", "local defaults"]);
        assert_eq!(
            draft.agent_notes,
            vec![
                "preserve local-only defaults",
                "test parser",
                "delegate_task verifier"
            ]
        );
        assert_eq!(
            parse_kanban_card_spec("  "),
            Err(AgenticKanbanError::EmptyTitle)
        );
    }

    #[test]
    fn kanban_board_adds_moves_removes_and_caps_oldest_tasks() {
        let mut board = AgenticKanbanBoard::with_cap(2).unwrap();
        let first = board
            .add_spec("Inspect Acropolis startup", KanbanLane::Todo)
            .unwrap();
        let second = board
            .add_spec(
                "Validate Acropolis config | ctx: local configuration",
                KanbanLane::Doing,
            )
            .unwrap();

        assert_eq!(first, "K1");
        assert_eq!(second, "K2");
        assert_eq!(board.lane_count(KanbanLane::Todo), 1);
        assert_eq!(board.lane_count(KanbanLane::Doing), 1);

        board.move_card(&second, KanbanLane::Done).unwrap();
        assert_eq!(board.lane_count(KanbanLane::Done), 1);
        assert_eq!(
            board.move_card("missing", KanbanLane::Done),
            Err(AgenticKanbanError::CardNotFound("missing".to_string()))
        );

        let third = board
            .add_spec("Record Acropolis result", KanbanLane::Todo)
            .unwrap();
        assert_eq!(third, "K3");
        assert_eq!(board.cards.len(), 2);
        assert!(board.cards.iter().all(|card| card.id != first));

        let removed = board.remove_card(&third).unwrap();
        assert_eq!(removed.title, "Record Acropolis result");
    }

    #[test]
    fn kanban_board_reports_delegated_work_and_renders_text() {
        let mut board = AgenticKanbanBoard::new();
        board
            .add_spec(
                "Delegated: review Acropolis dashboard | ctx: dashboard state | note: delegated work",
                KanbanLane::Doing,
            )
            .unwrap();
        let done = board
            .add_spec(
                "Delegated: document Acropolis runtime | note: delegate_task docs",
                KanbanLane::Todo,
            )
            .unwrap();
        board.move_card(&done, KanbanLane::Done).unwrap();

        let counts = board.delegated_work_counts();
        assert_eq!(counts.total, 2);
        assert_eq!(counts.active, 1);

        let report = board.render_text();
        assert!(report.contains("Kanban tasks"));
        assert!(report.contains("Delegated work: 1 active / 2 total"));
        assert!(report.contains("Doing (1)"));
        assert!(report.contains("context: dashboard state"));
        assert!(report.contains("notes: delegated work"));
    }

    #[test]
    fn loop_turn_limits_cap_requested_values() {
        assert_eq!(
            AgenticLoopTurnLimit::from_requested(0),
            Err(AgenticLoopError::ZeroTurnLimit)
        );
        let capped = AgenticLoopTurnLimit::from_requested(999).unwrap();
        assert_eq!(capped.requested, 999);
        assert_eq!(capped.applied, MAX_AGENTIC_LOOP_TURNS);
        assert!(capped.capped);
        let default = AgenticLoopTurnLimit::default_limit();
        assert_eq!(default.applied, DEFAULT_AGENTIC_LOOP_MAX_TURNS);
        assert!(!default.capped);
    }

    #[test]
    fn loop_markers_parse_done_and_blocked_reasons() {
        assert_eq!(
            loop_done_reason("ACROPOLIS_AGENT_DONE: verified locally\nextra"),
            Some("verified locally".to_string())
        );
        assert_eq!(
            loop_blocked_reason("  ACROPOLIS_AGENT_BLOCKED: needs visual pass"),
            Some("needs visual pass".to_string())
        );
        assert_eq!(loop_done_reason("not done"), None);
    }

    #[test]
    fn loop_run_observation_parses_local_run_messages() {
        assert_eq!(
            loop_run_observation("ACROPOLIS_AGENT_RUN: inspect repo\nextra"),
            Some("inspect repo".to_string())
        );
        assert_eq!(
            loop_run_observation("  ACROPOLIS_AGENT_RUN: validate offline"),
            Some("validate offline".to_string())
        );
        assert_eq!(loop_run_observation("ACROPOLIS_AGENT_RUN:   "), None);
        assert_eq!(loop_run_observation("DONE"), None);
    }

    #[test]
    fn loop_state_records_bounded_observations_and_blocks_at_turn_cap() {
        let mut loop_state = AgenticLoopState::new(
            "loop-1",
            "finish local Acropolis agent runtime logic",
            AgenticLoopTurnLimit::from_requested(2).unwrap(),
        )
        .unwrap();

        loop_state
            .record_turn(AgentLifecyclePhase::InspectRepository, "inspect repository")
            .unwrap();
        loop_state
            .record_turn(AgentLifecyclePhase::ExecuteChanges, "edit runtime state")
            .unwrap();

        assert_eq!(loop_state.turn, 2);
        assert_eq!(loop_state.observations.len(), 2);
        assert_eq!(
            loop_state.record_turn(AgentLifecyclePhase::ValidateChanges, "test changes"),
            Err(AgenticLoopError::TurnLimitReached {
                turn: 2,
                max_turns: 2,
            })
        );
        assert_eq!(loop_state.status, AgenticLoopStatus::Blocked);
        assert_eq!(
            loop_state.blocked_reason.as_deref(),
            Some("turn limit reached")
        );
    }

    #[test]
    fn loop_state_requires_validation_before_done_when_requested() {
        let mut loop_state =
            AgenticLoopState::with_default_limit("loop-2", "validate changes").unwrap();
        loop_state.require_validation();

        assert_eq!(
            loop_state.mark_done("complete"),
            Err(AgenticLoopError::ValidationRequired)
        );
        loop_state.record_validation_result(false);
        assert_eq!(loop_state.phase, AgentLifecyclePhase::RepairFailures);
        loop_state.record_validation_result(true);
        assert_eq!(loop_state.phase, AgentLifecyclePhase::Revalidate);
        loop_state.mark_done("validation passed").unwrap();

        assert_eq!(loop_state.status, AgenticLoopStatus::Done);
        assert_eq!(loop_state.phase, AgentLifecyclePhase::CompleteTask);
        assert_eq!(
            loop_state.record_turn(AgentLifecyclePhase::CompleteTask, "late"),
            Err(AgenticLoopError::LoopNotRunning)
        );
    }

    #[test]
    fn loop_state_renders_status_and_retains_newest_observations() {
        let mut loop_state = AgenticLoopState::with_default_limit("loop-3", "many turns").unwrap();
        for index in 0..(MAX_AGENTIC_LOOP_OBSERVATIONS + 2) {
            loop_state
                .record_turn(AgentLifecyclePhase::BuildContext, format!("turn {index}"))
                .unwrap();
        }

        assert_eq!(loop_state.observations.len(), MAX_AGENTIC_LOOP_OBSERVATIONS);
        assert_eq!(
            loop_state.observations.first().map(String::as_str),
            Some("turn 2")
        );
        let report = loop_state.render_status();
        assert!(report.contains("Loop loop-3"));
        assert!(report.contains("status: running"));
        assert!(report.contains("turns: 26/100"));
        assert!(report.contains("observations"));
    }

    #[test]
    fn loop_diagnostics_summarize_local_runtime_state() {
        let mut running = AgenticLoopState::new(
            "loop-running",
            "watch turn pressure",
            AgenticLoopTurnLimit::from_requested(5).unwrap(),
        )
        .unwrap();
        for index in 0..4 {
            running
                .record_turn(AgentLifecyclePhase::ExecuteChanges, format!("turn {index}"))
                .unwrap();
        }
        running.require_validation();

        let mut done = AgenticLoopState::with_default_limit("loop-done", "finish").unwrap();
        done.record_turn(AgentLifecyclePhase::ValidateChanges, "verify locally")
            .unwrap();
        done.mark_done("verified").unwrap();

        let mut blocked = AgenticLoopState::new(
            "loop-blocked",
            "hit cap",
            AgenticLoopTurnLimit::from_requested(1).unwrap(),
        )
        .unwrap();
        blocked
            .record_turn(AgentLifecyclePhase::BuildContext, "inspect once")
            .unwrap();
        let _ = blocked.record_turn(AgentLifecyclePhase::BuildContext, "inspect twice");

        let diagnostics = agentic_loop_diagnostics(&[running, done, blocked]);

        assert_eq!(diagnostics.loops, 3);
        assert_eq!(diagnostics.running, 1);
        assert_eq!(diagnostics.done, 1);
        assert_eq!(diagnostics.blocked, 1);
        assert_eq!(diagnostics.turns_used, 6);
        assert_eq!(diagnostics.turn_capacity, 106);
        assert_eq!(diagnostics.validation_pending, 1);
        assert_eq!(diagnostics.near_turn_cap, 1);
        assert_eq!(diagnostics.blocked_at_turn_cap, 1);
        assert!(diagnostics.safety_clear());
    }

    #[test]
    fn loop_diagnostics_render_text_without_runtime_side_effects() {
        let diagnostics = agentic_loop_diagnostics(&[]);
        let report = diagnostics.render_text();

        assert_eq!(diagnostics.turn_pressure_percent(), 0);
        assert_eq!(diagnostics.observation_pressure_percent(), 0);
        assert!(diagnostics.safety_clear());
        assert!(report.contains("Agentic loop diagnostics"));
        assert!(report.contains("local_only=true"));
        assert!(report.contains("provider_calls=false"));
        assert!(report.contains("sockets_opened=false"));
        assert!(report.contains("safety=clear"));
    }

    #[test]
    fn loop_replay_applies_run_messages_and_done_marker_locally() {
        let mut loop_state = AgenticLoopState::with_default_limit("loop-replay", "replay").unwrap();
        let report = replay_agentic_loop_messages(
            &mut loop_state,
            AgentLifecyclePhase::ExecuteChanges,
            &[
                "noise",
                "ACROPOLIS_AGENT_RUN: inspect repository",
                "ACROPOLIS_AGENT_RUN: edit local state",
                "ACROPOLIS_AGENT_DONE: verified offline",
                "ACROPOLIS_AGENT_RUN: after terminal marker",
            ],
        );

        assert_eq!(report.messages, 5);
        assert_eq!(report.observations, 2);
        assert_eq!(report.done_markers, 1);
        assert_eq!(report.ignored, 2);
        assert_eq!(report.errors, 0);
        assert_eq!(report.final_status, AgenticLoopStatus::Done);
        assert_eq!(report.final_turn, 2);
        assert_eq!(loop_state.done_reason.as_deref(), Some("verified offline"));
        assert!(report.safety_clear());
        assert!(report.render_text().contains("Agentic loop replay"));
    }

    #[test]
    fn empty_loop_replay_report_is_display_only_and_safe() {
        let report = AgenticLoopReplayReport::empty();

        assert_eq!(report.messages, 0);
        assert_eq!(report.final_status, AgenticLoopStatus::Running);
        assert_eq!(report.final_turn, 0);
        assert!(report.safety_clear());
        assert!(report.render_text().contains("messages=0 observations=0"));
    }

    #[test]
    fn loop_replay_preserves_validation_gate_before_done() {
        let mut loop_state =
            AgenticLoopState::with_default_limit("loop-gated", "validate").unwrap();
        loop_state.require_validation();

        let blocked_report = replay_agentic_loop_messages(
            &mut loop_state,
            AgentLifecyclePhase::ValidateChanges,
            &[
                "ACROPOLIS_AGENT_RUN: test",
                "ACROPOLIS_AGENT_DONE: premature",
            ],
        );
        assert_eq!(blocked_report.observations, 1);
        assert_eq!(blocked_report.done_markers, 1);
        assert_eq!(blocked_report.errors, 1);
        assert_eq!(blocked_report.final_status, AgenticLoopStatus::Running);

        loop_state.record_validation_result(true);
        let done_report = replay_agentic_loop_messages(
            &mut loop_state,
            AgentLifecyclePhase::CompleteTask,
            &["ACROPOLIS_AGENT_DONE: validation passed"],
        );
        assert_eq!(done_report.errors, 0);
        assert_eq!(done_report.final_status, AgenticLoopStatus::Done);
        assert_eq!(loop_state.done_reason.as_deref(), Some("validation passed"));
    }

    #[test]
    fn loop_replay_applies_blocked_marker_without_runtime_side_effects() {
        let mut loop_state =
            AgenticLoopState::with_default_limit("loop-blocked", "blocked").unwrap();
        let report = replay_agentic_loop_messages(
            &mut loop_state,
            AgentLifecyclePhase::BuildContext,
            &[
                "ACROPOLIS_AGENT_RUN: inspect blocker",
                "ACROPOLIS_AGENT_BLOCKED: needs human decision",
            ],
        );

        assert_eq!(report.observations, 1);
        assert_eq!(report.blocked_markers, 1);
        assert_eq!(report.final_status, AgenticLoopStatus::Blocked);
        assert_eq!(
            loop_state.blocked_reason.as_deref(),
            Some("needs human decision")
        );
        assert!(report.safety_clear());
    }
}
