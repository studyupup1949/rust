use advisorygraphen_core::{
    json_id, validate_document, AdvisoryError, AdvisoryResult, AdvisorySpaceEnvelope,
    ReportEnvelope, HYPOTHESIS_EVENT_SCHEMA, REVIEW_EVENT_SCHEMA,
};
use advisorygraphen_interpretation::InterpretationPackage;
use advisorygraphen_lift::lift_snapshot;
use advisorygraphen_projection::{build_projection, project};
use advisorygraphen_reasoning::{
    blocker_resolution_state, check_space, close_status, frontier_items, propose_completions,
    propose_hypothesis_lifecycle, waiting_items,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

mod case_review;
mod code_snapshot;
mod dogfood;
mod dry_run_gluing;
mod hypothesis_propagation;
mod hypothesis_review;
mod micro_review;
mod options;
mod projection_report;
mod review;
use case_review::apply_candidate_reviews;
pub use code_snapshot::{code_repo_snapshot_workflow, CodeRepoSnapshotOptions};
pub use dogfood::{
    dogfood_adversarial_fixture_workflow, dogfood_repo_snapshot_workflow,
    DogfoodAdversarialFixtureOptions, DogfoodRepoSnapshotOptions,
};
use hypothesis_propagation::{
    extend_candidates_from_supported_hypotheses, mark_orphaned_candidates, reframe_obstructions,
};
use hypothesis_review::apply_hypothesis_events;
pub use options::{
    CaseCloseCheckOptions, CaseImportOptions, CaseReasonOptions, CheckOptions,
    CompletionApplyAcceptedOptions, CompletionDryRunOptions, CompletionProposeOptions,
    FacadeCompletionReviewOptions, FacadeHypothesisReviewOptions, FacadeProposeOptions,
    FacadeReportOptions, FacadeStatusOptions, HypothesisApplyProposalsOptions,
    HypothesisFalsifyOptions, HypothesisProposeOptions, LiftOptions, MicroReviewOptions,
    ObservationRecordOptions, ProjectOptions, ReviewOptions, ValidateOptions,
};
use projection_report::{attach_completion_report, read_projection_report};
use review::{higher_graphen_completion_review, review_report_path, review_space_id};
use serde::{Deserialize, Serialize};

const CASE_MANIFEST_FILE: &str = "advisorygraphen.case-manifest.json";
const DEFAULT_FACADE_REVISION: &str = "revision:facade-initial";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseArtifacts {
    input: String,
    space: String,
    check_report: String,
    completions_report: String,
    hypothesis_report: String,
    ai_agent_projection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CaseManifest {
    schema: String,
    space_id: String,
    package: String,
    ruleset: String,
    store_path: String,
    artifacts: CaseArtifacts,
    head_revision: String,
    created_at: String,
    updated_at: String,
}

enum Materialization {
    Applied {
        cells: Vec<Value>,
        incidences: Vec<Value>,
    },
    Skipped {
        reason: String,
    },
}

enum DryRunMaterialization {
    Applied {
        cells: Vec<Value>,
        incidences: Vec<Value>,
        removed_incidence_ids: Vec<String>,
    },
    Skipped {
        reason: String,
    },
}

#[path = "lib/apply_accepted.rs"]
mod apply_accepted;
#[path = "lib/basic_workflows.rs"]
mod basic_workflows;
#[path = "lib/case_store.rs"]
mod case_store;
#[path = "lib/dry_run_materialization.rs"]
mod dry_run_materialization;
#[path = "lib/facade.rs"]
mod facade;
#[path = "lib/facade_reviews.rs"]
mod facade_reviews;
#[path = "lib/hypothesis_lifecycle.rs"]
mod hypothesis_lifecycle_workflows;
#[path = "lib/hypothesis_proposals.rs"]
mod hypothesis_proposals;
#[path = "lib/io_store.rs"]
mod io_store;
#[path = "lib/materialization.rs"]
mod materialization;
#[path = "lib/observations.rs"]
mod observations;
#[path = "lib/reviews.rs"]
mod reviews_workflows;

pub use apply_accepted::completions_apply_accepted_workflow;
pub use basic_workflows::{
    check_workflow, completions_dry_run_workflow, completions_propose_workflow, lift_workflow,
    micro_review_workflow, validate_workflow,
};
pub use case_store::{case_close_check_workflow, case_import_workflow, case_reason_workflow};
pub use facade::{facade_propose_workflow, facade_status_workflow, project_workflow};
pub use facade_reviews::{
    facade_completion_review_workflow, facade_hypothesis_review_workflow, facade_report_workflow,
};
pub use hypothesis_lifecycle_workflows::{
    hypothesis_accept_workflow, hypothesis_reject_workflow, hypothesis_support_workflow,
};
pub use hypothesis_proposals::{hypothesis_apply_proposals_workflow, hypothesis_propose_workflow};
pub use io_store::{read_json, write_json_if_requested, write_string_if_requested};
pub use observations::{hypothesis_falsify_workflow, observation_record_workflow};
pub use reviews_workflows::review_workflow;

use case_store::{
    ensure_new_case_dir, path_to_manifest_string, read_case_manifest, sync_manifest_head,
    write_case_manifest,
};
use dry_run_materialization::*;
use hypothesis_lifecycle_workflows::{
    application_skip, autonomy_decision, hypothesis_event_from_proposal,
    hypothesis_lifecycle_event, read_autonomy_policy,
};
use io_store::*;
use materialization::*;
use reviews_workflows::*;
