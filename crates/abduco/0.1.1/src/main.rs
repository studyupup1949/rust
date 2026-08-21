#![allow(unused_variables, unused_assignments, dead_code)]

use std::env;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;
use std::time::SystemTime;

use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;

use nix::pty::Winsize;
use nix::sys::termios::Termios;

macro_rules! CTRL {
    ( $ch:expr ) => {
        $ch as u8 & 0x1F
    };
}

const ABDUCO_CMD: &'static str = "dvtm";

#[derive(Debug)]
struct Dir {
    path: Box<Path>,
    personal: bool,
}

#[derive(Debug)]
enum State {
    StateConnected,
    StateAttached,
    StateDetached,
    StateDisconnected,
}

#[derive(Debug)]
enum Flags {
    ClientReadonly,
    ClientLowpriority,
}

#[derive(Debug)]
struct Client {
    socket: u32,
    state: State,
    need_resize: bool,
    flags: Flags,
    next: Box<Client>,
}

#[derive(Debug)]
struct U {
    msg: [char; 4096],
    ws: (u16, u16),
    i_32: u32,
    i_64: u64,
}

#[derive(Debug)]
struct Packet {
    packet_type: u32,
    len: u32,
    u: U,
}

#[derive(Debug)]
struct Server {
    clients: Vec<Client>,
    socket: u32,
    pty_output: Packet,
    pty: u32,
    exit_status: u32,
    term: Termios,
    winsize: Winsize,
    pid: i32,     // pid_t
    running: i32, // sig_atomic_t
    name: String,
    session_name: String,
    host: [u8; 255],
    read_pty: bool,
}

impl Dir {
    fn form_env(env: &str, personal: bool) -> Self {
        let path = env::var(env).unwrap_or("".to_string());
        Self {
            path: Path::new(&path).into(),
            personal,
        }
    }

    fn from_path(path: &str, personal: bool) -> Self {
        Self {
            path: Path::new(&path).into(),
            personal,
        }
    }
}

fn usage() -> ExitCode {
    println!("usage: abduco [-a|-A|-c|-n] [-r] [-l] [-f] [-e detachkey] name command");
    ExitCode::FAILURE
}

fn is_home(path: &Path) -> bool {
    path == Path::new(&env::var("HOME").unwrap())
}

fn basename(s: &str) -> &str {
    Path::new(s).file_name().unwrap().to_str().unwrap()
}

fn hostname() -> String {
    nix::unistd::gethostname()
        .expect("ERROR: gethostname")
        .into_string()
        .expect("ERROR: invalid hostname may not be UTF-8")
}

fn list_sessions(socket_dirs: &[Dir], base_name: &str) -> ExitCode {
    let mut session_dir = String::new();
    for dir in socket_dirs {
        if dir.path == Path::new("").into() {
            continue;
        }
        session_dir = if is_home(&dir.path) {
            format!("{}/.{}/", dir.path.display(), base_name)
        } else {
            format!("{}/{}/", dir.path.display(), base_name)
        };

        if Path::new(&session_dir).is_dir() {
            break;
        }
    }

    println!("Active sessions (on host {})", hostname());
    for entry in std::fs::read_dir(session_dir).unwrap() {
        match entry {
            Ok(e) => {
                let en = format!("{}", e.path().display());
                if !e.file_type().unwrap().is_socket() {
                    continue;
                }

                let metadata = e.metadata().unwrap();
                let last_modified = metadata.modified().unwrap();
                let foo = last_modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();

                let date_format = time::macros::format_description!("[weekday repr:short]\t [year repr:full]-[month]-[day] [hour]:[minute]:[second]");
                let f = time::OffsetDateTime::from_unix_timestamp(0).unwrap() + foo;
                let offset = time::UtcOffset::current_local_offset().unwrap();
                let f = f.to_offset(offset);
                let time = f.format(date_format).unwrap();

                let mode = metadata.mode();
                let status = if mode & 00100 != 0 {
                    // S_IXUSR  00100
                    '*'
                } else if mode & 00010 != 0 {
                    // S_IXGRP  00010
                    '+'
                } else {
                    ' '
                };

                if en.contains(&hostname()) {
                    match en.find(&hostname()) {
                        Some(idx) => {
                            let pid = get_pid_of_session(&en).unwrap();
                            eprintln!("{status} {time}\t{pid}\t{}", basename(&en[0..idx - 1]));
                        }
                        None => todo!(),
                    }
                }
            }
            Err(_) => return ExitCode::FAILURE,
        }
    }

    ExitCode::SUCCESS
}

fn create_session(session_name: &str) {
    println!("create_session: {session_name}");
}

fn attach_session(session_name: &str) {
    println!("attach_session: {session_name}");
}

fn get_pid_of_session(session_name: &str) -> Result<u64, std::io::Error> {
    let mut socket = UnixStream::connect(session_name)?;

    let mut response: [u8; 8] = [0; 8];
    socket.read_exact(&mut response)?;

    let type_of_response = u32::from_le_bytes(
        response[0..4]
            .try_into()
            .expect("attach_session: Unable to read the `type` from `Packet`"),
    );
    if type_of_response != 5 {
        // MSG_PID = 5
        todo!();
    }

    let length_to_read = u32::from_le_bytes(
        response[4..8]
            .try_into()
            .expect("attach_session: Unable to read the `length` from `Packet`"),
    );

    let mut buf: Vec<u8> = vec![0; length_to_read as usize];
    socket.read_exact(&mut buf)?;

    Ok(u64::from_le_bytes(
        buf[0..length_to_read as usize]
            .try_into()
            .expect("attach_session: Unable to read the `PID` from `Packet`"),
    ))
}

fn session_exists(session_name: &str) -> bool {
    println!("session_exists: {session_name}");
    false
}

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().expect("program name");
    let mut args = args.collect::<Vec<_>>();
    args.reverse();
    let socket_dirs: [Dir; 4] = [
        Dir::form_env("ABDUCO_SOCKET_DIR", false),
        Dir::form_env("HOME", true),
        Dir::form_env("TMPDIR", false),
        Dir::from_path("/tmp", false),
    ];

    if args.is_empty() {
        return list_sessions(&socket_dirs, basename(&program));
    }

    let mut action = ' ';
    let mut force = false;
    let mut passthrough = false;
    let mut quiet = false;

    let mut key_detach = CTRL!('\\');
    let mut handle_e = false;

    while let Some(arg) = args.pop() {
        if !arg.starts_with("-") {
            if handle_e {
                let mut chars = arg.chars();
                if let Some(x) = chars.next() {
                    if x == '^' {
                        if let Some(x) = chars.next() {
                            key_detach = CTRL!(x);
                        }
                    } else {
                        key_detach = x as u8;
                    }
                }
                handle_e = false;
            }

            args.push(arg);
            break;
        }
        let mut chars = arg.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '-' => continue,
                'a' => action = 'a',
                'A' => action = 'A',
                'c' => action = 'c',
                'n' => action = 'n',
                'e' => {
                    if let Some(x) = chars.next() {
                        if x == '^' {
                            if let Some(x) = chars.next() {
                                key_detach = CTRL!(x);
                            }
                        } else {
                            key_detach = x as u8;
                        }
                    } else {
                        handle_e = true;
                    }
                    break;
                }
                'f' => force = true,
                'p' => passthrough = true,
                'q' => quiet = true,
                'r' => todo!("readonly"),
                'l' => todo!("low priority"),
                'v' => {
                    let version = env!("CARGO_PKG_VERSION");
                    println!("abduco-{version} © 2024 Syed Fasiuddin");
                    return ExitCode::SUCCESS;
                }
                x => return usage(),
            }
        }
    }

    let session_name = match args.pop() {
        Some(name) => name,
        None => return usage(),
    };

    let command = match args.pop() {
        Some(name) => name,
        None => ABDUCO_CMD.to_string(),
    };
    if !args.is_empty() {
        args.reverse();
    }

    match action {
        'A' => {
            if !session_exists(&session_name) {
                create_session(&session_name);
            }
            attach_session(&session_name);
        }
        'a' => {
            if !session_exists(&session_name) {
                eprintln!("error: session does not exist");
                return ExitCode::FAILURE;
            }
            attach_session(&session_name);
        }
        'c' => {
            if session_exists(&session_name) {
                eprintln!("error: session already exist");
                return ExitCode::FAILURE;
            }
            create_session(&session_name);
            attach_session(&session_name);
        }
        'n' => {
            if session_exists(&session_name) {
                eprintln!("error: session already exist");
                return ExitCode::FAILURE;
            }
            create_session(&session_name);
        }
        _ => return usage(),
    }

    ExitCode::SUCCESS
}
