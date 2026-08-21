use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMark {
    pub name: String,
    pub when: SystemTime,
    pub fields: Vec<(String, String)>,
}

impl TraceMark {
    pub fn render_line(&self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(" ");
        if fields.is_empty() {
            self.name.clone()
        } else {
            format!("{} {}", self.name, fields)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceBook {
    marks: Vec<TraceMark>,
}

impl TraceBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark(&mut self, name: impl Into<String>) {
        self.marks.push(TraceMark {
            name: name.into(),
            when: SystemTime::now(),
            fields: Vec::new(),
        });
    }

    pub fn mark_with_fields(
        &mut self,
        name: impl Into<String>,
        fields: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) {
        self.marks.push(TraceMark {
            name: name.into(),
            when: SystemTime::now(),
            fields: fields
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        });
    }

    pub fn marks(&self) -> &[TraceMark] {
        &self.marks
    }

    pub fn clear(&mut self) {
        self.marks.clear();
    }

    pub fn render_lines(&self) -> Vec<String> {
        self.marks.iter().map(TraceMark::render_line).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceExportPlan {
    pub destination: String,
    pub enabled: bool,
    pub blocked_reason: Option<String>,
    pub mark_count: usize,
}

impl TraceExportPlan {
    pub fn from_guard(destination: impl Into<String>, book: &TraceBook, allow_paths: bool) -> Self {
        let destination = destination.into();
        if allow_paths {
            Self {
                destination,
                enabled: true,
                blocked_reason: None,
                mark_count: book.marks.len(),
            }
        } else {
            Self {
                destination,
                enabled: false,
                blocked_reason: Some("trace export is disabled by safety config".to_string()),
                mark_count: book.marks.len(),
            }
        }
    }

    pub fn render_local(&self, book: &TraceBook) -> TraceExportArtifact {
        TraceExportArtifact {
            destination: self.destination.clone(),
            enabled: self.enabled,
            mark_count: self.mark_count,
            lines: book.render_lines(),
            blocked_reason: self.blocked_reason.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceExportArtifact {
    pub destination: String,
    pub enabled: bool,
    pub mark_count: usize,
    pub lines: Vec<String>,
    pub blocked_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_book_records_local_marks_only() {
        let mut book = TraceBook::new();
        book.mark_with_fields("sync", [("peer", "a")]);
        assert_eq!(book.marks().len(), 1);
        assert_eq!(book.marks()[0].name, "sync");
        assert_eq!(
            book.marks()[0].fields,
            vec![("peer".to_string(), "a".to_string())]
        );
    }

    #[test]
    fn trace_export_plan_is_closed_by_default() {
        let mut book = TraceBook::new();
        book.mark("chain");
        let plan = TraceExportPlan::from_guard("local-trace", &book, false);
        assert!(!plan.enabled);
        assert_eq!(plan.mark_count, 1);
        assert!(plan.blocked_reason.is_some());
    }

    #[test]
    fn trace_export_renders_local_artifact_without_writing() {
        let mut book = TraceBook::new();
        book.mark_with_fields("sync", [("peer", "a"), ("slot", "10")]);
        let plan = TraceExportPlan::from_guard("local-trace", &book, false);

        let artifact = plan.render_local(&book);
        assert!(!artifact.enabled);
        assert_eq!(artifact.mark_count, 1);
        assert_eq!(artifact.lines, vec!["sync peer=a slot=10"]);
        assert!(artifact.blocked_reason.is_some());
    }
}
