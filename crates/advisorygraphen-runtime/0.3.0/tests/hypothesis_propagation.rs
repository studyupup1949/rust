use advisorygraphen_projection::OutputFormat;
use advisorygraphen_runtime::{
    case_import_workflow, case_reason_workflow, check_workflow,
    completions_apply_accepted_workflow, completions_dry_run_workflow,
    completions_propose_workflow, hypothesis_apply_proposals_workflow, hypothesis_falsify_workflow,
    hypothesis_propose_workflow, hypothesis_support_workflow, lift_workflow,
    observation_record_workflow, project_workflow, review_workflow, CaseImportOptions,
    CaseReasonOptions, CheckOptions, CompletionApplyAcceptedOptions, CompletionDryRunOptions,
    CompletionProposeOptions, HypothesisApplyProposalsOptions, HypothesisFalsifyOptions,
    HypothesisProposeOptions, LiftOptions, ObservationRecordOptions, ProjectOptions, ReviewOptions,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/technical-advisory/direct-db-access")
        .join(path)
}

#[path = "hypothesis_propagation/completion_materialization.rs"]
mod completion_materialization;
#[path = "hypothesis_propagation/falsification.rs"]
mod falsification;
#[path = "hypothesis_propagation/observation_support.rs"]
mod observation_support;
#[path = "hypothesis_propagation/policy_apply.rs"]
mod policy_apply;
#[path = "hypothesis_propagation/refinement_lineage.rs"]
mod refinement_lineage;
#[path = "hypothesis_propagation/seed_trace.rs"]
mod seed_trace;
