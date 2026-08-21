use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_SANDBOX: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
enum SnapshotEntry {
    Directory,
    File(Vec<u8>),
    Symlink,
    Other,
}

pub(super) struct IsolatedCommand {
    _sandbox: tempfile::TempDir,
    state_root: PathBuf,
    fallback_roots: Vec<PathBuf>,
    canary_ids: Vec<String>,
    before: BTreeMap<PathBuf, SnapshotEntry>,
    command: Command,
    verified: bool,
}

pub(super) struct HarnessError {
    failures: Vec<String>,
}

impl HarnessError {
    pub(super) fn has_failure(&self, class: &str) -> bool {
        self.failures
            .iter()
            .any(|failure| failure.starts_with(class))
    }
}

impl fmt::Debug for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HarnessError")
            .field("failures", &self.failures)
            .finish()
    }
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "isolated command failed: {}",
            self.failures.join(", ")
        )
    }
}

impl std::error::Error for HarnessError {}

pub(super) fn aaai() -> IsolatedCommand {
    IsolatedCommand::new(env!("CARGO_BIN_EXE_aaai"))
}

pub(super) fn aaai_with_program(program: impl AsRef<OsStr>) -> IsolatedCommand {
    IsolatedCommand::new(program)
}

impl IsolatedCommand {
    fn new(program: impl AsRef<OsStr>) -> Self {
        let sandbox = tempfile::tempdir().expect("create CLI sandbox");
        let sequence = NEXT_SANDBOX.fetch_add(1, Ordering::Relaxed);
        let random_suffix = sandbox
            .path()
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("synthetic");
        let state_root = sandbox.path().join("state");
        let fallback = sandbox.path().join("fallback");
        let home = fallback.join("home");
        let profile = fallback.join("profile");
        let xdg = fallback.join("xdg");
        let roaming = fallback.join("roaming");
        let local = fallback.join("local");
        let fallback_roots = vec![
            home.join(".config").join("aaai"),
            home.join("Library")
                .join("Application Support")
                .join("aaai"),
            xdg.join("aaai"),
            roaming.join("aaai"),
            local.join("aaai"),
        ];
        let canary_ids: Vec<_> = (0..fallback_roots.len())
            .map(|slot| {
                format!(
                    "AAAI_CANARY_{}_{}_{}_{}",
                    std::process::id(),
                    sequence,
                    slot,
                    random_suffix
                )
            })
            .collect();

        for (root, canary) in fallback_roots.iter().zip(&canary_ids) {
            seed_fallback_store(root, canary);
        }
        let before = snapshot(sandbox.path(), &fallback_roots)
            .expect("snapshot fallback stores before child execution");

        let mut command = Command::new(program);
        command
            .env("AAAI_TEST_STATE_DIR", &state_root)
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", &xdg)
            .env("USERPROFILE", &profile)
            .env("APPDATA", &roaming)
            .env("LOCALAPPDATA", &local);

        Self {
            _sandbox: sandbox,
            state_root,
            fallback_roots,
            canary_ids,
            before,
            command,
            verified: false,
        }
    }

    pub(super) fn arg(&mut self, argument: impl AsRef<OsStr>) -> &mut Self {
        self.command.arg(argument);
        self
    }

    pub(super) fn args<I, S>(&mut self, arguments: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(arguments);
        self
    }

    pub(super) fn current_dir(&mut self, directory: impl AsRef<Path>) -> &mut Self {
        self.command.current_dir(directory);
        self
    }

    pub(super) fn state_root(&self) -> &Path {
        &self.state_root
    }

    pub(super) fn canary_id(&self) -> &str {
        &self.canary_ids[0]
    }

    pub(super) fn synthetic_path_with_canary(&self) -> PathBuf {
        self.state_root.join(self.canary_id()).join("project")
    }

    pub(super) fn seed_history(&self, count: usize) {
        fs::create_dir_all(&self.state_root).expect("create allowed state root");
        let mut lines = String::new();
        for index in 0..count {
            let record = serde_json::json!({
                "run_at": format!("2026-01-01T00:00:{index:02}Z"),
                "before": format!("/allowed/before-{index}"),
                "after": format!("/allowed/after-{index}"),
                "definition": null,
                "result": if index % 2 == 0 { "PASSED" } else { "FAILED" },
                "total": 1,
                "ok": usize::from(index % 2 == 0),
                "pending": 0,
                "failed": usize::from(index % 2 != 0),
                "error": 0
            });
            lines.push_str(&record.to_string());
            lines.push('\n');
        }
        fs::write(self.state_root.join("history.jsonl"), lines).expect("seed allowed history");
    }

    pub(super) fn history_records(&self) -> Vec<serde_json::Value> {
        let contents = fs::read_to_string(self.state_root.join("history.jsonl"))
            .expect("read allowed history");
        contents
            .lines()
            .map(|line| serde_json::from_str(line).expect("parse allowed history record"))
            .collect()
    }

    pub(super) fn mutate_fallback_for_test(&self) {
        fs::write(
            self.fallback_roots[0].join("sentinel"),
            "synthetic mutation",
        )
        .expect("mutate synthetic fallback sentinel");
    }

    pub(super) fn run_status(&mut self) -> Result<ExitStatus, HarnessError> {
        self.execute().map(|output| output.status)
    }

    pub(super) fn run_output(&mut self) -> Result<Output, HarnessError> {
        self.execute()
    }

    fn execute(&mut self) -> Result<Output, HarnessError> {
        let output = self.command.output();
        let mut failures = Vec::new();

        match &output {
            Ok(output) => {
                scan_stream(
                    "stdout-disclosure",
                    &output.stdout,
                    &self.canary_ids,
                    &mut failures,
                );
                scan_stream(
                    "stderr-disclosure",
                    &output.stderr,
                    &self.canary_ids,
                    &mut failures,
                );
            }
            Err(_) => failures.push("spawn-failure".into()),
        }

        match snapshot(self._sandbox.path(), &self.fallback_roots) {
            Ok(after) if after != self.before => {
                for path in changed_paths(&self.before, &after) {
                    failures.push(format!("fallback-mutation:{}", path.display()));
                }
            }
            Ok(_) => {}
            Err(_) => failures.push("snapshot-failure".into()),
        }
        self.verified = true;

        if failures.is_empty() {
            Ok(output.expect("successful spawn when no harness failures were recorded"))
        } else {
            Err(HarnessError { failures })
        }
    }
}

impl Drop for IsolatedCommand {
    fn drop(&mut self) {
        if self.verified || std::thread::panicking() {
            return;
        }
        let unchanged = snapshot(self._sandbox.path(), &self.fallback_roots)
            .map(|after| after == self.before)
            .unwrap_or(false);
        assert!(
            unchanged,
            "unconsumed isolated command detected fallback mutation"
        );
    }
}

fn seed_fallback_store(root: &Path, canary: &str) {
    fs::create_dir_all(root).expect("create fallback store");
    let history = serde_json::json!({
        "run_at": "2026-01-01T00:00:00Z",
        "before": canary,
        "after": "/synthetic/after",
        "definition": null,
        "result": "PASSED",
        "total": 0,
        "ok": 0,
        "pending": 0,
        "failed": 0,
        "error": 0
    });
    fs::write(root.join("history.jsonl"), format!("{history}\n")).expect("seed fallback history");
    fs::write(
        root.join("profiles.yaml"),
        format!(
            "profiles:\n  - name: {canary}\n    before: /synthetic/before\n    after: /synthetic/after\n"
        ),
    )
    .expect("seed fallback profiles");
    fs::write(
        root.join("prefs.yaml"),
        format!("theme: light\nlanguage: {canary}\nglobal_ignored_dirs: []\n"),
    )
    .expect("seed fallback preferences");
    fs::write(root.join("sentinel"), canary).expect("seed fallback sentinel");
}

fn scan_stream(class: &str, stream: &[u8], canaries: &[String], failures: &mut Vec<String>) {
    for canary in canaries {
        if stream
            .windows(canary.len())
            .any(|window| window == canary.as_bytes())
        {
            failures.push(format!("{class}:{canary}"));
        }
    }
}

fn snapshot(
    sandbox: &Path,
    roots: &[PathBuf],
) -> std::io::Result<BTreeMap<PathBuf, SnapshotEntry>> {
    let mut entries = BTreeMap::new();
    for root in roots {
        snapshot_path(sandbox, root, &mut entries)?;
    }
    Ok(entries)
}

fn snapshot_path(
    sandbox: &Path,
    path: &Path,
    entries: &mut BTreeMap<PathBuf, SnapshotEntry>,
) -> std::io::Result<()> {
    let relative = path
        .strip_prefix(sandbox)
        .expect("fallback root remains inside its sandbox")
        .to_path_buf();
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.is_dir() {
        entries.insert(relative, SnapshotEntry::Directory);
        let mut children: Vec<_> = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<_, _>>()?;
        children.sort();
        for child in children {
            snapshot_path(sandbox, &child, entries)?;
        }
    } else if metadata.is_file() {
        entries.insert(relative, SnapshotEntry::File(fs::read(path)?));
    } else if metadata.file_type().is_symlink() {
        entries.insert(relative, SnapshotEntry::Symlink);
    } else {
        entries.insert(relative, SnapshotEntry::Other);
    }
    Ok(())
}

fn changed_paths(
    before: &BTreeMap<PathBuf, SnapshotEntry>,
    after: &BTreeMap<PathBuf, SnapshotEntry>,
) -> Vec<PathBuf> {
    let mut paths: Vec<_> = before
        .keys()
        .chain(after.keys())
        .filter(|path| before.get(*path) != after.get(*path))
        .cloned()
        .collect();
    paths.sort();
    paths.dedup();
    paths
}
