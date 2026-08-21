use super::*;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "advisorygraphen",
    version,
    about = "Structured technical advisory CLI"
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    Version,
    Validate(ValidateArgs),
    Lift(LiftArgs),
    Check(CheckArgs),
    Propose(FacadeProposeArgs),
    Status(FacadeStatusArgs),
    Report(FacadeReportArgs),
    Review {
        #[command(subcommand)]
        command: FacadeReviewCommand,
    },
    Micro {
        #[command(subcommand)]
        command: MicroCommand,
    },
    Completions {
        #[command(subcommand)]
        command: CompletionsCommand,
    },
    Project(ProjectArgs),
    Dogfood {
        #[command(subcommand)]
        command: DogfoodCommand,
    },
    Code {
        #[command(subcommand)]
        command: CodeCommand,
    },
    Case {
        #[command(subcommand)]
        command: CaseCommand,
    },
    Hypothesis {
        #[command(subcommand)]
        command: HypothesisCommand,
    },
    Observation {
        #[command(subcommand)]
        command: ObservationCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(super) enum HypothesisCommand {
    Propose(HypothesisProposeArgs),
    ApplyProposals(HypothesisApplyProposalsArgs),
    Falsify(HypothesisFalsifyArgs),
    Support(HypothesisFalsifyArgs),
    Accept(HypothesisFalsifyArgs),
    Reject(HypothesisFalsifyArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum ObservationCommand {
    Record(ObservationRecordArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum FacadeReviewCommand {
    Completion {
        #[command(subcommand)]
        command: FacadeCompletionReviewCommand,
    },
    Hypothesis {
        #[command(subcommand)]
        command: FacadeHypothesisReviewCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(super) enum FacadeCompletionReviewCommand {
    Accept(FacadeCompletionReviewArgs),
    Reject(FacadeCompletionReviewArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum FacadeHypothesisReviewCommand {
    Support(FacadeHypothesisReviewArgs),
    Falsify(FacadeHypothesisReviewArgs),
    Accept(FacadeHypothesisReviewArgs),
    Reject(FacadeHypothesisReviewArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum MicroCommand {
    Review(MicroReviewArgs),
}
