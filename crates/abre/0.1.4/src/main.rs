mod shorten;

use clap::Parser;
use regex::Regex;
use serde_json::Value;
use std::io::{self, BufRead, Write};

use shorten::Strategy;

#[derive(Parser)]
#[command(name = "abre", about = "Shorten repetitive text for display")]
struct Args {
    /// Capture group regex — group 1 selects the part to shorten
    #[arg(short = 'c')]
    capture: Option<String>,

    /// Built-in capture preset (url-path, url-full, url-domain, docker)
    #[arg(short = 'p')]
    preset: Option<String>,

    /// Segment separator
    #[arg(short = 's', default_value = "/")]
    separator: String,

    /// Use shortest-unique-suffix strategy
    #[arg(long)]
    suffix: bool,

    /// Use truncate strategy (shorten shared segments to N chars)
    #[arg(long)]
    truncate: bool,

    /// Characters to keep per segment in truncate mode
    #[arg(short = 'n', default_value_t = 1)]
    truncate_n: usize,

    /// Replacement string for collapsed segments
    #[arg(long, default_value = "…")]
    ellipsis: String,

    /// JSON line mode — operate on a key
    #[arg(long)]
    json: bool,

    /// JSON key to shorten
    #[arg(short = 'k')]
    key: Option<String>,

    /// Write shortened value to a new key (keeps original)
    #[arg(long)]
    add_key: Option<String>,

    /// Modify key in place, save original to this key
    #[arg(long)]
    keep_original: Option<String>,
}

fn resolve_capture(args: &Args) -> Option<Regex> {
    let pattern = if let Some(ref p) = args.preset {
        Some(match p.as_str() {
            "url-path" => r"https?://(?:www\.)?[^/]+(.*)".to_string(),
            "url-full" => r"https?://(?:www\.)?(.*)".to_string(),
            "url-domain" => r"https?://(?:www\.)?([^/]+)".to_string(),
            "docker" => r"([^:]+):.+".to_string(),
            other => {
                eprintln!("unknown preset: {other}");
                std::process::exit(1);
            }
        })
    } else {
        args.capture.clone()
    };

    pattern.map(|p| {
        Regex::new(&p).unwrap_or_else(|e| {
            eprintln!("invalid regex: {e}");
            std::process::exit(1);
        })
    })
}

/// Extract (prefix, target, suffix) from a line using the capture regex.
/// prefix = text before the full match (group 0), so non-captured anchoring
/// parts of the regex (like `https?://`) are stripped from output.
/// If no regex or no match, target = full line and prefix/suffix are empty.
fn extract<'a>(line: &'a str, re: Option<&Regex>) -> (&'a str, &'a str, &'a str) {
    if let Some(re) = re {
        if let Some(caps) = re.captures(line) {
            if let Some(m) = caps.get(1) {
                let full = caps.get(0).unwrap();
                return (&line[..full.start()], m.as_str(), &line[m.end()..]);
            }
        }
    }
    ("", line, "")
}

fn main() {
    let args = Args::parse();
    let capture_re = resolve_capture(&args);

    let strategy = if args.suffix {
        Strategy::Suffix
    } else if args.truncate {
        Strategy::Truncate(args.truncate_n)
    } else {
        Strategy::Collapse
    };

    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().map(|l| l.expect("read error")).collect();

    if lines.is_empty() {
        return;
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    if args.json {
        process_json(&lines, &args, &capture_re, &strategy, &mut out);
    } else {
        process_plain(&lines, &capture_re, &strategy, &args.separator, &args.ellipsis, &mut out);
    }
}

fn process_plain(
    lines: &[String],
    capture_re: &Option<Regex>,
    strategy: &Strategy,
    sep: &str,
    ellipsis: &str,
    out: &mut impl Write,
) {
    // Extract targets
    let parts: Vec<(&str, &str, &str)> = lines
        .iter()
        .map(|l| extract(l, capture_re.as_ref()))
        .collect();

    let targets: Vec<&str> = parts.iter().map(|(_, t, _)| *t).collect();

    // Split into segments
    let segments: Vec<Vec<&str>> = targets.iter().map(|t| t.split(sep).collect()).collect();

    // Shorten
    let shortened = shorten::shorten(&segments, sep, ellipsis, strategy);

    // Reassemble and output
    for (i, (prefix, _, suffix)) in parts.iter().enumerate() {
        let _ = writeln!(out, "{}{}{}", prefix, shortened[i], suffix);
    }
}

fn process_json(
    lines: &[String],
    args: &Args,
    capture_re: &Option<Regex>,
    strategy: &Strategy,
    out: &mut impl Write,
) {
    let key = args.key.as_deref().unwrap_or_else(|| {
        eprintln!("--json requires -k <key>");
        std::process::exit(1);
    });

    // Parse JSON lines and extract values
    let mut objects: Vec<Value> = lines
        .iter()
        .map(|l| {
            serde_json::from_str(l).unwrap_or_else(|e| {
                eprintln!("invalid JSON: {e}");
                std::process::exit(1);
            })
        })
        .collect();

    // Extract the target strings from JSON objects
    let values: Vec<String> = objects
        .iter()
        .map(|obj| {
            obj.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    eprintln!("key '{key}' not found or not a string");
                    std::process::exit(1);
                })
                .to_string()
        })
        .collect();

    // Extract capture parts
    let parts: Vec<(&str, &str, &str)> = values
        .iter()
        .map(|v| extract(v, capture_re.as_ref()))
        .collect();

    let targets: Vec<&str> = parts.iter().map(|(_, t, _)| *t).collect();
    let segments: Vec<Vec<&str>> = targets
        .iter()
        .map(|t| t.split(&*args.separator).collect())
        .collect();

    let shortened = shorten::shorten(&segments, &args.separator, &args.ellipsis, strategy);

    // Reassemble shortened values
    let shortened_full: Vec<String> = parts
        .iter()
        .zip(shortened.iter())
        .map(|((prefix, _, suffix), short)| format!("{}{}{}", prefix, short, suffix))
        .collect();

    // Apply to JSON objects
    for (i, obj) in objects.iter_mut().enumerate() {
        let map = obj.as_object_mut().unwrap();

        if let Some(ref add_key) = args.add_key {
            // Keep original, add shortened as new key
            map.insert(add_key.clone(), Value::String(shortened_full[i].clone()));
        } else if let Some(ref keep_key) = args.keep_original {
            // Save original, replace with shortened
            map.insert(keep_key.clone(), Value::String(values[i].clone()));
            map.insert(key.to_string(), Value::String(shortened_full[i].clone()));
        } else {
            // Modify in place
            map.insert(key.to_string(), Value::String(shortened_full[i].clone()));
        }

        let _ = writeln!(out, "{}", serde_json::to_string(obj).unwrap());
    }
}
