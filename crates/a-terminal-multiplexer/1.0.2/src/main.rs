#![allow(dead_code, unused_variables)]

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction as RDirection, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{self, Write, Read},
    sync::{Arc, Mutex},
    time::Duration,
};
use copypasta::{ClipboardContext, ClipboardProvider};

// --- TYPES AND ENUMS ---

type PaneId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    PendingNew,
    PendingTab,
    RenamingTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LayoutNode {
    Empty,
    Pane(PaneId),
    Split {
        direction: Direction,
        children: Vec<LayoutNode>,
    },
}

struct Pane {
    id: PaneId,
    writer: Box<dyn std::io::Write + Send>,
    parser: Arc<Mutex<vt100::Parser>>,
}

struct Tab {
    name: String,
    layout_root: LayoutNode,
    active_pane_id: Option<PaneId>,
}

#[derive(Deserialize, Debug, Clone)]
struct AtmConfig {
    default_shell: String,
    active_border_color: String,
}

impl AtmConfig {
    fn load() -> Self {
        let mut config_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        config_path.push("atm");
        config_path.push("config.toml");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(parsed) = toml::from_str::<AtmConfig>(&content) {
                return parsed;
            }
        }
        Self {
            default_shell: "elvish".to_string(),
            active_border_color: "cyan".to_string(),
        }
    }
}

struct App {
    config: AtmConfig,
    panes: HashMap<PaneId, Pane>,
    tabs: Vec<Tab>,
    active_tab_index: usize,
    next_pane_id: PaneId,
    mode: Mode,
    should_quit: bool,
    rename_input: String,
}

impl App {
    fn new() -> Self {
        let config = AtmConfig::load();
        let mut app = Self {
            config,
            panes: HashMap::new(),
            tabs: Vec::new(),
            active_tab_index: 0,
            next_pane_id: 1,
            mode: Mode::Normal,
            should_quit: false,
            rename_input: String::new(),
        };
        app.spawn_initial_tab();
        app
    }

    fn spawn_shell_for_pane(&mut self, id: PaneId) -> (Box<dyn std::io::Write + Send>, Arc<Mutex<vt100::Parser>>) {
        let pty_pair = portable_pty::native_pty_system()
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let shell = &self.config.default_shell;
        let cmd = portable_pty::CommandBuilder::new(shell);
        let mut child = pty_pair.slave.spawn_command(cmd).unwrap();

        let mut reader = pty_pair.master.try_clone_reader().unwrap();
        let writer = pty_pair.master.take_writer().unwrap();

        let parser = Arc::new(Mutex::new(vt100::Parser::new(24, 80, 0)));
        let parser_clone = parser.clone();

        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut p = parser_clone.lock().unwrap();
                        p.process(&buf[..n]);
                    }
                }
            }
            let _ = child.kill();
        });

        let mut w = writer;
        let _ = w.write_all(b"\x1bc"); 
        let _ = w.flush();

        (w, parser)
    }

    fn spawn_initial_tab(&mut self) {
        let id = self.next_pane_id;
        self.next_pane_id += 1;
        let (writer, parser) = self.spawn_shell_for_pane(id);
        self.panes.insert(id, Pane { id, writer, parser });

        self.tabs.push(Tab {
            name: "Tab 1".to_string(),
            layout_root: LayoutNode::Pane(id),
            active_pane_id: Some(id),
        });
    }

    fn spawn_new_tab(&mut self) {
        self.tabs.push(Tab {
            name: format!("Tab {}", self.tabs.len() + 1),
            layout_root: LayoutNode::Empty,
            active_pane_id: None,
        });
        self.active_tab_index = self.tabs.len() - 1;
    }

    fn close_current_tab(&mut self) {
        if self.tabs.len() <= 1 {
            self.should_quit = true;
            return;
        }
        let active_tab = &self.tabs[self.active_tab_index];
        let mut pane_ids = Vec::new();
        fn collect_ids(node: &LayoutNode, ids: &mut Vec<PaneId>) {
            match node {
                LayoutNode::Empty => {}
                LayoutNode::Pane(id) => ids.push(*id),
                LayoutNode::Split { children, .. } => {
                    for child in children {
                        collect_ids(child, ids);
                    }
                }
            }
        }
        collect_ids(&active_tab.layout_root, &mut pane_ids);
        for id in pane_ids {
            self.panes.remove(&id);
        }
        self.tabs.remove(self.active_tab_index);
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
    }

    fn split_active_pane(&mut self, dir: Direction) {
        let new_id = self.next_pane_id;
        self.next_pane_id += 1;
        
        let (writer, parser) = self.spawn_shell_for_pane(new_id);
        self.panes.insert(new_id, Pane { id: new_id, writer, parser });

        let active_tab = &mut self.tabs[self.active_tab_index];
        
        if let Some(target_id) = active_tab.active_pane_id {
            fn replace_node(node: LayoutNode, target: PaneId, dir: Direction, new_id: PaneId) -> LayoutNode {
                match node {
                    LayoutNode::Empty => LayoutNode::Pane(new_id),
                    LayoutNode::Pane(id) => {
                        if id == target {
                            LayoutNode::Split {
                                direction: dir,
                                children: vec![LayoutNode::Pane(id), LayoutNode::Pane(new_id)],
                            }
                        } else {
                            LayoutNode::Pane(id)
                        }
                    }
                    LayoutNode::Split { direction, children } => {
                        let new_children = children
                            .into_iter()
                            .map(|c| replace_node(c, target, dir, new_id))
                            .collect();
                        LayoutNode::Split { direction, children: new_children }
                    }
                }
            }
            active_tab.layout_root = replace_node(active_tab.layout_root.clone(), target_id, dir, new_id);
        } else {
            active_tab.layout_root = LayoutNode::Pane(new_id);
        }
        active_tab.active_pane_id = Some(new_id);
    }

    fn close_active_pane(&mut self) {
        let active_tab = &mut self.tabs[self.active_tab_index];
        let target_id = match active_tab.active_pane_id {
            Some(id) => id,
            None => {
                self.close_current_tab();
                return;
            }
        };

        self.panes.remove(&target_id);
        let new_root = remove_node_from_layout(active_tab.layout_root.clone(), target_id);
        
        if let Some(root) = new_root {
            active_tab.layout_root = root;
            let mut remaining_rects = Vec::new();
            compute_layout(&active_tab.layout_root, Rect::default(), &mut remaining_rects);
            if let Some(&(next_active_id, _)) = remaining_rects.first() {
                active_tab.active_pane_id = Some(next_active_id);
            } else {
                active_tab.active_pane_id = None;
            }
        } else {
            active_tab.layout_root = LayoutNode::Empty;
            active_tab.active_pane_id = None;
        }
    }

    fn reset_active_pane_shell(&mut self) {
        if let Some(active_id) = self.tabs[self.active_tab_index].active_pane_id {
            self.panes.remove(&active_id);
            let (writer, parser) = self.spawn_shell_for_pane(active_id);
            self.panes.insert(active_id, Pane { id: active_id, writer, parser });
        }
    }

    fn initialize_empty_pane(&mut self) {
        if self.tabs[self.active_tab_index].layout_root == LayoutNode::Empty {
            let id = self.next_pane_id;
            self.next_pane_id += 1;
            
            let (writer, parser) = self.spawn_shell_for_pane(id);
            self.panes.insert(id, Pane { id, writer, parser });
            
            let active_tab = &mut self.tabs[self.active_tab_index];
            active_tab.layout_root = LayoutNode::Pane(id);
            active_tab.active_pane_id = Some(id);
        }
    }
}

// --- LAYOUT ENGINE ---

fn compute_layout(node: &LayoutNode, area: Rect, results: &mut Vec<(PaneId, Rect)>) {
    match node {
        LayoutNode::Empty => {}
        LayoutNode::Pane(id) => {
            results.push((*id, area));
        }
        LayoutNode::Split { direction, children } => {
            if children.is_empty() { return; }
            let r_dir = match direction {
                Direction::Horizontal => RDirection::Horizontal,
                Direction::Vertical => RDirection::Vertical,
            };
            let constraints = vec![Constraint::Percentage(100 / children.len() as u16); children.len()];
            let chunks = Layout::default()
                .direction(r_dir)
                .constraints(constraints)
                .split(area);
            for (i, child) in children.iter().enumerate() {
                compute_layout(child, chunks[i], results);
            }
        }
    }
}

fn remove_node_from_layout(node: LayoutNode, target_id: PaneId) -> Option<LayoutNode> {
    match node {
        LayoutNode::Empty => None,
        LayoutNode::Pane(id) => {
            if id == target_id { None } else { Some(LayoutNode::Pane(id)) }
        }
        LayoutNode::Split { direction, children } => {
            let mut new_children = Vec::new();
            for child in children {
                if let Some(updated_child) = remove_node_from_layout(child, target_id) {
                    new_children.push(updated_child);
                }
            }
            if new_children.is_empty() {
                None
            } else if new_children.len() == 1 {
                Some(new_children[0].clone())
            } else {
                Some(LayoutNode::Split { direction, children: new_children })
            }
        }
    }
}

// --- DESIGN SCALABLE ASCII ART ENGINE ---

fn draw_scalable_splash(f: &mut Frame<'_>, area: Rect, is_focused: bool) {
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let logo = if inner_area.width >= 40 && inner_area.height >= 10 {
        vec![
            " ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄▄     ▄▄",
            "█       █       █ █   █ █",
            "█   ▄   █▄▄   ▄▄█  █▄█  █",
            "█  █▄█  █  █ █  █  ███  █",
            "█       █  █ █  █  █ █  █",
            "█   ▄   █  █ █  █  █ █  █",
            "█▄▄█ █▄▄█▄▄█▄█▄▄█▄▄█ █▄▄█",
        ]
    } else if inner_area.width >= 15 && inner_area.height >= 5 {
        vec![
            " ┌─┐┌┬┐┌─┐ ",
            " ├─┤ │ ─┬─┐",
            " ┴ ┴ ┴ ┴ ┴ ",
        ]
    } else {
        vec!["[ATM]"]
    };

    let logo_height = logo.len() as u16;
    let max_logo_width = logo.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

    if inner_area.height > logo_height && inner_area.width > max_logo_width {
        let top_padding = (inner_area.height - logo_height) / 2;
        let mut lines = vec![Line::from(""); top_padding as usize];

        for line in logo {
            let left_padding = (inner_area.width - line.chars().count() as u16) / 2;
            let padding_space = " ".repeat(left_padding as usize);
            lines.push(Line::from(vec![
                Span::raw(padding_space),
                Span::styled(line, Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
            ]));
        }

        let hint = "Press 'i' to drop into shell";
        if inner_area.height > lines.len() as u16 + 2 {
            lines.push(Line::from(""));
            let hint_padding = (inner_area.width - hint.chars().count() as u16) / 2;
            lines.push(Line::from(vec![
                Span::raw(" ".repeat(hint_padding as usize)),
                Span::styled(hint, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner_area);
    }
}

// --- RENDERING PIPELINE ---

fn render_layout_node(
    f: &mut Frame<'_>,
    app: &App,
    pane_rects: &[(PaneId, Rect)],
    main_center_rect: Rect,
) {
    let current_tab = &app.tabs[app.active_tab_index];
    
    if current_tab.layout_root == LayoutNode::Empty {
        draw_scalable_splash(f, main_center_rect, true);
        return;
    }

    let chosen_color = match app.config.active_border_color.as_str() {
        "green" => Color::Green,
        "blue" => Color::Blue,
        "red" => Color::Red,
        _ => Color::Cyan,
    };

    for &(id, rect) in pane_rects {
        let is_active = Some(id) == current_tab.active_pane_id;
        let border_style = if is_active && app.mode == Mode::Insert {
            Style::default().fg(Color::Yellow)
        } else if is_active {
            Style::default().fg(chosen_color)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);
        
        let inner_area = block.inner(rect);
        f.render_widget(block, rect);

        if let Some(pane) = app.panes.get(&id) {
            let mut parser = pane.parser.lock().unwrap();
            
            let (rows, cols) = parser.screen().size();
            if rows != inner_area.height || cols != inner_area.width {
                parser.set_size(inner_area.height, inner_area.width);
            }

            let screen = parser.screen();
            let mut lines = Vec::new();
            for r in 0..inner_area.height {
                let mut spans = Vec::new();
                for c in 0..inner_area.width {
                    if let Some(cell) = screen.cell(r, c) {
                        let mut s = Style::default();
                        if cell.bold() { s = s.add_modifier(Modifier::BOLD); }
                        if cell.italic() { s = s.add_modifier(Modifier::ITALIC); }
                        
                        let fg = match cell.fgcolor() {
                            vt100::Color::Default => Color::Reset,
                            vt100::Color::Idx(i) => Color::Indexed(i),
                            vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
                        };
                        let bg = match cell.bgcolor() {
                            vt100::Color::Default => Color::Reset,
                            vt100::Color::Idx(i) => Color::Indexed(i),
                            vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
                        };
                        spans.push(Span::styled(cell.contents().to_string(), s.fg(fg).bg(bg)));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                }
                lines.push(Line::from(spans));
            }
            let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
            f.render_widget(paragraph, inner_area);

            if is_active && app.mode == Mode::Insert {
                let (r, c) = screen.cursor_position();
                let cx = inner_area.x + c;
                let cy = inner_area.y + r;
                if cx < inner_area.x + inner_area.width && cy < inner_area.y + inner_area.height {
                    f.set_cursor(cx, cy);
                }
            }
        }
    }
}

fn ui(f: &mut Frame<'_>, app: &App) -> Vec<(PaneId, Rect)> {
    let main_chunks = Layout::default()
        .direction(RDirection::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    let mut tab_spans = Vec::new();
    for (i, tab) in app.tabs.iter().enumerate() {
        if i == app.active_tab_index {
            tab_spans.push(Span::styled(format!(" [{}] {} ", i + 1, tab.name), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        } else {
            tab_spans.push(Span::raw(format!("  {} {}  ", i + 1, tab.name)));
        }
    }
    let tab_bar = Paragraph::new(Line::from(tab_spans)).block(Block::default().borders(Borders::BOTTOM).title(" ATM MULTIPLEXER "));
    f.render_widget(tab_bar, main_chunks[0]);

    let mut pane_rects = Vec::new();
    let current_tab = &app.tabs[app.active_tab_index];
    compute_layout(&current_tab.layout_root, main_chunks[1], &mut pane_rects);
    render_layout_node(f, app, &pane_rects, main_chunks[1]);

    let mode_str = match app.mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert => " INSERT ",
        Mode::PendingNew => " SPLIT SPLASH ",
        Mode::PendingTab => " TAB MANAGEMENT ",
        Mode::RenamingTab => " RENAME TAB ",
    };
    let help_hint = match app.mode {
        Mode::Normal => "i: insert | y: yank output | n: split menu | t: tab menu | c: close pane | r: reset shell | q: quit",
        Mode::Insert => "Ctrl+g: escape to normal mode",
        Mode::PendingNew => "h: split horizontally | v: split vertically | t: new tab",
        Mode::PendingTab => "n: new tab | c: close tab | r: rename | 1-9: switch tabs",
        Mode::RenamingTab => "Enter: apply name | Esc: discard name change",
    };
    let footer_line = Line::from(vec![
        Span::styled(mode_str, Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(help_hint, Style::default().fg(Color::LightBlue)),
    ]);
    f.render_widget(Paragraph::new(footer_line), main_chunks[2]);

    if app.mode == Mode::RenamingTab {
        let block = Block::default().title(" Rename Active Tab ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
        let area = Rect::new(f.size().width / 4, f.size().height / 3, f.size().width / 2, 3);
        f.render_widget(Clear, area);
        f.render_widget(Paragraph::new(app.rename_input.as_str()).block(block), area);
    }

    pane_rects
}

// --- HELPER BUILT-IN YANK ENGINE ---

fn copy_last_output_to_clipboard(app: &App) {
    let current_tab = &app.tabs[app.active_tab_index];
    let active_id = match current_tab.active_pane_id {
        Some(id) => id,
        None => return,
    };

    if let Some(pane) = app.panes.get(&active_id) {
        let parser = pane.parser.lock().unwrap();
        let screen = parser.screen();
        let (rows, cols) = screen.size();

        let mut captured_lines = Vec::new();

        // 1. Gather text rows, filtering individual vt100 formatting memory spaces
        for r in 0..rows {
            let mut line_str = String::new();
            for c in 0..cols {
                if let Some(cell) = screen.cell(r, c) {
                    line_str.push_str(&cell.contents());
                } else {
                    line_str.push(' ');
                }
            }
            let trimmed = line_str.trim_end().to_string();
            if !trimmed.is_empty() {
                captured_lines.push(trimmed);
            }
        }

        // 2. Walk backward to scoop out information blocks between command boundaries
        let mut target_output = Vec::new();
        for line in captured_lines.iter().rev() {
            // Adjust heuristic symbol identifier rules if your shell configuration changes
            if line.contains("❯") || line.contains("$ ") || line.contains("# ") {
                break;
            }
            target_output.insert(0, line.clone());
        }

        let clean_text = target_output.join("\n");
        if !clean_text.is_empty() {
            if let Ok(mut ctx) = ClipboardContext::new() {
                let _ = ctx.set_contents(clean_text);
            }
        }
    }
}

// --- INPUT MANAGEMENT ENGINE ---

fn handle_key_event(app: &mut App, key: KeyEvent, pane_rects: &[(PaneId, Rect)]) {
    match app.mode {
        Mode::Normal => {
            match key.code {
                KeyCode::Char('i') => {
                    if app.tabs[app.active_tab_index].layout_root == LayoutNode::Empty {
                        app.initialize_empty_pane();
                    }
                    app.mode = Mode::Insert;
                }
                KeyCode::Char('y') => {
                    copy_last_output_to_clipboard(app);
                }
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('n') => app.mode = Mode::PendingNew, 
                KeyCode::Char('t') => app.mode = Mode::PendingTab,
                KeyCode::Char('r') => app.reset_active_pane_shell(),
                KeyCode::Char('c') => app.close_active_pane(),
                KeyCode::Char('h') | KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Char('l') => {
                    if let Some(active_pane_id) = app.tabs[app.active_tab_index].active_pane_id {
                        if let Some(&(_, current_rect)) = pane_rects.iter().find(|(id, _)| *id == active_pane_id) {
                            let current_cx = current_rect.x as i32 + (current_rect.width as i32 / 2);
                            let current_cy = current_rect.y as i32 + (current_rect.height as i32 / 2);
                            
                            let mut best_match: Option<(PaneId, i32)> = None;
                            
                            for &(id, rect) in pane_rects {
                                if id == active_pane_id { continue; }
                                
                                let target_cx = rect.x as i32 + (rect.width as i32 / 2);
                                let target_cy = rect.y as i32 + (rect.height as i32 / 2);

                                let is_valid_direction = match key.code {
                                    KeyCode::Char('h') => target_cx < current_cx && (target_cy - current_cy).abs() < (rect.height as i32),
                                    KeyCode::Char('l') => target_cx > current_cx && (target_cy - current_cy).abs() < (rect.height as i32),
                                    KeyCode::Char('k') => target_cy < current_cy && (target_cx - current_cx).abs() < (rect.width as i32),
                                    KeyCode::Char('j') => target_cy > current_cy && (target_cx - current_cx).abs() < (rect.width as i32),
                                    _ => false,
                                };

                                if is_valid_direction {
                                    let dist = (target_cx - current_cx).pow(2) + (target_cy - current_cy).pow(2);
                                    if best_match.map_or(true, |(_, d)| dist < d) {
                                        best_match = Some((id, dist));
                                    }
                                }
                            }
                            if let Some((next_id, _)) = best_match {
                                app.tabs[app.active_tab_index].active_pane_id = Some(next_id);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Mode::Insert => {
            if key.code == KeyCode::Char('g') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.mode = Mode::Normal;
                return;
            }
            
            if let Some(active_id) = app.tabs[app.active_tab_index].active_pane_id {
                if let Some(pane) = app.panes.get_mut(&active_id) {
                    let mut bytes_to_write = Vec::new();

                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if let KeyCode::Char(c) = key.code {
                            if c.is_ascii_alphabetic() {
                                let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                                let control_byte = c as u8 - base + 1;
                                bytes_to_write.push(control_byte);
                            }
                        }
                    } else {
                        match key.code {
                            KeyCode::Char(c) => {
                                let mut buf = [0; 4];
                                let s = c.encode_utf8(&mut buf);
                                bytes_to_write.extend_from_slice(s.as_bytes());
                            }
                            KeyCode::Enter => bytes_to_write.push(b'\r'), 
                            KeyCode::Backspace => bytes_to_write.push(127),
                            KeyCode::Tab => bytes_to_write.push(b'\t'),
                            KeyCode::Esc => bytes_to_write.push(27), 
                            KeyCode::Up => bytes_to_write.extend_from_slice(&[27, b'[', b'A']),   
                            KeyCode::Down => bytes_to_write.extend_from_slice(&[27, b'[', b'B']), 
                            KeyCode::Right => bytes_to_write.extend_from_slice(&[27, b'[', b'C']), 
                            KeyCode::Left => bytes_to_write.extend_from_slice(&[27, b'[', b'D']), 
                            _ => {}
                        }
                    }

                    if !bytes_to_write.is_empty() {
                        let _ = pane.writer.write_all(&bytes_to_write);
                        let _ = pane.writer.flush();
                    }
                }
            }
        }
        Mode::PendingNew => {
            match key.code {
                KeyCode::Char('h') => app.split_active_pane(Direction::Horizontal),
                KeyCode::Char('v') => app.split_active_pane(Direction::Vertical),
                KeyCode::Char('t') => app.spawn_new_tab(), 
                _ => {}
            }
            app.mode = Mode::Normal;
        }
        Mode::PendingTab => {
            match key.code {
                KeyCode::Char('n') => app.spawn_new_tab(),
                KeyCode::Char('c') => app.close_current_tab(),
                KeyCode::Char('r') => {
                    app.rename_input = app.tabs[app.active_tab_index].name.clone();
                    app.mode = Mode::RenamingTab;
                    return;
                }
                KeyCode::Char(ch) if ch.is_ascii_digit() => {
                    if let Some(digit) = ch.to_digit(10) {
                        let target_idx = (digit as usize).saturating_sub(1);
                        if target_idx < app.tabs.len() { app.active_tab_index = target_idx; }
                    }
                }
                _ => {}
            }
            app.mode = Mode::Normal;
        }
        Mode::RenamingTab => {
            match key.code {
                KeyCode::Enter => {
                    app.tabs[app.active_tab_index].name = app.rename_input.clone();
                    app.mode = Mode::Normal;
                }
                KeyCode::Esc => app.mode = Mode::Normal,
                KeyCode::Char(c) => app.rename_input.push(c),
                KeyCode::Backspace => { app.rename_input.pop(); }
                _ => {}
            }
        }
    }
}

// --- RUNTIME ROOT ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut pane_rects = Vec::new();

    loop {
        if app.should_quit { break; }

        terminal.draw(|f| {
            pane_rects = ui(f, &app);
        })?;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    handle_key_event(&mut app, key, &pane_rects);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}