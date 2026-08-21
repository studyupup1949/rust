use crate::llm::SystemTools;
use crate::rsi::RSIEngine;
use crate::memory::store::MemoryStore;
use crate::memory::patterns::PatternEngine;
use std::sync::Arc;
use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend, Backend},
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::Alignment,
    text::Line,
};

pub struct Dashboard {
    tools: SystemTools,
    agents: Vec<String>,
    current_agent: usize,
    view_mode: ViewMode,
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    AgentList,
    AgentDetail(String),
    Analysis(String),
    Improvements(String),
}

impl Dashboard {
    pub fn new(
        rsi: Arc<RSIEngine>,
        memory: Arc<MemoryStore>,
        patterns: Arc<PatternEngine>,
        agents: Vec<String>,
    ) -> Self {
        Dashboard {
            tools: SystemTools::new(rsi, memory, patterns),
            agents,
            current_agent: 0,
            view_mode: ViewMode::AgentList,
        }
    }

    pub fn render_agent_list(&self) -> String {
        let mut output = String::new();
        output.push_str("┌─ AAS AGENT MONITOR ────────────────────────────┐\n");
        output.push_str("│ AGENTS (use ↑↓ to navigate, ENTER to inspect)   │\n");
        output.push_str("├─────────────────────────────────────────────────┤\n");

        for (idx, agent) in self.agents.iter().enumerate() {
            let marker = if idx == self.current_agent { "→ " } else { "  " };
            output.push_str(&format!("│ {} {:<40} │\n", marker, agent));
        }

        output.push_str("├─────────────────────────────────────────────────┤\n");
        output.push_str("│ [q] quit | [↑↓] navigate | [ENTER] detail      │\n");
        output.push_str("└─────────────────────────────────────────────────┘\n");
        output
    }

    pub async fn render_agent_detail(&self, agent_name: &str) -> String {
        let state = self.tools.get_self_state(agent_name).await;
        let mut output = String::new();

        output.push_str(&format!("┌─ {} ────────────────────────────┐\n", agent_name.to_uppercase()));
        output.push_str(&format!("│ Confidence Threshold: {:.2}               │\n", state.confidence_threshold));
        output.push_str(&format!("│ Polling Interval:     {}s              │\n", state.polling_interval_secs));
        output.push_str(&format!("│ Success Rate:         {:.1}%                │\n", state.success_rate * 100.0));
        output.push_str(&format!("│ Cycles (last 20):     {}                   │\n", state.cycles_last_20));
        output.push_str(&format!("│ Actions OK/FAIL:      {}/{}                │\n", state.actions_succeeded, state.actions_failed));
        output.push_str(&format!("│ Patterns Cached:      {}                   │\n", state.recent_patterns_cached));
        output.push_str("├────────────────────────────────────────────────┤\n");
        output.push_str(&format!("│ Status: {}                         │\n", state.recommendation));
        output.push_str("├────────────────────────────────────────────────┤\n");
        output.push_str("│ [a] analyze  [t] tune  [p] patterns  [q] back │\n");
        output.push_str("└────────────────────────────────────────────────┘\n");

        output
    }

    pub async fn render_analysis(&self, agent_name: &str) -> String {
        let analysis = self.tools.self_analyze(agent_name).await;
        let mut output = String::new();

        output.push_str(&format!("┌─ ANALYSIS: {} ──────────────┐\n", agent_name.to_uppercase()));
        output.push_str("├─────────────────────────────────────────────────┤\n");

        if analysis.identified_issues.is_empty() {
            output.push_str("│ ✓ No issues detected                            │\n");
        } else {
            output.push_str("│ ISSUES:                                         │\n");
            for issue in &analysis.identified_issues {
                output.push_str(&format!("│  • {}                        │\n",
                    if issue.len() > 40 {
                        format!("{}...", &issue[..37])
                    } else {
                        issue.clone()
                    }));
            }
        }

        output.push_str("├─────────────────────────────────────────────────┤\n");
        output.push_str("│ SUGGESTED IMPROVEMENTS:                         │\n");
        for (idx, imp) in analysis.proposed_improvements.iter().take(3).enumerate() {
            output.push_str(&format!("│ {}. {} │\n", idx + 1, imp.title));
            output.push_str(&format!("│    → {} │\n", imp.expected_benefit));
        }

        output.push_str("├─────────────────────────────────────────────────┤\n");
        output.push_str("│ [1-3] apply improvement  [q] back              │\n");
        output.push_str("└─────────────────────────────────────────────────┘\n");

        output
    }

    pub fn render_footer(&self) -> String {
        format!("aas@localhost> agents={} | mode={:?}\n", self.agents.len(), self.view_mode)
    }

    pub async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let text = match &self.view_mode {
                    ViewMode::AgentList => self.render_agent_list(),
                    ViewMode::AgentDetail(agent) => {
                        // ponytail: blocking call in render; add async when perf matters
                        tokio::runtime::Handle::current()
                            .block_on(self.render_agent_detail(agent))
                    }
                    ViewMode::Analysis(agent) => {
                        tokio::runtime::Handle::current()
                            .block_on(self.render_analysis(agent))
                    }
                    ViewMode::Improvements(_) => String::from("Improvements view (TBD)\n[q] back"),
                };

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("AAS Monitor");
                let para = Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Left);
                f.render_widget(para, f.size());
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => match self.view_mode {
                            ViewMode::AgentList => break,
                            ViewMode::AgentDetail(_) | ViewMode::Analysis(_) | ViewMode::Improvements(_) => {
                                self.view_mode = ViewMode::AgentList;
                            }
                        },
                        KeyCode::Up => {
                            if matches!(self.view_mode, ViewMode::AgentList) {
                                self.current_agent = self.current_agent.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if matches!(self.view_mode, ViewMode::AgentList) {
                                self.current_agent = (self.current_agent + 1).min(self.agents.len().saturating_sub(1));
                            }
                        }
                        KeyCode::Enter => {
                            if matches!(self.view_mode, ViewMode::AgentList) {
                                let agent = self.agents[self.current_agent].clone();
                                self.view_mode = ViewMode::AgentDetail(agent);
                            }
                        }
                        KeyCode::Char('a') => {
                            if let ViewMode::AgentDetail(agent) = &self.view_mode {
                                let agent = agent.clone();
                                self.view_mode = ViewMode::Analysis(agent);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}
