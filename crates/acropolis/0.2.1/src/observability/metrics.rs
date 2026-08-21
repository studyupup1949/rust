use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricsBook {
    counters: BTreeMap<String, u64>,
    gauges: BTreeMap<String, i64>,
}

impl MetricsBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, name: impl Into<String>, by: u64) {
        let entry = self.counters.entry(name.into()).or_insert(0);
        *entry = entry.saturating_add(by);
    }

    pub fn set_gauge(&mut self, name: impl Into<String>, value: i64) {
        self.gauges.insert(name.into(), value);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counters: self.counters.clone(),
            gauges: self.gauges.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub counters: BTreeMap<String, u64>,
    pub gauges: BTreeMap<String, i64>,
}

impl MetricsSnapshot {
    pub fn render_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for (name, value) in &self.counters {
            lines.push(format!("counter.{name}={value}"));
        }
        for (name, value) in &self.gauges {
            lines.push(format!("gauge.{name}={value}"));
        }
        lines
    }

    pub fn render_text(&self) -> String {
        let mut text = self.render_lines().join("\n");
        if !text.is_empty() {
            text.push('\n');
        }
        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsExportPlan {
    pub endpoint: String,
    pub enabled: bool,
    pub blocked_reason: Option<String>,
    pub lines: Vec<String>,
}

impl MetricsExportPlan {
    pub fn closed(endpoint: impl Into<String>, snapshot: &MetricsSnapshot) -> Self {
        Self {
            endpoint: endpoint.into(),
            enabled: false,
            blocked_reason: Some("metrics export is disabled by safety config".to_string()),
            lines: snapshot.render_lines(),
        }
    }

    pub fn from_guard(
        endpoint: impl Into<String>,
        snapshot: &MetricsSnapshot,
        allow_paths: bool,
    ) -> Self {
        let endpoint = endpoint.into();
        if allow_paths {
            Self {
                endpoint,
                enabled: true,
                blocked_reason: None,
                lines: snapshot.render_lines(),
            }
        } else {
            Self::closed(endpoint, snapshot)
        }
    }

    pub fn render_local(&self) -> MetricsExportArtifact {
        MetricsExportArtifact {
            endpoint: self.endpoint.clone(),
            enabled: self.enabled,
            content_type: "text/plain; version=0.0.4".to_string(),
            body: if self.lines.is_empty() {
                String::new()
            } else {
                format!("{}\n", self.lines.join("\n"))
            },
            blocked_reason: self.blocked_reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsExportArtifact {
    pub endpoint: String,
    pub enabled: bool,
    pub content_type: String,
    pub body: String,
    pub blocked_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_book_snapshots_counters_and_gauges() {
        let mut book = MetricsBook::new();
        book.increment("chain", 1);
        book.increment("chain", 2);
        book.set_gauge("peers", 4);
        let snapshot = book.snapshot();
        assert_eq!(snapshot.counters["chain"], 3);
        assert_eq!(snapshot.gauges["peers"], 4);
        assert_eq!(
            snapshot.render_lines(),
            vec!["counter.chain=3", "gauge.peers=4"]
        );
    }

    #[test]
    fn metrics_export_plan_is_closed_by_default() {
        let mut book = MetricsBook::new();
        book.increment("chain", 1);
        let snapshot = book.snapshot();
        let plan = MetricsExportPlan::from_guard("127.0.0.1:12798", &snapshot, false);
        assert!(!plan.enabled);
        assert_eq!(plan.lines, vec!["counter.chain=1"]);
        assert!(plan.blocked_reason.is_some());
    }

    #[test]
    fn metrics_export_renders_local_artifact_without_serving() {
        let mut book = MetricsBook::new();
        book.increment("chain", 3);
        book.set_gauge("peers", 4);
        let snapshot = book.snapshot();
        let plan = MetricsExportPlan::from_guard("127.0.0.1:12798", &snapshot, false);

        let artifact = plan.render_local();
        assert!(!artifact.enabled);
        assert_eq!(artifact.content_type, "text/plain; version=0.0.4");
        assert_eq!(artifact.body, "counter.chain=3\ngauge.peers=4\n");
        assert!(artifact.blocked_reason.is_some());
    }
}
