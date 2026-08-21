use crate::agentic::{
    agent_flow_snapshot, agentic_safety_gate, AgentFlowSnapshot, AgentRailKind, FlowKanbanCard,
    KanbanLane,
};
use crate::app::{StartupPlan, StartupStep, StartupStepKind};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use scrin::effects::{EffectKind, EffectPlayer, LoaderKind, LoaderPlayer};
use scrin::layout::{Constraint, Layout};
use scrin::overlays::modal::ModalStyle;
use scrin::overlays::toast::ToastKind;
use scrin::overlays::{Modal, Overlay, OverlayPosition, Toast, Transition};
use scrin::status_bar::{StatusBar, StatusBarPosition};
use scrin::widgets::block::{Block, BorderStyle};
use scrin::widgets::gauge::Gauge;
use scrin::widgets::list::{List, ListItem};
use scrin::widgets::sparkline::Sparkline;
use scrin::widgets::table::{self, Table};
use scrin::widgets::tabs::Tabs;
use scrin::widgets::Widget;
use scrin::{Buffer, Color, Rect, Style, Terminal, TerminalOptions};
use std::io;
use std::time::Duration;

const TAB_NAMES: [&str; 5] = ["Overview", "Plan", "Safety", "Agent Flow", "Effects"];

pub fn render_startup_dashboard(plan: &StartupPlan) -> io::Result<()> {
    let mut terminal = Terminal::init_with(TerminalOptions {
        mouse_capture: true,
        bracketed_paste: false,
        alternate_screen: true,
        hide_cursor: true,
    })?;

    let result = run_dashboard(&mut terminal, plan);
    let restore_result = terminal.restore();
    result.and(restore_result)
}

pub fn render_startup_plan_once(plan: &StartupPlan) -> io::Result<()> {
    let mut terminal = Terminal::init_with(TerminalOptions::default())?;
    let mut state = DashboardState::new(plan);
    let draw_result = terminal.draw(|frame| {
        let area = frame.area();
        render_dashboard_frame(frame.buffer(), area, plan, &mut state);
    });
    let restore_result = terminal.restore();
    draw_result.and(restore_result)
}

fn run_dashboard(terminal: &mut Terminal, plan: &StartupPlan) -> io::Result<()> {
    let mut state = DashboardState::new(plan);

    loop {
        state.tick();
        terminal.draw(|frame| {
            let area = frame.area();
            render_dashboard_frame(frame.buffer(), area, plan, &mut state);
        })?;

        if event::poll(Duration::from_millis(48))? {
            if let Event::Key(key) = event::read()? {
                if state.handle_key(key.modifiers, key.code) {
                    return Ok(());
                }
            }
        }
    }
}

struct DashboardState {
    selected_tab: usize,
    selected_effect: usize,
    tick: usize,
    banner: EffectPlayer,
    effect_showcase: EffectPlayer,
    loader: LoaderPlayer,
    help_modal: Modal,
    toasts: Vec<Toast>,
}

impl DashboardState {
    fn new(plan: &StartupPlan) -> Self {
        let banner = EffectPlayer::new(EffectKind::Matrix, "Acropolis offline node")
            .with_accent(Color::rgb(88, 166, 255))
            .with_gradient_colors(
                vec![
                    Color::rgb(88, 166, 255),
                    Color::rgb(163, 113, 247),
                    Color::rgb(63, 185, 80),
                ],
                45.0,
            )
            .with_duration(96)
            .with_seed(plan.network_magic as u64);
        let effect_showcase = EffectPlayer::new(EFFECT_KINDS[0], "safe local first")
            .with_accent(Color::rgb(63, 185, 80))
            .with_duration(80)
            .with_seed(7);
        let loader =
            LoaderPlayer::new(LoaderKind::Bar).with_label("offline startup readiness".to_string());
        let mut help_modal =
            Modal::new("Acropolis dashboard", "Interactive local-only startup view")
                .with_style(ModalStyle::Dialog)
                .with_transition(Transition::Fade)
                .with_border_color(Color::rgb(163, 113, 247))
                .with_options(vec![
                    "Tab/Right: next panel".to_string(),
                    "Left: previous panel".to_string(),
                    "Agent Flow: local rails, Kanban, swarm lanes".to_string(),
                    "m or ?: toggle this overlay".to_string(),
                    "e: cycle Aisling effect".to_string(),
                    "t: show toast overlay".to_string(),
                    "q or Ctrl-C: quit".to_string(),
                ]);
        help_modal.show();
        let toasts = vec![Toast::new(
            "tabs, overlays, and Aisling effects are active",
            ToastKind::Success,
        )
        .with_position(OverlayPosition::BottomRight)
        .with_transition(Transition::SlideUp)
        .with_lifetime(Duration::from_secs(5))];
        Self {
            selected_tab: 0,
            selected_effect: 0,
            tick: 0,
            banner,
            effect_showcase,
            loader,
            help_modal,
            toasts,
        }
    }

    fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.banner.advance();
        self.effect_showcase.advance();
        self.help_modal.update(Duration::from_millis(48));
        for toast in &mut self.toasts {
            toast.update(Duration::from_millis(48));
        }
        self.toasts.retain(|toast| !toast.is_expired());
    }

    fn handle_key(&mut self, modifiers: KeyModifiers, code: KeyCode) -> bool {
        match (modifiers, code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c'))
            | (KeyModifiers::CONTROL, KeyCode::Char('q'))
            | (_, KeyCode::Char('q')) => true,
            (_, KeyCode::Tab) | (_, KeyCode::Right) | (_, KeyCode::Char('l')) => {
                self.selected_tab = (self.selected_tab + 1) % TAB_NAMES.len();
                self.push_toast(format!("selected {} tab", TAB_NAMES[self.selected_tab]));
                false
            }
            (_, KeyCode::BackTab) | (_, KeyCode::Left) | (_, KeyCode::Char('h')) => {
                self.selected_tab = if self.selected_tab == 0 {
                    TAB_NAMES.len() - 1
                } else {
                    self.selected_tab - 1
                };
                self.push_toast(format!("selected {} tab", TAB_NAMES[self.selected_tab]));
                false
            }
            (_, KeyCode::Char('?')) | (_, KeyCode::Char('m')) => {
                self.help_modal.toggle();
                false
            }
            (_, KeyCode::Char('e')) => {
                self.selected_effect = (self.selected_effect + 1) % EFFECT_KINDS.len();
                self.effect_showcase = EffectPlayer::new(
                    EFFECT_KINDS[self.selected_effect],
                    EFFECT_LABELS[self.selected_effect],
                )
                .with_accent(effect_accent(self.selected_effect))
                .with_duration(80)
                .with_seed(self.selected_effect as u64 + 7);
                self.push_toast(format!("Aisling effect: {}", self.effect_showcase.name()));
                false
            }
            (_, KeyCode::Char('t')) => {
                self.push_toast("offline overlay check passed".to_string());
                false
            }
            (_, KeyCode::Esc) => {
                self.help_modal.hide();
                false
            }
            _ => false,
        }
    }

    fn push_toast(&mut self, message: String) {
        self.toasts.push(
            Toast::new(&message, ToastKind::Info)
                .with_position(OverlayPosition::BottomRight)
                .with_transition(Transition::SlideUp)
                .with_lifetime(Duration::from_secs(3)),
        );
    }
}

const EFFECT_KINDS: [EffectKind; 5] = [
    EffectKind::Matrix,
    EffectKind::Decrypt,
    EffectKind::SynthGrid,
    EffectKind::Thunderstorm,
    EffectKind::Fireworks,
];

const EFFECT_LABELS: [&str; 5] = [
    "safe local first",
    "decoding startup plan",
    "sync paths disabled",
    "network listeners closed",
    "ready for local tests",
];

fn render_dashboard_frame(
    buffer: &mut Buffer,
    area: Rect,
    plan: &StartupPlan,
    state: &mut DashboardState,
) {
    let bg = Color::rgb(6, 10, 16);
    buffer.fill(area, ' ', Color::rgb(201, 209, 217), Some(bg));
    if area.width < 36 || area.height < 12 {
        buffer.set_str(
            area.x as usize,
            area.y as usize,
            "Acropolis dashboard needs a larger terminal",
            Color::rgb(248, 81, 73),
            Some(bg),
        );
        return;
    }

    let status_area = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
    let main_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
    let outer = Block::new("Acropolis")
        .with_title_right("Tab/Arrows switch panels | ? help | e effects | q quit")
        .with_borders(BorderStyle::Double)
        .with_border_color(Color::rgb(88, 166, 255))
        .with_bg(Color::rgb(13, 17, 23))
        .with_inner_margin(Rect::new(1, 0, 1, 0));
    outer.render(buffer, main_area);

    let inner = outer.inner(main_area);
    let rows = Layout::vertical(vec![
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    render_banner(buffer, rows[0], state);
    let tabs = Tabs::new(&TAB_NAMES)
        .with_selected(state.selected_tab)
        .with_style(Style::new().fg(Color::rgb(139, 148, 158)))
        .with_highlight_style(Style::new().fg(Color::rgb(255, 178, 72)).bold());
    tabs.render(buffer, rows[1]);
    render_separator(buffer, rows[2], Color::rgb(48, 54, 61));

    match state.selected_tab {
        0 => render_overview_tab(buffer, rows[3], plan, state),
        1 => render_plan_tab(buffer, rows[3], plan),
        2 => render_safety_tab(buffer, rows[3], plan),
        3 => render_agent_flow_tab(buffer, rows[3], plan, state),
        4 => render_effects_tab(buffer, rows[3], state),
        _ => {}
    }

    render_status_bar(buffer, status_area, plan, state);
    for toast in &state.toasts {
        toast.render(buffer, area);
    }
    state.help_modal.render(buffer, area);
}

fn render_banner(buffer: &mut Buffer, area: Rect, state: &mut DashboardState) {
    if area.height == 0 {
        return;
    }
    let columns =
        Layout::horizontal(vec![Constraint::Percentage(64), Constraint::Fill(1)]).split(area);
    state.banner.render_to_buffer(buffer, columns[0]);
    let loader_area = Rect::new(
        columns[1].x,
        columns[1].y.saturating_add(1),
        columns[1].width,
        columns[1].height.saturating_sub(1),
    );
    let progress = LoaderPlayer::progress_from_fraction(((state.tick % 100) as f32) / 100.0);
    state
        .loader
        .render(state.tick, progress, buffer, loader_area);
}

fn render_overview_tab(
    buffer: &mut Buffer,
    area: Rect,
    plan: &StartupPlan,
    state: &mut DashboardState,
) {
    let columns =
        Layout::horizontal(vec![Constraint::Percentage(46), Constraint::Fill(1)]).split(area);
    let left = Block::new("Readiness")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(63, 185, 80))
        .with_bg(Color::rgb(13, 17, 23));
    left.render(buffer, columns[0]);
    let left_inner = left.inner(columns[0]);
    let readiness_rows = Layout::vertical(vec![
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(left_inner);
    Gauge::new()
        .with_ratio(enabled_ratio(plan))
        .with_label("local steps")
        .with_gauge_style(Style::new().fg(Color::rgb(63, 185, 80)))
        .render(buffer, readiness_rows[0]);
    Gauge::new()
        .with_ratio(closed_network_ratio(plan))
        .with_label("network closed")
        .with_gauge_style(Style::new().fg(Color::rgb(88, 166, 255)))
        .render(buffer, readiness_rows[1]);
    Gauge::new()
        .with_ratio(if plan.network_magic > 0 { 1.0 } else { 0.0 })
        .with_label("network magic")
        .with_gauge_style(Style::new().fg(Color::rgb(255, 178, 72)))
        .render(buffer, readiness_rows[2]);
    render_separator(buffer, readiness_rows[3], Color::rgb(48, 54, 61));
    render_lines(
        buffer,
        readiness_rows[4],
        &[
            format!("network: {}", plan.network_name),
            format!("magic: {}", plan.network_magic),
            format!("steps: {}", plan.steps.len()),
            format!("closed path steps: {}", closed_steps(plan)),
            "default run remains offline".to_string(),
        ],
        Color::rgb(201, 209, 217),
    );

    let right = Block::new("Activity")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(163, 113, 247))
        .with_bg(Color::rgb(13, 17, 23));
    right.render(buffer, columns[1]);
    let right_inner = right.inner(columns[1]);
    let right_rows = Layout::vertical(vec![
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(0),
    ])
    .split(right_inner);
    let sparkline = Sparkline::new()
        .with_data(activity_points(state.tick))
        .with_max(16)
        .with_color(Color::rgb(163, 113, 247));
    sparkline.render(buffer, right_rows[0]);
    render_lines(
        buffer,
        right_rows[1],
        &[
            "animated with scrin + Aisling".to_string(),
            "toast and modal overlays enabled".to_string(),
            "tabs are live; no sockets are opened".to_string(),
        ],
        Color::rgb(139, 148, 158),
    );
    render_recent_steps(buffer, right_rows[2], plan);
}

fn render_plan_tab(buffer: &mut Buffer, area: Rect, plan: &StartupPlan) {
    let block = Block::new("Startup Plan")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(88, 166, 255))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let inner = block.inner(area);
    if inner.width < 18 {
        return;
    }
    let header = table::Row::new(vec![
        table::Cell::new("step"),
        table::Cell::new("kind"),
        table::Cell::new("state"),
        table::Cell::new("detail"),
    ]);
    let rows = plan
        .steps
        .iter()
        .map(|step| {
            table::Row::new(vec![
                table::Cell::new(&step.name).with_style(Style::new().fg(Color::rgb(201, 209, 217))),
                table::Cell::new(kind_label(&step.kind))
                    .with_style(Style::new().fg(Color::rgb(139, 148, 158))),
                table::Cell::new(if step.enabled { "open" } else { "closed" }).with_style(
                    Style::new().fg(if step.enabled {
                        Color::rgb(63, 185, 80)
                    } else {
                        Color::rgb(248, 81, 73)
                    }),
                ),
                table::Cell::new(&step.detail)
                    .with_style(Style::new().fg(Color::rgb(139, 148, 158))),
            ])
        })
        .collect::<Vec<_>>();
    let widths = plan_table_widths(inner.width);
    Table::new(&rows, &widths)
        .with_header(&header)
        .render(buffer, inner);
}

fn render_safety_tab(buffer: &mut Buffer, area: Rect, plan: &StartupPlan) {
    let columns =
        Layout::horizontal(vec![Constraint::Percentage(45), Constraint::Fill(1)]).split(area);
    let safeguards = Block::new("Safeguards")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(63, 185, 80))
        .with_bg(Color::rgb(13, 17, 23));
    safeguards.render(buffer, columns[0]);
    let items = vec![
        ListItem::new("no protocol sockets in the default plan"),
        ListItem::new("no peer dials or remote fetches"),
        ListItem::new("no on-disk mutation without opt-in"),
        ListItem::new("network listeners remain planned only"),
        ListItem::new("tests stay local/offline"),
    ];
    List::new(&items)
        .with_selected(0)
        .with_highlight_style(
            Style::new()
                .fg(Color::rgb(13, 17, 23))
                .bg(Color::rgb(63, 185, 80)),
        )
        .render(buffer, safeguards.inner(columns[0]));

    let disabled = Block::new("Closed Surfaces")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(248, 81, 73))
        .with_bg(Color::rgb(13, 17, 23));
    disabled.render(buffer, columns[1]);
    let mut lines = plan
        .steps
        .iter()
        .filter(|step| !step.enabled)
        .map(|step| format!("{}: {}", step.name, step.detail))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push("no closed surfaces recorded".to_string());
    }
    render_lines(
        buffer,
        disabled.inner(columns[1]),
        &lines,
        Color::rgb(201, 209, 217),
    );
}

fn render_agent_flow_tab(
    buffer: &mut Buffer,
    area: Rect,
    plan: &StartupPlan,
    state: &DashboardState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let snapshot = agent_flow_snapshot(plan);
    if area.width < 54 || area.height < 10 {
        render_agent_flow_compact(buffer, area, &snapshot);
        return;
    }

    if area.width >= 110 && area.height >= 16 {
        let columns = Layout::horizontal(vec![
            Constraint::Percentage(28),
            Constraint::Percentage(44),
            Constraint::Fill(1),
        ])
        .split(area);
        render_agentic_rails_panel(buffer, columns[0], &snapshot, state.tick);
        render_flow_kanban_panel(buffer, columns[1], &snapshot, state.tick);
        let right_rows = Layout::vertical(vec![Constraint::Percentage(56), Constraint::Fill(1)])
            .split(columns[2]);
        render_swarm_agents_panel(buffer, right_rows[0], &snapshot, state.tick);
        render_flow_verification_panel(buffer, right_rows[1], &snapshot, state.tick);
        return;
    }

    let rows = Layout::vertical(vec![
        Constraint::Length(area.height.min(7)),
        Constraint::Min(0),
        Constraint::Length(area.height.saturating_sub(8).min(8)),
    ])
    .split(area);
    render_agentic_rails_panel(buffer, rows[0], &snapshot, state.tick);
    render_flow_kanban_panel(buffer, rows[1], &snapshot, state.tick);
    if rows.len() > 2 && rows[2].height > 0 {
        let bottom = Layout::horizontal(vec![Constraint::Percentage(56), Constraint::Fill(1)])
            .split(rows[2]);
        render_swarm_agents_panel(buffer, bottom[0], &snapshot, state.tick);
        render_flow_verification_panel(buffer, bottom[1], &snapshot, state.tick);
    }
}

fn render_agent_flow_compact(buffer: &mut Buffer, area: Rect, snapshot: &AgentFlowSnapshot) {
    let block = Block::new("Agent Flow Compact")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(163, 113, 247))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let mut lines = vec![
        format!(
            "mode={} local_only={} live_agents_running={}",
            snapshot.active_mode.label(),
            snapshot.local_only,
            snapshot.live_agents_running
        ),
        format!(
            "kanban todo={} doing={} done={}",
            snapshot.kanban_count(KanbanLane::Todo),
            snapshot.kanban_count(KanbanLane::Doing),
            snapshot.kanban_count(KanbanLane::Done)
        ),
        format!("subagent lanes={}", snapshot.swarm_agents.len()),
        format!(
            "loop diagnostics total={} safety={}",
            snapshot.loop_diagnostics.loops,
            if snapshot.loop_diagnostics.safety_clear() {
                "clear"
            } else {
                "blocked"
            }
        ),
        format!(
            "loop replay messages={} safety={}",
            snapshot.loop_replay.messages,
            if snapshot.loop_replay.safety_clear() {
                "clear"
            } else {
                "blocked"
            }
        ),
        compact_safety_gate_line(snapshot),
    ];
    lines.extend(
        snapshot
            .rails
            .iter()
            .map(|rail| format!("{} [{}]", rail.name, rail.status)),
    );
    render_lines(buffer, block.inner(area), &lines, Color::rgb(201, 209, 217));
}

fn render_agentic_rails_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &AgentFlowSnapshot,
    tick: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let block = Block::new("Agentic Rails / Human Rails")
        .with_title_right("glow pulse")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(flow_glow_color(tick, 0))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let mut lines = vec![
        format!(
            "local_only={} live_agents_running={}",
            snapshot.local_only, snapshot.live_agents_running
        ),
        format!(
            "active mode: {} ({})",
            snapshot.active_mode.label(),
            snapshot.active_mode.contract()
        ),
    ];
    lines.extend(snapshot.modes.iter().map(|mode| {
        format!(
            "mode {:<5} {}",
            mode.label().to_ascii_lowercase(),
            mode.contract()
        )
    }));
    lines.extend(snapshot.rails.iter().map(|rail| {
        format!(
            "{} {:<13} [{}] {}",
            rail_kind_prefix(rail.kind),
            rail.name,
            rail.status,
            rail.detail
        )
    }));
    render_lines(buffer, block.inner(area), &lines, Color::rgb(201, 209, 217));
}

fn render_flow_kanban_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &AgentFlowSnapshot,
    tick: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let block = Block::new("Kanban Flow")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(flow_glow_color(tick, 1))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let inner = block.inner(area);
    if inner.width < 42 || inner.height < 6 {
        render_lines(
            buffer,
            inner,
            &compact_kanban_lines(snapshot),
            Color::rgb(201, 209, 217),
        );
        return;
    }

    let columns = Layout::horizontal(vec![
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Fill(1),
    ])
    .split(inner);
    for (index, lane) in [KanbanLane::Todo, KanbanLane::Doing, KanbanLane::Done]
        .into_iter()
        .enumerate()
    {
        render_kanban_lane(buffer, columns[index], snapshot, lane, tick + index);
    }
}

fn render_kanban_lane(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &AgentFlowSnapshot,
    lane: KanbanLane,
    tick: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let title = format!("{} {}", lane.label(), snapshot.kanban_count(lane));
    let block = Block::new(&title)
        .with_borders(BorderStyle::Rounded)
        .with_border_color(flow_glow_color(tick, lane_color_index(lane)))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    render_lines(
        buffer,
        block.inner(area),
        &kanban_lane_lines(snapshot, lane, area.height.saturating_sub(2) as usize),
        lane_color(lane),
    );
}

fn render_swarm_agents_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &AgentFlowSnapshot,
    tick: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let block = Block::new("Swarm / Subagent Lanes")
        .with_title_right("many lanes")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(flow_glow_color(tick, 2))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let mut lines = vec![
        "strategy: perfect imbalance via typed handoffs".to_string(),
        format!("lanes: {} display-only", snapshot.swarm_agents.len()),
    ];
    for agent in &snapshot.swarm_agents {
        lines.push(format!(
            "{:<10} {:<7} {:<17} w{} -> {}",
            agent.role,
            agent.status,
            format!("{}:{}", agent.kind, agent.stance),
            agent.weight,
            compact_static_list(&agent.handoff_to, 2)
        ));
        lines.push(format!("  phase: {} | {}", agent.phase.label(), agent.goal));
    }
    render_lines(buffer, block.inner(area), &lines, Color::rgb(201, 209, 217));
}

fn render_flow_verification_panel(
    buffer: &mut Buffer,
    area: Rect,
    snapshot: &AgentFlowSnapshot,
    tick: usize,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let block = Block::new("Verification Matrix")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(flow_glow_color(tick, 3))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let mut lines = safety_gate_lines(snapshot);
    lines.push(format!(
        "loop diagnostics total={} running={} blocked={} safety={}",
        snapshot.loop_diagnostics.loops,
        snapshot.loop_diagnostics.running,
        snapshot.loop_diagnostics.blocked,
        if snapshot.loop_diagnostics.safety_clear() {
            "clear"
        } else {
            "blocked"
        }
    ));
    lines.push(format!(
        "loop pressure turns={} observations={} validation_pending={}",
        snapshot.loop_diagnostics.turn_pressure_percent(),
        snapshot.loop_diagnostics.observation_pressure_percent(),
        snapshot.loop_diagnostics.validation_pending
    ));
    lines.push(format!(
        "loop replay messages={} observations={} done={} blocked={} errors={} safety={}",
        snapshot.loop_replay.messages,
        snapshot.loop_replay.observations,
        snapshot.loop_replay.done_markers,
        snapshot.loop_replay.blocked_markers,
        snapshot.loop_replay.errors,
        if snapshot.loop_replay.safety_clear() {
            "clear"
        } else {
            "blocked"
        }
    ));
    lines.extend(
        snapshot
            .rig_profiles
            .iter()
            .map(|profile| {
                format!(
                    "{:<7} {:<42} {}",
                    profile.name, profile.command, profile.detail
                )
            })
            .collect::<Vec<_>>(),
    );
    lines.push("verification results remain local".to_string());
    render_lines(buffer, block.inner(area), &lines, Color::rgb(139, 148, 158));
}

fn compact_safety_gate_line(snapshot: &AgentFlowSnapshot) -> String {
    let gate = agentic_safety_gate(snapshot);
    let actions = gate.action_items();
    format!(
        "safety gate {} blockers={} actions={}",
        if gate.is_clear() { "clear" } else { "blocked" },
        gate.blockers.len(),
        if actions.is_empty() {
            "none".to_string()
        } else {
            actions.len().to_string()
        }
    )
}

fn safety_gate_lines(snapshot: &AgentFlowSnapshot) -> Vec<String> {
    let gate = agentic_safety_gate(snapshot);
    let actions = gate.action_items();
    let mut lines = vec![format!(
        "agentic safety gate={} blockers={}",
        if gate.is_clear() { "clear" } else { "blocked" },
        gate.blockers.len()
    )];
    if gate.is_clear() {
        lines.push("agentic safety actions=none".to_string());
    } else {
        lines.extend(
            gate.blockers
                .iter()
                .take(2)
                .map(|blocker| format!("  blocker: {blocker}")),
        );
        lines.extend(
            actions
                .iter()
                .take(2)
                .map(|action| format!("  action: {action}")),
        );
    }
    lines
}

fn render_effects_tab(buffer: &mut Buffer, area: Rect, state: &mut DashboardState) {
    let columns = Layout::horizontal(vec![Constraint::Length(28), Constraint::Fill(1)]).split(area);
    let list_block = Block::new("Aisling Effects")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(255, 178, 72))
        .with_bg(Color::rgb(13, 17, 23));
    list_block.render(buffer, columns[0]);
    let items = EFFECT_KINDS
        .iter()
        .enumerate()
        .map(|(index, kind)| ListItem::new(&format!("{} - {}", index + 1, kind.name())))
        .collect::<Vec<_>>();
    List::new(&items)
        .with_selected(state.selected_effect)
        .with_highlight_style(
            Style::new()
                .fg(Color::rgb(13, 17, 23))
                .bg(effect_accent(state.selected_effect)),
        )
        .render(buffer, list_block.inner(columns[0]));

    let preview = Block::new("Effect Preview")
        .with_title_right("press e to cycle")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(effect_accent(state.selected_effect))
        .with_bg(Color::rgb(13, 17, 23));
    preview.render(buffer, columns[1]);
    let inner = preview.inner(columns[1]);
    let rows = Layout::vertical(vec![Constraint::Length(2), Constraint::Min(0)]).split(inner);
    render_lines(
        buffer,
        rows[0],
        &[format!(
            "{} frames | frame {}",
            state.effect_showcase.total_frames(),
            state.effect_showcase.current_frame_index() + 1
        )],
        Color::rgb(139, 148, 158),
    );
    state.effect_showcase.render_to_buffer(buffer, rows[1]);
}

fn render_status_bar(buffer: &mut Buffer, area: Rect, plan: &StartupPlan, state: &DashboardState) {
    let mut status = StatusBar::new()
        .with_position(StatusBarPosition::Bottom)
        .with_bg(Color::rgb(6, 10, 16));
    status.set_left(
        &format!("{} magic {}", plan.network_name, plan.network_magic),
        Color::rgb(88, 166, 255),
    );
    status.set_center(TAB_NAMES[state.selected_tab], Color::rgb(255, 178, 72));
    status.set_right(
        &format!("closed {}/{}", closed_steps(plan), plan.steps.len()),
        Color::rgb(139, 148, 158),
    );
    status.render(buffer, area);
}

fn rail_kind_prefix(kind: AgentRailKind) -> &'static str {
    match kind {
        AgentRailKind::Human => "human>",
        AgentRailKind::Agentic => "agent>",
        AgentRailKind::Verification => "verify>",
    }
}

fn compact_kanban_lines(snapshot: &AgentFlowSnapshot) -> Vec<String> {
    let mut lines = Vec::new();
    for lane in [KanbanLane::Todo, KanbanLane::Doing, KanbanLane::Done] {
        lines.push(format!(
            "{} ({})",
            lane.label(),
            snapshot.kanban_count(lane)
        ));
        lines.extend(kanban_lane_lines(snapshot, lane, 2));
    }
    lines
}

fn kanban_lane_lines(snapshot: &AgentFlowSnapshot, lane: KanbanLane, height: usize) -> Vec<String> {
    let matching = snapshot
        .kanban_cards
        .iter()
        .filter(|card| card.lane == lane)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return vec!["empty lane".to_string()];
    }
    let mut lines = Vec::new();
    let mut shown = 0usize;
    for card in matching.iter().take(height.max(1)) {
        lines.push(format!(
            "#{} {}{}",
            card.id,
            card.title,
            task_context_badge(card)
        ));
        shown += 1;
        if lines.len() < height && (!card.context_labels.is_empty() || !card.agent_notes.is_empty())
        {
            lines.push(format!(
                "  ctx {} note {}",
                compact_static_list(&card.context_labels, 2),
                compact_static_list(&card.agent_notes, 1)
            ));
        }
    }
    if matching.len() > shown {
        lines.push(format!("+{} more", matching.len() - shown));
    }
    lines
}

fn task_context_badge(card: &FlowKanbanCard) -> String {
    let context_count = card.context_labels.len();
    let note_count = card.agent_notes.len();
    if context_count == 0 && note_count == 0 {
        String::new()
    } else {
        format!(" [ctx {context_count} note {note_count}]")
    }
}

fn compact_static_list(items: &[&'static str], limit: usize) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    let mut values = items.iter().take(limit).copied().collect::<Vec<_>>();
    if items.len() > limit {
        values.push("+more");
    }
    values.join(",")
}

fn lane_color(lane: KanbanLane) -> Color {
    match lane {
        KanbanLane::Todo => Color::rgb(88, 166, 255),
        KanbanLane::Doing => Color::rgb(255, 178, 72),
        KanbanLane::Done => Color::rgb(63, 185, 80),
    }
}

fn lane_color_index(lane: KanbanLane) -> usize {
    match lane {
        KanbanLane::Todo => 1,
        KanbanLane::Doing => 4,
        KanbanLane::Done => 2,
    }
}

fn flow_glow_color(tick: usize, offset: usize) -> Color {
    match (tick / 8 + offset) % 5 {
        0 => Color::rgb(88, 166, 255),
        1 => Color::rgb(163, 113, 247),
        2 => Color::rgb(63, 185, 80),
        3 => Color::rgb(255, 178, 72),
        _ => Color::rgb(255, 0, 128),
    }
}

fn render_recent_steps(buffer: &mut Buffer, area: Rect, plan: &StartupPlan) {
    let block = Block::new("Plan Highlights")
        .with_borders(BorderStyle::Rounded)
        .with_border_color(Color::rgb(48, 54, 61))
        .with_bg(Color::rgb(13, 17, 23));
    block.render(buffer, area);
    let highlights = plan
        .steps
        .iter()
        .take(6)
        .map(step_summary)
        .collect::<Vec<_>>();
    render_lines(
        buffer,
        block.inner(area),
        &highlights,
        Color::rgb(201, 209, 217),
    );
}

fn render_separator(buffer: &mut Buffer, area: Rect, color: Color) {
    if area.height == 0 {
        return;
    }
    for x in area.x..area.right() {
        buffer.set_str(x as usize, area.y as usize, "-", color, None);
    }
}

fn render_lines(buffer: &mut Buffer, area: Rect, lines: &[String], color: Color) {
    for (row, line) in lines.iter().take(area.height as usize).enumerate() {
        let display = truncate(line, area.width as usize);
        buffer.set_str(
            area.x as usize,
            area.y as usize + row,
            &display,
            color,
            None,
        );
    }
}

fn step_summary(step: &StartupStep) -> String {
    let state = if step.enabled { "open" } else { "closed" };
    format!("{} [{}] {}", step.name, state, step.detail)
}

fn truncate(value: &str, max_width: usize) -> String {
    value.chars().take(max_width).collect()
}

fn kind_label(kind: &StartupStepKind) -> &'static str {
    match kind {
        StartupStepKind::BlockProduction => "block-production",
        StartupStepKind::Config => "config",
        StartupStepKind::LocalState => "local",
        StartupStepKind::Storage => "storage",
        StartupStepKind::Ledger => "ledger",
        StartupStepKind::Network => "network",
        StartupStepKind::Interfaces => "interfaces",
        StartupStepKind::Safety => "safety",
        StartupStepKind::Sync => "sync",
    }
}

fn enabled_ratio(plan: &StartupPlan) -> f64 {
    if plan.steps.is_empty() {
        0.0
    } else {
        plan.steps.iter().filter(|step| step.enabled).count() as f64 / plan.steps.len() as f64
    }
}

fn closed_network_ratio(plan: &StartupPlan) -> f64 {
    let network_steps = plan
        .steps
        .iter()
        .filter(|step| step.kind == StartupStepKind::Network)
        .collect::<Vec<_>>();
    if network_steps.is_empty() {
        1.0
    } else {
        network_steps.iter().filter(|step| !step.enabled).count() as f64
            / network_steps.len() as f64
    }
}

fn closed_steps(plan: &StartupPlan) -> usize {
    plan.steps.iter().filter(|step| !step.enabled).count()
}

fn plan_table_widths(width: u16) -> [u16; 4] {
    let fixed = 31_u16.min(width);
    let detail = width.saturating_sub(fixed).max(8);
    [14.min(width), 9.min(width), 8.min(width), detail]
}

fn activity_points(tick: usize) -> Vec<u64> {
    (0..40)
        .map(|index| {
            let phase = (tick + index * 3) % 16;
            4 + phase.min(16 - phase) as u64
        })
        .collect()
}

fn effect_accent(index: usize) -> Color {
    match index % EFFECT_KINDS.len() {
        0 => Color::rgb(0, 255, 65),
        1 => Color::rgb(88, 166, 255),
        2 => Color::rgb(255, 0, 128),
        3 => Color::rgb(163, 113, 247),
        _ => Color::rgb(255, 178, 72),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agentic::AgentMode;
    use crate::{Node, NodeConfig};

    fn startup_plan() -> StartupPlan {
        Node::new(NodeConfig::default()).unwrap().startup_plan()
    }

    #[test]
    fn dashboard_frame_renders_to_buffer_without_terminal() {
        let plan = startup_plan();
        let mut state = DashboardState::new(&plan);
        let mut buffer = Buffer::new(96, 28);

        render_dashboard_frame(&mut buffer, Rect::new(0, 0, 96, 28), &plan, &mut state);

        let text = buffer.to_plain_string();
        assert!(text.contains("Acropolis"));
        assert!(text.contains("local"));
        assert!(text.contains("closed"));
    }

    #[test]
    fn dashboard_frame_reports_small_terminal_without_panicking() {
        let plan = startup_plan();
        let mut state = DashboardState::new(&plan);
        let mut buffer = Buffer::new(30, 10);

        render_dashboard_frame(&mut buffer, Rect::new(0, 0, 30, 10), &plan, &mut state);

        assert!(buffer
            .to_plain_string()
            .contains("Acropolis dashboard needs"));
    }

    #[test]
    fn agent_flow_snapshot_models_acropolis_modes_and_local_rails() {
        let plan = startup_plan();
        let snapshot = agent_flow_snapshot(&plan);

        assert!(snapshot.local_only);
        assert!(!snapshot.live_agents_running);
        assert_eq!(snapshot.active_mode, AgentMode::Build);
        assert_eq!(snapshot.modes, vec![AgentMode::Build, AgentMode::Plan]);
        assert!(snapshot
            .rails
            .iter()
            .any(|rail| rail.kind == AgentRailKind::Human));
        assert!(snapshot
            .rails
            .iter()
            .any(|rail| rail.kind == AgentRailKind::Agentic));
        assert!(snapshot
            .rails
            .iter()
            .any(|rail| rail.kind == AgentRailKind::Verification));
        assert_eq!(snapshot.kanban_count(KanbanLane::Todo), 1);
        assert_eq!(snapshot.kanban_count(KanbanLane::Doing), 1);
        assert_eq!(snapshot.kanban_count(KanbanLane::Done), 1);
        assert!(snapshot.swarm_agents.len() >= 5);
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
    fn dashboard_agent_flow_tab_renders_rails_swarm_and_kanban_without_terminal() {
        let plan = startup_plan();
        let mut state = DashboardState::new(&plan);
        state.selected_tab = 3;
        state.help_modal.hide();
        let mut buffer = Buffer::new(128, 34);

        render_dashboard_frame(&mut buffer, Rect::new(0, 0, 128, 34), &plan, &mut state);

        let text = buffer.to_plain_string();
        assert!(text.contains("Agentic Rails"));
        assert!(text.contains("Human Review Rail"));
        assert!(text.contains("Kanban Flow"));
        assert!(text.contains("strategy: perfect imbalance"));
        assert!(text.contains("Scout"));
        assert!(text.contains("Verification Matrix"));
        assert!(text.contains("loop diagnostics"));
        assert!(text.contains("loop replay"));
        assert!(text.contains("agentic safety gate=clear"));
        assert!(text.contains("agentic safety actions=none"));
        assert!(text.contains("local_only=true"));
    }

    #[test]
    fn compact_agent_flow_reports_safety_actions_without_terminal() {
        let plan = startup_plan();
        let snapshot = agent_flow_snapshot(&plan);
        let mut buffer = Buffer::new(80, 10);

        render_agent_flow_compact(&mut buffer, Rect::new(0, 0, 80, 10), &snapshot);

        let text = buffer.to_plain_string();
        assert!(text.contains("safety gate clear blockers=0 actions=none"));
    }

    #[test]
    fn dashboard_state_navigation_stays_local() {
        let plan = startup_plan();
        let mut state = DashboardState::new(&plan);

        assert!(!state.handle_key(KeyModifiers::NONE, KeyCode::Right));
        assert_eq!(state.selected_tab, 1);
        assert!(!state.handle_key(KeyModifiers::NONE, KeyCode::Left));
        assert_eq!(state.selected_tab, 0);
        assert!(state.handle_key(KeyModifiers::NONE, KeyCode::Char('q')));
    }
}
