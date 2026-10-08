use std::path::PathBuf;

use clap::{Args, ValueEnum, ValueHint};

/// Arguments for `wsr run`; all options describe unimplemented behavior.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Workflow file to select
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: Option<PathBuf>,

    /// Trigger event to select (e.g. push, pull_request)
    #[arg(long)]
    pub event: Option<String>,

    /// Request a plan without execution (not implemented yet)
    #[arg(long)]
    pub dry_run: bool,

    /// Request detailed diagnostics (not implemented yet)
    #[arg(long)]
    pub verbose: bool,

    /// Skip prompts; does not grant access (not implemented yet)
    #[arg(long)]
    pub yes: bool,

    /// Requested output format (not implemented yet)
    #[arg(long, value_enum, default_value = "human")]
    pub format: OutputFormat,
}

/// Requested presentation format; neither format implements operational reporting yet.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable output.
    Human,
    /// GitHub Actions annotations.
    Gha,
}
