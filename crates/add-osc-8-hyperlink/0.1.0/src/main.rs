use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

const OSC: &str = "\x1b]";
const BEL: &str = "\x07";

fn make_hyperlink(url: &str, text: &str) -> String {
    format!("{OSC}8;;{url}{BEL}{text}{OSC}8;;{BEL}")
}

fn build_pattern(prefixes: &[String]) -> String {
    format!(r#"(?:{})(?:/[^$\s;~:"\x1b]+)?"#, prefixes.join("|"))
}

fn process_line(
    line: &str,
    re: &Regex,
    hostname: &str,
    home: &str,
    cwd: &Path,
) -> String {
    re.replace_all(line, |caps: &regex::Captures| {
        let matched = &caps[0];

        // Expand ~ to home directory
        let expanded = if matched.starts_with("~/") {
            format!("{}{}", home, &matched[1..])
        } else {
            matched.to_string()
        };

        // Convert to absolute path
        let abs_path = if Path::new(&expanded).is_absolute() {
            expanded
        } else {
            cwd.join(&expanded).to_string_lossy().into_owned()
        };

        let url = format!("file://{}{}", hostname, abs_path);
        make_hyperlink(&url, matched)
    })
    .into_owned()
}

fn get_prefixes(cwd: &Path) -> Vec<String> {
    let mut prefixes: Vec<String> = vec![
        "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib64",
        "/lost+found", "/mnt", "/opt", "/proc", "/root", "/run",
        "/sbin", "/srv", "/sys", "/tmp", "/usr", "/var",
    ]
    .into_iter()
    .map(|s| regex::escape(s))
    .collect();

    // Add current directory entries as relative path prefixes
    if let Ok(entries) = fs::read_dir(cwd) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                prefixes.push(regex::escape(name));
            }
        }
    }

    // Add home directory prefix
    prefixes.push(regex::escape("~"));

    prefixes
}

fn main() -> io::Result<()> {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "localhost".to_string());

    let home = env::var("HOME").unwrap_or_default();
    let cwd = env::current_dir()?;

    let prefixes = get_prefixes(&cwd);
    let pattern = build_pattern(&prefixes);
    let re = Regex::new(&pattern).expect("Invalid regex");

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let result = process_line(&line, &re, &hostname, &home, &cwd);
        writeln!(stdout, "{}", result)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_regex() -> Regex {
        let prefixes = vec![
            regex::escape("/tmp"),
            regex::escape("/home"),
            regex::escape("src"),
            regex::escape("~"),
        ];
        Regex::new(&build_pattern(&prefixes)).unwrap()
    }

    #[test]
    fn test_make_hyperlink() {
        let result = make_hyperlink("file://host/path", "text");
        assert_eq!(result, "\x1b]8;;file://host/path\x07text\x1b]8;;\x07");
    }

    #[test]
    fn test_absolute_path() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        let result = process_line("/tmp/test.txt", &re, "host", "/home/user", &cwd);
        assert!(result.contains("file://host/tmp/test.txt"));
        assert!(result.contains("\x1b]8;;"));
    }

    #[test]
    fn test_relative_path() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        let result = process_line("src/main.rs", &re, "host", "/home/user", &cwd);
        assert!(result.contains("file://host/work/src/main.rs"));
    }

    #[test]
    fn test_home_expansion() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        let result = process_line("~/documents/file.txt", &re, "host", "/home/user", &cwd);
        assert!(result.contains("file://host/home/user/documents/file.txt"));
    }

    #[test]
    fn test_preserves_ansi_colors() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        // Simulates: \x1b[31mmodified: src/main.rs\x1b[m
        let input = "\x1b[31mmodified: src/main.rs\x1b[m";
        let result = process_line(input, &re, "host", "/home/user", &cwd);

        // Should preserve color codes
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("\x1b[m"));
        // Should add hyperlink
        assert!(result.contains("\x1b]8;;"));
        // Color reset should NOT be part of the path
        assert!(!result.contains("main.rs\x1b[m\x07"));
    }

    #[test]
    fn test_no_path_unchanged() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        let input = "just some text without paths";
        let result = process_line(input, &re, "host", "/home/user", &cwd);
        assert_eq!(result, input);
    }

    #[test]
    fn test_multiple_paths() {
        let re = test_regex();
        let cwd = PathBuf::from("/work");
        let input = "comparing /tmp/a.txt and /tmp/b.txt";
        let result = process_line(input, &re, "host", "/home/user", &cwd);
        // Should have two hyperlinks
        assert_eq!(result.matches("\x1b]8;;file://").count(), 2);
    }
}
