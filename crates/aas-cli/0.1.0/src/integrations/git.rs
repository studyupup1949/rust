use std::path::Path;
use tokio::process::Command;

pub struct GitOps;

impl GitOps {
    pub async fn status_summary(path: &Path) -> Result<(usize, usize, usize), String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "status", "--porcelain"])
            .output()
            .await
            .map_err(|e| format!("git status failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

        let modified = lines.len();
        let staged = lines.iter().filter(|l| {
            l.starts_with('M') || l.starts_with('A') || l.starts_with('D')
        }).count();
        let untracked = lines.iter().filter(|l| l.starts_with("??")).count();

        Ok((modified, staged, untracked))
    }

    pub async fn create_commit(path: &Path, message: &str) -> Result<String, String> {
        Command::new("git")
            .args(["-C", &path.to_string_lossy(), "add", "-A"])
            .output()
            .await
            .map_err(|e| format!("git add failed: {}", e))?;

        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "commit", "-m", message])
            .output()
            .await
            .map_err(|e| format!("git commit failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Commit failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    pub async fn get_diff(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "diff", "--stat"])
            .output()
            .await
            .map_err(|e| format!("git diff failed: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub async fn get_log(path: &Path, count: usize) -> Result<Vec<String>, String> {
        let output = Command::new("git")
            .args([
                "-C",
                &path.to_string_lossy(),
                "log",
                &format!("-{}", count),
                "--oneline",
            ])
            .output()
            .await
            .map_err(|e| format!("git log failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }

    pub async fn get_current_branch(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .await
            .map_err(|e| format!("git branch failed: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub async fn rollback_commit(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "revert", "--no-edit", "HEAD"])
            .output()
            .await
            .map_err(|e| format!("git revert failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Revert failed: {}", stderr));
        }

        Ok("Reverted last commit".to_string())
    }
}
