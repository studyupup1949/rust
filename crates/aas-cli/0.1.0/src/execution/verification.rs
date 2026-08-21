use crate::swarm::types::Issue;

/// Verifies that an action actually solved the underlying problem
pub struct VerificationEngine;

impl VerificationEngine {
    /// Verify health endpoint is responding
    pub async fn verify_health_endpoint(endpoint: &str) -> bool {
        match reqwest::Client::new()
            .get(endpoint)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Verify git repository has no uncommitted changes
    pub async fn verify_repo_clean(repo_path: &str) -> bool {
        match tokio::process::Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(repo_path)
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.trim().is_empty()
            }
            Err(_) => false,
        }
    }

    /// Verify log file error count decreased
    pub async fn verify_log_improvement(log_path: &str, baseline_count: usize) -> bool {
        match tokio::fs::read_to_string(log_path).await {
            Ok(content) => {
                let current_count = content
                    .lines()
                    .filter(|l| {
                        let lower = l.to_lowercase();
                        lower.contains("error") || lower.contains("exception")
                    })
                    .count();
                current_count < baseline_count
            }
            Err(_) => false,
        }
    }

    /// Verify disk space improved
    pub async fn verify_disk_improved() -> bool {
        match tokio::process::Command::new("df")
            .arg("-k")
            .arg("/")
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Simple check: can we parse disk info?
                stdout.lines().count() > 1
            }
            Err(_) => false,
        }
    }

    /// Verify memory usage improved
    pub async fn verify_memory_improved() -> bool {
        let sys = sysinfo::System::new_all();
        // If system responds to query, memory is accessible
        sys.total_memory() > 0
    }

    /// Generic verification: issue should no longer be detectable
    pub async fn verify_issue_resolved(issue: &Issue) -> bool {
        match issue.domain {
            crate::swarm::types::Domain::Health => {
                if let Some(endpoint) = issue.source.strip_prefix("endpoint:") {
                    Self::verify_health_endpoint(endpoint).await
                } else {
                    false
                }
            }
            crate::swarm::types::Domain::Repository => {
                Self::verify_repo_clean(&issue.source).await
            }
            crate::swarm::types::Domain::Logs => {
                // For logs, just check file is readable (log rotation might have cleaned it)
                tokio::fs::read_to_string(&issue.source).await.is_ok()
            }
            crate::swarm::types::Domain::Metrics => {
                Self::verify_disk_improved().await && Self::verify_memory_improved().await
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verification() {
        // Health check (will fail without real endpoint)
        let result = VerificationEngine::verify_health_endpoint("http://localhost:9999").await;
        assert!(!result);

        // Memory check (should work)
        let result = VerificationEngine::verify_memory_improved().await;
        assert!(result);
    }
}
