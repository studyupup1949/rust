mod app;
mod monitor;
mod game;
mod ui;
mod i18n;

use std::io;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
};
use ratatui::prelude::*;

use app::{App, MonitorKind};
use game::save::SaveManager;
use game::GameState;
use monitor::{MockMonitor, TokenMonitor};
use ui::Renderer;
use ui::theme::Theme;

#[derive(Parser)]
#[command(name = "abyss-protocol")]
#[command(about = "Abyss Protocol - Cyber Elder God CLI Idle Game")]
#[command(version)]
struct Cli {
    /// Language: en or zh (default: auto-detect)
    #[arg(long, default_value = None)]
    lang: Option<String>,

    /// Enable mock mode (simulated token events for testing)
    #[arg(long)]
    mock: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. 检测语言，创建 LocaleManager
    let locale = i18n::detect_locale(cli.lang.as_deref());
    let locale_manager = i18n::LocaleManager::load(locale)?;

    // 2. 创建 SaveManager，尝试加载存档
    let save_manager = SaveManager::new()?;
    let game_state = match save_manager.load()? {
        Some(state) => state,
        None => GameState::new(),
    };

    // 3. 创建 monitor（根据 --mock 标志）
    let monitor = if cli.mock {
        MonitorKind::Mock(MockMonitor::start())
    } else {
        MonitorKind::Real(TokenMonitor::new()?)
    };

    // 4. 创建 App
    let mut app = App::new(locale_manager, game_state, save_manager, monitor, cli.mock);

    // 5. 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 6. 设置 panic hook 恢复终端
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        original_hook(panic_info);
    }));

    // 7. 创建渲染器
    let renderer = Renderer::new(Theme::default());

    // 8. 主循环 (~100ms / 10 FPS)
    loop {
        // 渲染
        terminal.draw(|frame| {
            renderer.render(frame, &mut app);
        })?;

        // Poll 输入事件（100ms 超时）
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // crossterm 0.28+ 在 Windows 上会发送 Press 和 Release 事件，只处理 Press
                if key.kind == KeyEventKind::Press {
                    // 崩溃序列期间阻止所有输入
                    if !app.animation.collapse.blocks_input() {
                        app.handle_key_event(key);
                    }
                }
            }
        }

        // Tick: poll token events + update state + auto-save check
        app.tick();

        // 动画系统更新（在 tick 之后，使用最新的游戏状态）
        let size = terminal.size()?;
        let screen_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let whispers = app.game_state.currency.whispers;
        let san = app.game_state.san.current;
        app.animation.update(0.1, whispers, san, screen_area, &mut rand::thread_rng());

        // 检查是否退出
        if app.should_quit() {
            break;
        }
    }

    // 9. 退出时存档
    let _ = app.save_manager.save(&app.game_state);

    // 10. 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

    Ok(())
}
