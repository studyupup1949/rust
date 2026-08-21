//! Helpers. Port of `adbutils/_utils.py` (subset; extended in later milestones).

use std::time::Duration;

use crate::errors::{AdbError, Result};

/// Environment variable overriding the adb binary path (`ADBUTILS_ADB_PATH`).
const ENV_ADB_PATH: &str = "ADBUTILS_ADB_PATH";

/// Resolve the adb binary used only for `start-server`.
///
/// Resolution order: `ADBUTILS_ADB_PATH` env → bundled binary staged by
/// `build.rs` (`ADBUTILS_BUNDLED_ADB`, if it exists) → `adb` on `PATH`.
pub fn adb_path() -> String {
    if let Ok(p) = std::env::var(ENV_ADB_PATH) {
        if !p.is_empty() {
            return p;
        }
    }
    if let Some(p) = option_env!("ADBUTILS_BUNDLED_ADB") {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }
    "adb".to_string()
}

/// Locate `bin` on `PATH` (like `shutil.which`). Returns the first executable
/// match, honoring `PATHEXT` on Windows.
pub fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "bat", "cmd"] {
                let c = dir.join(format!("{bin}.{ext}"));
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

const MB: f64 = 1024.0 * 1024.0;

/// Human-readable byte size in MB (`humanize`).
pub fn humanize(n: u64) -> String {
    format!("{:.1} MB", n as f64 / MB)
}

/// Best-effort primary local IP via a UDP connect trick (`current_ip`).
pub fn current_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            Ok(s.local_addr()?.ip().to_string())
        })
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Escape a large set of shell/regex metacharacters (`escape_special_characters`).
/// Note the quirky mappings from the Python original: space → `%s`, backslash and
/// quotes are doubled.
pub fn escape_special_characters(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str(r"\\\\"),
            '\'' => out.push_str(r"\\'"),
            '"' => out.push_str("\\\\\""),
            ' ' => out.push_str("%s"),
            '-' | '+' | '[' | ']' | '(' | ')' | '{' | '}' | '^' | '$' | '*' | '.' | ',' | ':'
            | '~' | ';' | '>' | '<' | '%' | '#' | '`' | '!' | '?' | '|' | '=' | '@' | '/' | '_'
            | '&' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Run `adb start-server` once (mirrors the fallback in `_safe_connect`).
pub async fn start_server() -> Result<()> {
    let adb = adb_path();
    let child = tokio::process::Command::new(&adb)
        .arg("start-server")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| AdbError::Connection(format!("failed to run `{adb} start-server`: {e}")))?;
    let out = tokio::time::timeout(Duration::from_secs(20), async move {
        child.wait_with_output().await
    })
    .await
    .map_err(|_| AdbError::Timeout("adb start-server timeout".into()))?
    .map_err(|e| AdbError::Connection(format!("adb start-server failed: {e}")))?;
    if !out.status.success() {
        return Err(AdbError::Connection(format!(
            "adb start-server exited with {}",
            out.status
        )));
    }
    Ok(())
}

/// shlex-style quoting of a single argument. Mirrors `shlex.quote`.
fn shlex_quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    let safe = arg.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'@' | b'%' | b'_' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'-')
    });
    if safe {
        return arg.to_string();
    }
    // Wrap in single quotes, escaping embedded single quotes as '"'"'.
    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

/// Join args into a single shell command string using shlex quoting.
/// Mirrors `adbutils/_utils.py::list2cmdline` (uses `shlex.quote`, not
/// `subprocess.list2cmdline`).
pub fn list2cmdline<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .map(|a| shlex_quote(a.as_ref()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Grab a free local TCP port by binding to port 0 (`get_free_port`).
pub fn get_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| AdbError::Connection(format!("cannot get free port: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AdbError::Connection(format!("cannot get free port: {e}")))?
        .port();
    Ok(port)
}

/// POSIX-join two path fragments, avoiding a double slash (`append_path`).
pub fn append_path(base: &str, addition: &str) -> String {
    let base = base.trim_end_matches('/');
    let addition = addition.trim_start_matches('/');
    format!("{base}/{addition}")
}

/// Command input accepted by `shell`/`shell2`: either a pre-joined string or an
/// argument list (quoted via [`list2cmdline`]).
pub enum CmdArgs {
    Str(String),
    List(Vec<String>),
}

impl CmdArgs {
    /// Render to the final command string sent over the wire.
    pub fn to_cmdline(&self) -> String {
        match self {
            CmdArgs::Str(s) => s.clone(),
            CmdArgs::List(v) => list2cmdline(v),
        }
    }
}

impl From<&str> for CmdArgs {
    fn from(s: &str) -> Self {
        CmdArgs::Str(s.to_string())
    }
}

impl From<String> for CmdArgs {
    fn from(s: String) -> Self {
        CmdArgs::Str(s)
    }
}

impl From<&String> for CmdArgs {
    fn from(s: &String) -> Self {
        CmdArgs::Str(s.clone())
    }
}

impl From<Vec<String>> for CmdArgs {
    fn from(v: Vec<String>) -> Self {
        CmdArgs::List(v)
    }
}

impl From<Vec<&str>> for CmdArgs {
    fn from(v: Vec<&str>) -> Self {
        CmdArgs::List(v.into_iter().map(String::from).collect())
    }
}

impl<const N: usize> From<[&str; N]> for CmdArgs {
    fn from(v: [&str; N]) -> Self {
        CmdArgs::List(v.into_iter().map(String::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list2cmdline_quotes_specials() {
        // Matches Python shlex.quote: safe tokens untouched, specials single-quoted.
        assert_eq!(list2cmdline(["echo", "hello"]), "echo hello");
        assert_eq!(list2cmdline(["echo", "&"]), "echo '&'");
        assert_eq!(list2cmdline(["a b", "c"]), "'a b' c");
        assert_eq!(list2cmdline(["ro.serial"]), "ro.serial");
        assert_eq!(list2cmdline([""]), "''");
    }

    #[test]
    fn append_path_avoids_double_slash() {
        assert_eq!(append_path("/data/local/tmp", "x"), "/data/local/tmp/x");
        assert_eq!(append_path("/data/local/tmp/", "x"), "/data/local/tmp/x");
        assert_eq!(append_path("/a", "/b"), "/a/b");
    }

    #[test]
    fn humanize_formats_mb() {
        assert_eq!(humanize(1024 * 1024), "1.0 MB");
        assert_eq!(humanize(1024 * 1024 * 3 / 2), "1.5 MB");
    }

    #[test]
    fn escape_special_characters_matches_python() {
        // space -> %s, ampersand escaped.
        assert_eq!(escape_special_characters("a b"), r"a%sb");
        assert_eq!(escape_special_characters("a&b"), r"a\&b");
        assert_eq!(escape_special_characters("a.b"), r"a\.b");
    }
}
