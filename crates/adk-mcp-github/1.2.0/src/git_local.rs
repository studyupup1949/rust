use std::process::Command;

pub struct GitLocal;

impl GitLocal {
    fn run(args: &[&str], cwd: Option<&str>) -> Result<String, String> {
        let mut cmd = Command::new("git");
        cmd.args(args);
        if let Some(dir) = cwd { cmd.current_dir(dir); }
        let output = cmd.output().map_err(|e| format!("Failed to run git: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if err.is_empty() { "git command failed".to_string() } else { err })
        }
    }

    pub fn clone(url: &str, path: Option<&str>) -> Result<String, String> {
        let mut args = vec!["clone", url];
        if let Some(p) = path { args.push(p); }
        Self::run(&args, None)
    }

    pub fn init(path: &str) -> Result<String, String> {
        Self::run(&["init", path], None)
    }

    pub fn status(cwd: &str) -> Result<String, String> {
        Self::run(&["status", "--short"], Some(cwd))
    }

    pub fn add(cwd: &str, paths: &[&str]) -> Result<String, String> {
        let mut args = vec!["add"];
        args.extend(paths);
        Self::run(&args, Some(cwd))
    }

    pub fn commit(cwd: &str, message: &str) -> Result<String, String> {
        Self::run(&["commit", "-m", message], Some(cwd))
    }

    pub fn push(cwd: &str, remote: Option<&str>, branch: Option<&str>) -> Result<String, String> {
        let mut args = vec!["push"];
        if let Some(r) = remote { args.push(r); }
        if let Some(b) = branch { args.push(b); }
        Self::run(&args, Some(cwd))
    }

    pub fn tag(cwd: &str, name: &str, message: Option<&str>) -> Result<String, String> {
        if let Some(m) = message {
            Self::run(&["tag", "-a", name, "-m", m], Some(cwd))
        } else {
            Self::run(&["tag", name], Some(cwd))
        }
    }

    pub fn log(cwd: &str, count: u32) -> Result<String, String> {
        Self::run(&["log", "--oneline", &format!("-{}", count)], Some(cwd))
    }

    pub fn remote_add(cwd: &str, name: &str, url: &str) -> Result<String, String> {
        Self::run(&["remote", "add", name, url], Some(cwd))
    }

    pub fn branch_current(cwd: &str) -> Result<String, String> {
        Self::run(&["branch", "--show-current"], Some(cwd))
    }
}
