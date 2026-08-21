use advisorygraphen_core::{AdvisoryError, AdvisoryResult, AdvisorySpaceEnvelope};
use advisorygraphen_reasoning::{
    blocker_resolution_state, close_status, frontier_items, waiting_items,
};
use serde_json::{json, Value};

mod correspondence;
mod higher;
mod hypotheses;

use hypotheses::{argumentation_incidences, falsifiers, hypotheses, hypothesis_summary};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Json,
    Markdown,
}

impl OutputFormat {
    pub fn parse(value: &str) -> AdvisoryResult<Self> {
        match value {
            "json" => Ok(Self::Json),
            "markdown" => Ok(Self::Markdown),
            other => Err(AdvisoryError::Validation(format!(
                "unsupported format: {other}"
            ))),
        }
    }
}

pub fn project(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
    format: OutputFormat,
) -> AdvisoryResult<String> {
    let projection = build_projection(space, report, audience)?;
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&projection)?),
        OutputFormat::Markdown => render_markdown(audience, &projection),
    }
}

pub fn build_projection(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
) -> AdvisoryResult<Value> {
    match audience {
        "executive" => executive_projection(space, report, audience),
        "developer_action" => developer_projection(space, report, audience),
        "audit_trace" => audit_projection(space, report, audience),
        "ai_agent" => ai_agent_projection(space, report, audience),
        "client_review" | "cli" => executive_projection(space, report, audience),
        other => Err(AdvisoryError::UnsupportedAudience(other.to_string())),
    }
}

#[path = "lib/audiences.rs"]
mod audiences;
#[path = "lib/explicit_hypotheses.rs"]
mod explicit_hypotheses;
#[path = "lib/markdown.rs"]
mod markdown;
#[path = "lib/observation_templates.rs"]
mod observation_templates;
#[path = "lib/promotion.rs"]
mod promotion;
#[path = "lib/recommendations.rs"]
mod recommendations;
#[path = "lib/summaries.rs"]
mod summaries;

use audiences::*;
use explicit_hypotheses::*;
use markdown::*;
use observation_templates::*;
use promotion::*;
use recommendations::*;
use summaries::*;
