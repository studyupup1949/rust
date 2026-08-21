use advisorygraphen_core::{AdvisoryError, Severity, TOOL_VERSION};
use advisorygraphen_projection::OutputFormat;
use advisorygraphen_runtime::{
    case_close_check_workflow, case_import_workflow, case_reason_workflow, check_workflow,
    code_repo_snapshot_workflow, completions_apply_accepted_workflow, completions_dry_run_workflow,
    completions_propose_workflow, dogfood_adversarial_fixture_workflow,
    dogfood_repo_snapshot_workflow, facade_completion_review_workflow,
    facade_hypothesis_review_workflow, facade_propose_workflow, facade_report_workflow,
    facade_status_workflow, hypothesis_accept_workflow, hypothesis_apply_proposals_workflow,
    hypothesis_falsify_workflow, hypothesis_propose_workflow, hypothesis_reject_workflow,
    hypothesis_support_workflow, lift_workflow, micro_review_workflow, observation_record_workflow,
    project_workflow, review_workflow, validate_workflow, CaseCloseCheckOptions, CaseImportOptions,
    CaseReasonOptions, CheckOptions, CodeRepoSnapshotOptions, CompletionApplyAcceptedOptions,
    CompletionDryRunOptions, CompletionProposeOptions, DogfoodAdversarialFixtureOptions,
    DogfoodRepoSnapshotOptions, FacadeCompletionReviewOptions, FacadeHypothesisReviewOptions,
    FacadeProposeOptions, FacadeReportOptions, FacadeStatusOptions,
    HypothesisApplyProposalsOptions, HypothesisFalsifyOptions, HypothesisProposeOptions,
    LiftOptions, MicroReviewOptions, ObservationRecordOptions, ProjectOptions, ReviewOptions,
    ValidateOptions,
};
use clap::Parser;
use serde::Serialize;

#[path = "main/args.rs"]
mod args;
#[path = "main/commands.rs"]
mod commands;

use args::*;
use commands::*;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(error.exit_code());
    }
}

fn run() -> Result<(), AdvisoryError> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Version) {
        Command::Version => {
            println!("advisorygraphen {TOOL_VERSION}");
            Ok(())
        }
        Command::Validate(args) => {
            require_json_format(&args.format)?;
            print_json(&validate_workflow(&ValidateOptions {
                input: args.input,
                schema: args.schema,
            })?)
        }
        Command::Lift(args) => {
            require_json_format(&args.format)?;
            let space = lift_workflow(&LiftOptions {
                input: args.input,
                package: args.package,
                output: args.output,
                command: Some(command_string()),
            })?;
            print_json(&space)
        }
        Command::Check(args) => {
            require_json_format(&args.format)?;
            let fail_on = parse_fail_on(args.fail_on.as_deref())?;
            let report = check_workflow(&CheckOptions {
                space: args.space,
                ruleset: args.ruleset,
                output: args.output,
                fail_on,
                command: Some(command_string()),
            })?;
            print_json(&report)
        }
        Command::Propose(args) => {
            require_json_format(&args.format)?;
            print_json(&facade_propose_workflow(&FacadeProposeOptions {
                input: args.input,
                case_dir: args.case_dir,
                package: args.package,
                ruleset: args.ruleset,
                audience: args.audience,
                command: Some(command_string()),
            })?)
        }
        Command::Status(args) => {
            require_json_format(&args.format)?;
            print_json(&facade_status_workflow(&FacadeStatusOptions {
                case_dir: args.case_dir,
                brief: args.brief,
            })?)
        }
        Command::Report(args) => {
            let format = OutputFormat::parse(&args.format)?;
            let rendered = facade_report_workflow(&FacadeReportOptions {
                case_dir: args.case_dir,
                audience: args.audience,
                format,
                output: args.output,
            })?;
            println!("{rendered}");
            Ok(())
        }
        Command::Review { command } => match command {
            FacadeReviewCommand::Completion { command } => match command {
                FacadeCompletionReviewCommand::Accept(args) => {
                    run_facade_completion_review(args, "accepted")
                }
                FacadeCompletionReviewCommand::Reject(args) => {
                    run_facade_completion_review(args, "rejected")
                }
            },
            FacadeReviewCommand::Hypothesis { command } => match command {
                FacadeHypothesisReviewCommand::Support(args) => {
                    run_facade_hypothesis_review(args, "support")
                }
                FacadeHypothesisReviewCommand::Falsify(args) => {
                    run_facade_hypothesis_review(args, "falsify")
                }
                FacadeHypothesisReviewCommand::Accept(args) => {
                    run_facade_hypothesis_review(args, "accept")
                }
                FacadeHypothesisReviewCommand::Reject(args) => {
                    run_facade_hypothesis_review(args, "reject")
                }
            },
        },
        Command::Micro { command } => match command {
            MicroCommand::Review(args) => {
                require_json_format(&args.format)?;
                let report = micro_review_workflow(&MicroReviewOptions {
                    input: args.input,
                    output: args.output,
                    command: Some(command_string()),
                })?;
                print_json(&report)
            }
        },
        Command::Completions { command } => match command {
            CompletionsCommand::Propose(args) => {
                require_json_format(&args.format)?;
                let report = completions_propose_workflow(&CompletionProposeOptions {
                    space: args.space,
                    from_report: args.from_report,
                    output: args.output,
                    command: Some(command_string()),
                })?;
                print_json(&report)
            }
            CompletionsCommand::DryRun(args) => {
                require_json_format(&args.format)?;
                let report = completions_dry_run_workflow(&CompletionDryRunOptions {
                    space: args.space,
                    from_report: args.from_report,
                    candidate_ids: args.candidate_ids,
                    output: args.output,
                    command: Some(command_string()),
                })?;
                print_json(&report)
            }
            CompletionsCommand::Accept(args) => run_review(args, "accepted"),
            CompletionsCommand::Reject(args) => run_review(args, "rejected"),
            CompletionsCommand::ApplyAccepted(args) => {
                require_json_format(&args.format)?;
                print_json(&completions_apply_accepted_workflow(
                    &CompletionApplyAcceptedOptions {
                        store: args.store,
                        space_id: args.space_id,
                        reviewer: args.reviewer,
                        reason: args.reason,
                        base_revision: args.base_revision,
                        dry_run: args.dry_run,
                    },
                )?)
            }
        },
        Command::Project(args) => {
            let format = OutputFormat::parse(&args.format)?;
            let rendered = project_workflow(&ProjectOptions {
                space: args.space,
                report: args.report,
                completions_report: args.completions_report,
                audience: args.audience,
                format,
                output: args.output,
            })?;
            println!("{rendered}");
            Ok(())
        }
        Command::Dogfood { command } => match command {
            DogfoodCommand::RepoSnapshot(args) => {
                require_json_format(&args.format)?;
                print_json(&dogfood_repo_snapshot_workflow(
                    &DogfoodRepoSnapshotOptions {
                        repo: args.repo,
                        output: args.output,
                    },
                )?)
            }
            DogfoodCommand::AdversarialFixture(args) => {
                require_json_format(&args.format)?;
                print_json(&dogfood_adversarial_fixture_workflow(
                    &DogfoodAdversarialFixtureOptions {
                        output: args.output,
                    },
                )?)
            }
        },
        Command::Code { command } => match command {
            CodeCommand::RepoSnapshot(args) => {
                require_json_format(&args.format)?;
                print_json(&code_repo_snapshot_workflow(&CodeRepoSnapshotOptions {
                    repo: args.repo,
                    output: args.output,
                })?)
            }
        },
        Command::Hypothesis { command } => match command {
            HypothesisCommand::Propose(args) => {
                require_json_format(&args.format)?;
                print_json(&hypothesis_propose_workflow(&HypothesisProposeOptions {
                    space: args.space,
                    from_report: args.from_report,
                    output: args.output,
                    command: Some(command_string()),
                })?)
            }
            HypothesisCommand::ApplyProposals(args) => {
                require_json_format(&args.format)?;
                print_json(&hypothesis_apply_proposals_workflow(
                    &HypothesisApplyProposalsOptions {
                        store: args.store,
                        from_report: args.from_report,
                        policy: args.policy,
                        reviewer: args.reviewer,
                        reason: args.reason,
                        base_revision: args.base_revision,
                        dry_run: args.dry_run,
                    },
                )?)
            }
            HypothesisCommand::Falsify(args) => run_hypothesis_event(args, "falsify"),
            HypothesisCommand::Support(args) => run_hypothesis_event(args, "support"),
            HypothesisCommand::Accept(args) => run_hypothesis_event(args, "accept"),
            HypothesisCommand::Reject(args) => run_hypothesis_event(args, "reject"),
        },
        Command::Observation { command } => match command {
            ObservationCommand::Record(args) => {
                require_json_format(&args.format)?;
                print_json(&observation_record_workflow(&ObservationRecordOptions {
                    store: args.store,
                    space_id: args.space_id,
                    from_projection: args.from_projection,
                    task_id: args.task_id,
                    result: args.result,
                    reviewer: args.reviewer,
                    reason: args.reason,
                    base_revision: args.base_revision,
                })?)
            }
        },
        Command::Case { command } => match command {
            CaseCommand::Import(args) => {
                require_json_format(&args.format)?;
                print_json(&case_import_workflow(&CaseImportOptions {
                    store: args.store,
                    space: args.space,
                    revision_id: args.revision_id,
                })?)
            }
            CaseCommand::Reason(args) => {
                require_json_format(&args.format)?;
                print_json(&case_reason_workflow(&CaseReasonOptions {
                    store: args.store,
                    space_id: args.space_id,
                })?)
            }
            CaseCommand::CloseCheck(args) => {
                require_json_format(&args.format)?;
                print_json(&case_close_check_workflow(&CaseCloseCheckOptions {
                    store: args.store,
                    space_id: args.space_id,
                    base_revision: args.base_revision,
                })?)
            }
        },
    }
}

fn run_facade_completion_review(
    args: FacadeCompletionReviewArgs,
    outcome: &str,
) -> Result<(), AdvisoryError> {
    require_json_format(&args.format)?;
    print_json(&facade_completion_review_workflow(
        &FacadeCompletionReviewOptions {
            case_dir: args.case_dir,
            candidate_id: args.candidate_id,
            reviewer: args.reviewer,
            reason: args.reason,
            outcome: outcome.to_string(),
        },
    )?)
}

fn run_facade_hypothesis_review(
    args: FacadeHypothesisReviewArgs,
    outcome: &str,
) -> Result<(), AdvisoryError> {
    require_json_format(&args.format)?;
    print_json(&facade_hypothesis_review_workflow(
        &FacadeHypothesisReviewOptions {
            case_dir: args.case_dir,
            hypothesis_id: args.hypothesis_id,
            evidence_ids: args.evidence,
            reviewer: args.reviewer,
            reason: args.reason,
            outcome: outcome.to_string(),
        },
    )?)
}

fn run_hypothesis_event(args: HypothesisFalsifyArgs, action: &str) -> Result<(), AdvisoryError> {
    require_json_format(&args.format)?;
    let options = HypothesisFalsifyOptions {
        store: args.store,
        from_report: args.from_report,
        hypothesis_id: args.hypothesis_id,
        evidence_ids: args.evidence,
        reviewer: args.reviewer,
        reason: args.reason,
        base_revision: args.base_revision,
    };
    let event = match action {
        "falsify" => hypothesis_falsify_workflow(&options)?,
        "support" => hypothesis_support_workflow(&options)?,
        "accept" => hypothesis_accept_workflow(&options)?,
        "reject" => hypothesis_reject_workflow(&options)?,
        other => {
            return Err(AdvisoryError::Validation(format!(
                "unsupported hypothesis action: {other}"
            )))
        }
    };
    print_json(&event)
}

fn run_review(args: ReviewArgs, outcome: &str) -> Result<(), AdvisoryError> {
    require_json_format(&args.format)?;
    print_json(&review_workflow(&ReviewOptions {
        store: args.store,
        candidate_id: args.candidate_id,
        from_report: args.from_report,
        reviewer: args.reviewer,
        reason: args.reason,
        outcome: outcome.to_string(),
        base_revision: args.base_revision,
    })?)
}

fn parse_fail_on(value: Option<&str>) -> Result<Option<Severity>, AdvisoryError> {
    value
        .map(|value| {
            Severity::parse(value)
                .ok_or_else(|| AdvisoryError::Validation(format!("invalid severity: {value}")))
        })
        .transpose()
}

fn require_json_format(format: &str) -> Result<(), AdvisoryError> {
    if format == "json" {
        Ok(())
    } else {
        Err(AdvisoryError::Validation(format!(
            "only json format is supported for this command: {format}"
        )))
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<(), AdvisoryError> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn command_string() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}
