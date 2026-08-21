use crate::swarm::types::Issue;

/// Generate a signature for an issue (used to cache solutions)
pub fn issue_signature(issue: &Issue) -> String {
    // Domain + first 3 words of title
    format!(
        "{}:{}",
        issue.domain.to_string().to_lowercase(),
        issue.title
            .to_lowercase()
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join("_")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let issue = Issue {
            id: "test".to_string(),
            domain: crate::swarm::types::Domain::Repository,
            agent_name: "test".to_string(),
            title: "Uncommitted changes accumulating".to_string(),
            description: "test".to_string(),
            severity: crate::swarm::types::Severity::Low,
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: Default::default(),
            signature: "test".to_string(),
            stage: crate::swarm::types::Stage::Detected,
        };

        let sig = issue_signature(&issue);
        assert!(sig.contains("repository"));
        assert!(sig.contains("uncommitted"));
    }
}
