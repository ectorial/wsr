//! Command-line interface definitions for wsr.
//!
//! Typed arguments, help, and version reporting are implemented. Operational commands are stubs.
//!
//! This crate owns the clap schema: the top-level [`Cli`] struct, every
//! subcommand enum, and all argument types. It is a library — it contains no
//! `main` function and no side effects.
//!
//! The application entrypoint in `wsr` parses OS-string arguments using this schema,
//! then hands the structured result to its command dispatch.
//!
//! # Why separate from the binary?
//!
//! Keeping clap definitions in a standalone library lets other tools (shell
//! completion generators, man-page renderers, test harnesses) import the CLI
//! schema without pulling in the full binary dependency graph.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

mod cache;
mod daemon;
mod hook;
mod run;

pub use cache::CacheCmd;
pub use daemon::DaemonArgs;
pub use hook::HookCmd;
pub use run::{OutputFormat, RunArgs};

/// Top-level CLI entry point.
#[derive(Parser, Debug)]
#[command(
    name = "wsr",
    about = "A pre-alpha local CI CLI scaffold",
    after_help = "Operational commands are not implemented yet. Options describe planned behavior only.",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

/// All wsr subcommands.
#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Set up a repository (not implemented yet)
    Init,

    /// Run workflows locally (not implemented yet)
    Run(RunArgs),

    /// Manage workflow watching (not implemented yet)
    Daemon(DaemonArgs),

    /// List workflows (not implemented yet)
    List,

    /// Inspect a workflow file (not implemented yet)
    Inspect {
        /// Path to the workflow file
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
    },

    /// Manage the cache (not implemented yet)
    #[command(arg_required_else_help = true)]
    Cache {
        #[command(subcommand)]
        action: CacheCmd,
    },

    /// Manage git hooks (not implemented yet)
    #[command(arg_required_else_help = true)]
    Hook {
        #[command(subcommand)]
        action: HookCmd,
    },

    /// Show repository status (not implemented yet)
    Status,
}
