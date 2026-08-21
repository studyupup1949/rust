use crate::swarm::types::*;
use chrono::Utc;

pub fn format_status(statuses: &[AgentStatus]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}\n",
        "━".repeat(60)
    ));
    output.push_str("  AUTONOMOUS AGENT SYSTEM - STATUS\n");
    output.push_str(&format!(
        "{}\n\n",
        "━".repeat(60)
    ));

    let all_running = statuses.iter().all(|s| s.running);
    let has_issues = statuses.iter().any(|s| !s.healthy);

    let status_icon = if has_issues { "🟡" } else if all_running { "🟢" } else { "🔴" };
    output.push_str(&format!("  System Status: {} {}\n", status_icon,
        if has_issues { "DEGRADED" } else if all_running { "HEALTHY" } else { "DOWN" }));
    output.push('\n');

    for status in statuses {
        let icon = if !status.running {
            "⬜"
        } else if !status.healthy {
            "🟡"
        } else {
            "🟢"
        };

        let last_check = status
            .last_check
            .map(|t| {
                let ago = (Utc::now() - t).num_seconds();
                if ago < 60 {
                    format!("{}s ago", ago)
                } else {
                    format!("{}m ago", ago / 60)
                }
            })
            .unwrap_or_else(|| "never".to_string());

        output.push_str(&format!("  {} {} AGENT\n", icon, status.name.to_uppercase()));
        output.push_str(&format!("     Status: {}\n", if status.running { "RUNNING" } else { "STOPPED" }));
        output.push_str(&format!("     Issues Detected: {}\n", status.issues_detected));
        output.push_str(&format!("     Actions Taken: {}\n", status.actions_taken));
        output.push_str(&format!("     Success Rate: {:.0}%\n", status.success_rate));
        output.push_str(&format!("     Last Check: {}\n", last_check));

        if let Some(ref issue) = status.current_issue {
            output.push_str(&format!("     Current Issue: {}\n", issue));
        }

        output.push('\n');
    }

    output
}

pub fn format_history(decisions: &[Decision]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}\n",
        "━".repeat(60)
    ));
    output.push_str("  DECISION HISTORY\n");
    output.push_str(&format!(
        "{}\n\n",
        "━".repeat(60)
    ));

    for decision in decisions {
        let status_icon = match decision.status {
            DecisionStatus::Completed => "✅",
            DecisionStatus::Failed => "❌",
            DecisionStatus::RolledBack => "↩️",
            DecisionStatus::InProgress => "🔄",
            DecisionStatus::AwaitingApproval => "⏳",
            DecisionStatus::Rejected => "🚫",
            DecisionStatus::Detected => "🔍",
            DecisionStatus::Analyzing => "📊",
        };

        let ago = (Utc::now() - decision.created_at).num_minutes();
        let time_str = if ago < 1 {
            "just now".to_string()
        } else if ago < 60 {
            format!("{}m ago", ago)
        } else {
            format!("{}h ago", ago / 60)
        };

        output.push_str(&format!(
            "  {} {} [{}]\n",
            status_icon, decision.issue.title, time_str
        ));
        output.push_str(&format!(
            "     Agent: {} | Status: {}\n",
            decision.issue.agent_name, decision.status
        ));
        output.push_str(&format!(
            "     Severity: {} | ID: {}\n\n",
            decision.issue.severity, decision.id
        ));
    }

    output
}

pub fn format_patterns(patterns: &[Pattern]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}\n",
        "━".repeat(60)
    ));
    output.push_str("  LEARNED PATTERNS\n");
    output.push_str(&format!(
        "{}\n\n",
        "━".repeat(60)
    ));

    for pattern in patterns {
        output.push_str(&format!(
            "  Pattern: {}\n",
            pattern.name
        ));
        output.push_str(&format!(
            "     Confidence: {:.0}%\n",
            pattern.confidence * 100.0
        ));
        output.push_str(&format!(
            "     Occurrences: {}\n",
            pattern.occurrences
        ));
        output.push_str(&format!(
            "     Domain: {}\n",
            pattern.domain
        ));
        output.push_str(&format!(
            "     Solution: {}\n\n",
            pattern.solution_description
        ));
    }

    output
}

pub fn format_predictions(predictions: &[Prediction]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}\n",
        "━".repeat(60)
    ));
    output.push_str("  ACTIVE PREDICTIONS\n");
    output.push_str(&format!(
        "{}\n\n",
        "━".repeat(60)
    ));

    for pred in predictions {
        let icon = if pred.confidence > 0.8 { "🔴" } else if pred.confidence > 0.6 { "🟡" } else { "🔵" };
        output.push_str(&format!(
            "  {} {}\n",
            icon, pred.predicted_issue
        ));
        output.push_str(&format!(
            "     Confidence: {:.0}%\n",
            pred.confidence * 100.0
        ));
        output.push_str(&format!(
            "     Expected in: {}\n",
            pred.time_until_expected
        ));
        output.push_str(&format!(
            "     Action: {}\n\n",
            pred.suggested_action
        ));
    }

    output
}

pub fn format_memory_stats(issues: u64, actions: u64, patterns: usize) -> String {
    format!(
        "Memory Statistics:\n\
         ─────────────────────\n\
         Issues Stored:    {}\n\
         Actions Taken:    {}\n\
         Patterns Learned: {}\n\
         ─────────────────────\n",
        issues, actions, patterns
    )
}
