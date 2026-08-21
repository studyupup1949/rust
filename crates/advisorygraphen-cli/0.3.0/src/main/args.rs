use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(super) struct FacadeProposeArgs {
    #[arg(long)]
    pub(super) input: PathBuf,
    #[arg(long = "case")]
    pub(super) case_dir: PathBuf,
    #[arg(long, default_value = "technical_advisory")]
    pub(super) package: String,
    #[arg(long, default_value = "technical_advisory_mvp")]
    pub(super) ruleset: String,
    #[arg(long, default_value = "ai_agent")]
    pub(super) audience: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct FacadeStatusArgs {
    #[arg(long = "case")]
    pub(super) case_dir: PathBuf,
    #[arg(long)]
    pub(super) brief: bool,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct FacadeReportArgs {
    #[arg(long = "case")]
    pub(super) case_dir: PathBuf,
    #[arg(long)]
    pub(super) audience: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct FacadeCompletionReviewArgs {
    #[arg(long = "case")]
    pub(super) case_dir: PathBuf,
    #[arg(long = "candidate-id")]
    pub(super) candidate_id: String,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct FacadeHypothesisReviewArgs {
    #[arg(long = "case")]
    pub(super) case_dir: PathBuf,
    #[arg(long = "hypothesis-id")]
    pub(super) hypothesis_id: String,
    #[arg(long = "evidence")]
    pub(super) evidence: Vec<String>,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct HypothesisProposeArgs {
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long = "from-report")]
    pub(super) from_report: PathBuf,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct HypothesisApplyProposalsArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "from-report")]
    pub(super) from_report: PathBuf,
    #[arg(long)]
    pub(super) policy: Option<PathBuf>,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long)]
    pub(super) dry_run: bool,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct HypothesisFalsifyArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "from-report")]
    pub(super) from_report: PathBuf,
    #[arg(long = "hypothesis-id")]
    pub(super) hypothesis_id: String,
    #[arg(long = "evidence")]
    pub(super) evidence: Vec<String>,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct ObservationRecordArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "space-id")]
    pub(super) space_id: String,
    #[arg(long = "from-projection")]
    pub(super) from_projection: PathBuf,
    #[arg(long = "task-id")]
    pub(super) task_id: String,
    #[arg(long)]
    pub(super) result: PathBuf,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Subcommand)]
pub(super) enum CompletionsCommand {
    Propose(CompletionProposeArgs),
    DryRun(CompletionDryRunArgs),
    Accept(ReviewArgs),
    Reject(ReviewArgs),
    ApplyAccepted(CompletionApplyAcceptedArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum CaseCommand {
    Import(CaseImportArgs),
    Reason(CaseReasonArgs),
    CloseCheck(CaseCloseCheckArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum DogfoodCommand {
    RepoSnapshot(DogfoodRepoSnapshotArgs),
    AdversarialFixture(DogfoodAdversarialFixtureArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum CodeCommand {
    RepoSnapshot(CodeRepoSnapshotArgs),
}

#[derive(Debug, Args)]
pub(super) struct ValidateArgs {
    #[arg(long)]
    pub(super) input: PathBuf,
    #[arg(long)]
    pub(super) schema: Option<String>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct LiftArgs {
    #[arg(long)]
    pub(super) input: PathBuf,
    #[arg(long)]
    pub(super) package: String,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CheckArgs {
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long)]
    pub(super) ruleset: String,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
    #[arg(long)]
    pub(super) fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct MicroReviewArgs {
    #[arg(long)]
    pub(super) input: PathBuf,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CompletionProposeArgs {
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long = "from-report")]
    pub(super) from_report: PathBuf,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CompletionDryRunArgs {
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long = "from-report")]
    pub(super) from_report: PathBuf,
    #[arg(long = "candidate-id")]
    pub(super) candidate_ids: Vec<String>,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CompletionApplyAcceptedArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "space-id")]
    pub(super) space_id: String,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long)]
    pub(super) dry_run: bool,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct ReviewArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "candidate-id")]
    pub(super) candidate_id: String,
    #[arg(long = "from-report")]
    pub(super) from_report: Option<PathBuf>,
    #[arg(long)]
    pub(super) reviewer: String,
    #[arg(long)]
    pub(super) reason: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct ProjectArgs {
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long)]
    pub(super) report: PathBuf,
    #[arg(long = "completions-report")]
    pub(super) completions_report: Option<PathBuf>,
    #[arg(long)]
    pub(super) audience: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct DogfoodRepoSnapshotArgs {
    #[arg(long, default_value = ".")]
    pub(super) repo: PathBuf,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct DogfoodAdversarialFixtureArgs {
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CodeRepoSnapshotArgs {
    #[arg(long, default_value = ".")]
    pub(super) repo: PathBuf,
    #[arg(long)]
    pub(super) output: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CaseImportArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long)]
    pub(super) space: PathBuf,
    #[arg(long = "revision-id")]
    pub(super) revision_id: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CaseReasonArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "space-id")]
    pub(super) space_id: String,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}

#[derive(Debug, Args)]
pub(super) struct CaseCloseCheckArgs {
    #[arg(long)]
    pub(super) store: PathBuf,
    #[arg(long = "space-id")]
    pub(super) space_id: String,
    #[arg(long = "base-revision")]
    pub(super) base_revision: Option<String>,
    #[arg(long, default_value = "json")]
    pub(super) format: String,
}
