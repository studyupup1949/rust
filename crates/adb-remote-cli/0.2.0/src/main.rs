use clap::Parser;

use crossterm::{
    cursor,
    event::{
        self, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, Event, KeyCode,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const ORIGIN_X: u16 = 4;
const ORIGIN_Y: u16 = 2;
const WIDTH: u16 = 28;
const HEIGHT: u16 = 18;

#[derive(Clone, Copy, PartialEq)]
enum Button {
    Power,
    Menu,
    Up,
    Down,
    Left,
    Right,
    Ok,
    VolDown,
    Mute,
    VolUp,
    Rewind,
    PlayPause,
    FastFwd,
    Home,
    Back,
}

impl Button {
    const ALL: [Button; 15] = [
        Button::Power,
        Button::Menu,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::Ok,
        Button::VolDown,
        Button::Mute,
        Button::VolUp,
        Button::Rewind,
        Button::PlayPause,
        Button::FastFwd,
        Button::Home,
        Button::Back,
    ];
    // label + position relative to ORIGIN
    fn info(self) -> (&'static str, u16, u16) {
        match self {
            Button::Power => (" [PWR] ", 4, 2),
            Button::Menu => (" [MENU] ", 17, 2),
            Button::Up => (" ^ ", 13, 5),
            Button::Left => (" < ", 9, 7),
            Button::Ok => (" O K ", 12, 7),
            Button::Right => (" > ", 17, 7),
            Button::Down => (" v ", 13, 9),
            Button::VolDown => (" VOL- ", 4, 12),
            Button::Mute => (" MUTE ", 11, 12),
            Button::VolUp => (" VOL+ ", 18, 12),
            Button::Rewind => (" << ", 5, 14),
            Button::PlayPause => (" |> ", 13, 14),
            Button::FastFwd => (" >> ", 19, 14),
            Button::Home => (" HOME ", 5, 16),
            Button::Back => (" BACK ", 17, 16),
        }
    }

    fn from_key(code: KeyCode) -> Option<Button> {
        match code {
            KeyCode::Up => Some(Button::Up),
            KeyCode::Down => Some(Button::Down),
            KeyCode::Left => Some(Button::Left),
            KeyCode::Right => Some(Button::Right),
            KeyCode::Enter => Some(Button::Ok),
            KeyCode::Char('p') => Some(Button::Power),
            KeyCode::Char('m') => Some(Button::Menu),
            KeyCode::Char('-') => Some(Button::VolDown),
            KeyCode::Char('=') | KeyCode::Char('+') => Some(Button::VolUp),
            KeyCode::Char('0') => Some(Button::Mute),
            KeyCode::Char('[') => Some(Button::Rewind),
            KeyCode::Char(']') => Some(Button::FastFwd),
            KeyCode::Char(' ') => Some(Button::PlayPause),
            KeyCode::Char('h') => Some(Button::Home),
            KeyCode::Backspace => Some(Button::Back),
            _ => None,
        }
    }

    fn adb_code(self) -> &'static str {
        match self {
            Button::Power => "26",
            Button::Menu => "82",
            Button::Up => "19",
            Button::Down => "20",
            Button::Left => "21",
            Button::Right => "22",
            Button::Ok => "23",
            Button::VolDown => "25",
            Button::VolUp => "24",
            Button::Mute => "164",
            Button::Rewind => "89",
            Button::PlayPause => "85",
            Button::FastFwd => "90",
            Button::Home => "3",
            Button::Back => "4",
        }
    }
    fn is_clicked(col: u16, row: u16) -> Option<Button> {
        for b in Button::ALL {
            let (label, x, y) = b.info();
            let bx = ORIGIN_X + x;
            let by = ORIGIN_Y + y;
            let bwidth = label.len() as u16;
            if row == by && col >= bx && col < bx + bwidth {
                return Some(b);
            }
        }
        None
    }
}

enum ConnectionStatus {
    Connected(String),
    Disconnected,
}

enum ActionStatus {
    Idle,
    Sent(&'static str),
    Failed(&'static str),
}

#[derive(Parser, Debug)]
#[command(name = "adb-remote", version, about)]
struct Args {
    /// Connect to a device over network ADB, e.g. -c 192.168.1.71:5555
    #[arg(short = 'c', long = "connect")]
    connect: Option<String>,
}

fn draw_frame(out: &mut impl Write) -> io::Result<()> {
    queue!(out, Clear(ClearType::All))?;
    queue!(out, SetForegroundColor(Color::Cyan));

    // top border
    queue!(out, cursor::MoveTo(ORIGIN_X, ORIGIN_Y))?;
    queue!(out, Print(format!("┌{}┐", "─".repeat(WIDTH as usize - 2))))?;

    // side borders
    for row in 1..HEIGHT - 1 {
        queue!(out, cursor::MoveTo(ORIGIN_X, ORIGIN_Y + row))?;
        queue!(out, Print("│"))?;
        queue!(out, cursor::MoveTo(ORIGIN_X + WIDTH - 1, ORIGIN_Y + row))?;
        queue!(out, Print("│"))?;
    }

    // bottom border
    queue!(out, cursor::MoveTo(ORIGIN_X, ORIGIN_Y + HEIGHT - 1))?;
    queue!(out, Print(format!("└{}┘", "─".repeat(WIDTH as usize - 2))))?;

    queue!(out, ResetColor)?;

    // draw every button in normal state
    for b in [
        Button::Power,
        Button::Menu,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::Ok,
        Button::VolDown,
        Button::Mute,
        Button::VolUp,
        Button::Rewind,
        Button::PlayPause,
        Button::FastFwd,
        Button::Home,
        Button::Back,
    ] {
        draw_button(out, b, false)?;
    }

    Ok(())
}

fn draw_button(out: &mut impl Write, b: Button, highlighted: bool) -> io::Result<()> {
    let (label, x, y) = b.info();
    queue!(out, cursor::MoveTo(ORIGIN_X + x, ORIGIN_Y + y))?;
    if highlighted {
        queue!(
            out,
            SetForegroundColor(Color::Green),
            SetBackgroundColor(Color::Black)
        )?;
    } else {
        queue!(out, SetForegroundColor(Color::White))?;
    }
    queue!(out, Print(label))?;
    queue!(out, ResetColor)?;
    Ok(())
}

fn draw_status(
    out: &mut impl Write,
    connection_status: &ConnectionStatus,
    action_status: &ActionStatus,
) -> io::Result<()> {
    let (_cols, rows) = terminal::size()?;

    queue!(out, cursor::MoveTo(ORIGIN_X, rows.saturating_sub(6)))?;
    queue!(out, Clear(ClearType::CurrentLine))?;
    queue!(out, Print("Arrows/Enter=DPAD  h=home  m=menu"))?;

    queue!(out, cursor::MoveTo(ORIGIN_X, rows.saturating_sub(5)))?;
    queue!(out, Print("Backspace=back     p=power q=quit"))?;

    queue!(out, cursor::MoveTo(ORIGIN_X, rows.saturating_sub(3)))?;
    queue!(out, Clear(ClearType::CurrentLine))?;
    match connection_status {
        ConnectionStatus::Connected(serial) => {
            queue!(out, SetForegroundColor(Color::DarkGrey))?;
            queue!(out, Print(format!("● Connected ({serial})")))?;
        }
        ConnectionStatus::Disconnected => {
            queue!(out, SetForegroundColor(Color::Red))?;
            queue!(out, Print("○ No device"))?;
        }
    }
    queue!(out, ResetColor)?;

    queue!(out, cursor::MoveTo(ORIGIN_X, rows.saturating_sub(2)))?;
    queue!(out, Clear(ClearType::CurrentLine))?;

    match action_status {
        ActionStatus::Idle => {
            queue!(out, SetForegroundColor(Color::DarkGrey))?;
            queue!(out, Print("Ready"))?;
        }
        ActionStatus::Sent(label) => {
            queue!(out, SetForegroundColor(Color::DarkGrey))?;
            queue!(out, Print(format!("Sent: {label}")))?;
        }
        ActionStatus::Failed(label) => {
            queue!(out, SetForegroundColor(Color::Red))?;
            queue!(out, Print(format!("Failed: {label}")))?;
        }
    }
    queue!(out, ResetColor)?;

    Ok(())
}

fn check_adb_installed() -> bool {
    Command::new("adb").arg("version").output().is_ok()
}

fn check_adb_connection() -> ConnectionStatus {
    // "adb get-state" returns "device" on stdout when a device is connected
    let output = Command::new("adb").args(["get-state"]).output();
    match output {
        Ok(o) if o.status.success() => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if state == "device" {
                // grab the serial too
                let serial = Command::new("adb")
                    .args(["get-serialno"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_else(|_| "unknown".into());
                ConnectionStatus::Connected(serial)
            } else {
                ConnectionStatus::Disconnected
            }
        }
        _ => ConnectionStatus::Disconnected,
    }
}

fn main() -> io::Result<()> {
    if !check_adb_installed() {
        eprintln!("Error: `adb` was not found on your PATH.");
        eprintln!();
        eprintln!("adb-remote requires Android Debug Bridge (adb) to be installed.");
        eprintln!("Install it via:");
        eprintln!("  - macOS:   brew install android-platform-tools");
        eprintln!("  - Linux:   sudo apt install adb   (or your distro's package manager)");
        eprintln!("  - Windows: https://developer.android.com/tools/releases/platform-tools");
        std::process::exit(1);
    }

    let args = Args::parse();

    if let Some(addr) = &args.connect {
        eprintln!("Connecting to: {:?}", addr); // {:?} shows hidden chars/quotes
        spawn_adb_connection(addr)?;
    }
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    execute!(
        stdout,
        EnableBracketedPaste,
        EnableFocusChange,
        EnableMouseCapture,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
    )?;

    let result = run(&mut stdout);

    execute!(stdout, cursor::Show, ResetColor);
    terminal::disable_raw_mode()?;
    execute!(stdout, ResetColor)?;
    execute!(stdout, LeaveAlternateScreen);
    result
}

fn run(stdout: &mut impl Write) -> io::Result<()> {
    let (mut _child, mut child_stdin) = spawn_adb_process()?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            tx.send(check_adb_connection());
            thread::sleep(Duration::from_secs(2));
        }
    });

    let mut connection_status = ConnectionStatus::Disconnected;
    let mut action_status = ActionStatus::Idle;
    let mut highlight: Option<(Button, Instant)> = None;

    draw_frame(stdout)?;
    draw_status(stdout, &connection_status, &action_status)?;
    stdout.flush()?;

    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(event) => {
                    if event.code == event::KeyCode::Char('q')
                        || event.code == event::KeyCode::Esc
                        || (event.modifiers.contains(event::KeyModifiers::CONTROL)
                            && event.code == event::KeyCode::Char('c'))
                    {
                        break;
                    }
                    if let Some(button) = Button::from_key(event.code) {
                        let code = button.adb_code();

                        match send_keyevent(&mut child_stdin, code) {
                            Ok(_) => action_status = ActionStatus::Sent(code),
                            Err(_) => action_status = ActionStatus::Failed(code),
                        }
                        draw_button(stdout, button, true);
                        draw_status(stdout, &connection_status, &action_status)?;
                        highlight = Some((button, Instant::now()));
                        stdout.flush()?;
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: _,
                }) => {
                    if let Some(button) = Button::is_clicked(column, row) {
                        send_keyevent(&mut child_stdin, button.adb_code());
                        draw_button(stdout, button, true)?;
                        stdout.flush()?;
                        std::thread::sleep(std::time::Duration::from_millis(120));
                        draw_button(stdout, button, false)?;
                        stdout.flush()?;
                    }
                }
                Event::Resize(_, _) => {
                    draw_frame(stdout)?;
                    draw_status(stdout, &connection_status, &action_status)?;
                    stdout.flush()?;
                }
                _ => (),
            }
        }
        if let Some((button, t)) = highlight {
            if t.elapsed() >= Duration::from_millis(150) {
                draw_button(stdout, button, false)?;
                highlight = None;
                stdout.flush()?;
            }
        }
        if let Ok(new_status) = rx.try_recv() {
            connection_status = new_status;
            draw_status(stdout, &connection_status, &action_status)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

fn spawn_adb_connection(addr: &str) -> io::Result<()> {
    let output = Command::new("adb").args(["connect", addr]).output()?;
    let msg = String::from_utf8_lossy(&output.stdout);
    eprintln!("{}", msg.trim());
    if !msg.contains("connected") {
        eprintln!("Failed to connect to {addr}, exiting.");
        std::process::exit(1);
    }
    Ok(())
}

fn spawn_adb_process() -> io::Result<(Child, ChildStdin)> {
    let mut child = Command::new("adb")
        .args(["shell"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    Ok((child, stdin))
}

fn send_keyevent(stdin: &mut ChildStdin, code: &str) -> io::Result<()> {
    writeln!(stdin, "input keyevent {}", code)?;
    stdin.flush()
}
