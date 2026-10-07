use clap::{Args, Parser, Subcommand, ValueEnum};

/// Develop and grow this workspace.
#[derive(Debug, Parser)]
#[command(name = "xtask", version, propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Check formatting, compilation, and lints.
    Check,
    /// Check the workspace, then run its tests.
    Test(TestArgs),
    /// Test the workspace, then build release artifacts.
    Build,
    /// Run the checks used by generated CI.
    Ci(CiArgs),
    /// Generate an HTML coverage report.
    Coverage,
    /// Add an optional project capability.
    Scaffold {
        /// Show planned writes without changing files.
        #[arg(long, global = true)]
        dry_run: bool,
        #[command(subcommand)]
        command: ScaffoldCommand,
    },
    /// Report the availability of optional development tools.
    Doctor,
    /// Manage pinned, project-local development tools.
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
    /// Initialize, inspect, or prepare releases.
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
}

#[derive(Debug, Args)]
pub(crate) struct TestArgs {
    /// Optional test-name filter.
    pub(crate) filter: Option<String>,
    /// Use cargo-nextest instead of Cargo's built-in test runner.
    #[arg(long)]
    pub(crate) nextest: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CiArgs {
    /// Also run documentation, dependency-policy, and typo checks.
    #[arg(long)]
    pub(crate) full: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ScaffoldCommand {
    /// Add a library or binary crate.
    Crate {
        /// Stable domain or operational noun appended to the crate prefix.
        semantic_name: String,
        /// Generate a binary instead of a library.
        #[arg(long)]
        bin: bool,
        /// Mark the crate as unpublished.
        #[arg(long)]
        private: bool,
    },
    /// Add a Clap model crate and executable entrypoint.
    Cli {
        /// Choose where the executable target lives.
        #[arg(long, value_enum, default_value_t)]
        entrypoint: CliEntrypoint,
    },
    /// Add a GitHub Actions workflow.
    Ci {
        /// Lean uses Cargo only; full also enables policy and typo checks.
        #[arg(long, value_enum, default_value_t)]
        preset: CiPreset,
    },
    /// Add contribution and crate-index documentation.
    Docs,
    /// Add agent instruction files.
    Agents {
        /// Also add CLAUDE.md pointing at AGENTS.md.
        #[arg(long)]
        claude: bool,
    },
}

/// Location of the executable target created by the CLI scaffold.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum CliEntrypoint {
    /// Keep the primary library free of command-line dependencies.
    #[default]
    Companion,
    /// Add a thin executable wrapper to the primary crate.
    Primary,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum CiPreset {
    #[default]
    Lean,
    Full,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ToolsCommand {
    /// Install an exact tool group under .xtask/tools.
    Sync {
        #[arg(value_enum, default_value_t)]
        group: ToolGroup,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum ToolGroup {
    Ci,
    Coverage,
    Release,
    Test,
    #[default]
    All,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ReleaseCommand {
    /// Create release configuration and initialize dist for binary projects.
    Init,
    /// Ask dist to preview the release plan.
    Plan {
        /// Optional release tag to plan.
        #[arg(long)]
        tag: Option<String>,
    },
    /// Prepare a version, changelog, commit, and tag without local publishing.
    Prepare {
        #[arg(value_enum, default_value_t)]
        level: ReleaseLevel,
        /// Apply changes. The default is cargo-release's dry run.
        #[arg(long)]
        execute: bool,
    },
    #[command(hide = true)]
    Changelog {
        #[arg(long)]
        tag: String,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum ReleaseLevel {
    #[default]
    Patch,
    Minor,
    Major,
}

impl ReleaseLevel {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Patch => "patch",
            Self::Minor => "minor",
            Self::Major => "major",
        }
    }
}
