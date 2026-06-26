//! Command-line interface definitions for wsr.
//!
//! This crate owns all clap-specific code: the top-level [`Cli`] struct, every
//! subcommand enum, and all argument types. It is a library — it contains no
//! `main` function and no side effects.
//!
//! The binary entry point (`crates/wsr/src/bin/wsr.rs`) depends on this crate
//! to parse `std::env::args()`, then hands the structured result to
//! `crates/wsr/src/commands/` for dispatch.
//!
//! # Why separate from the binary?
//!
//! Keeping clap definitions in a standalone library lets other tools (shell
//! completion generators, man-page renderers, test harnesses) import the CLI
//! schema without pulling in the full binary dependency graph.

use clap::{Parser, Subcommand};

/// Top-level CLI entry point.
#[derive(Parser, Debug)]
#[command(
    name = "wsr",
    about = "A local, Wasm-sandboxed CI runner — run your workflows before you push, not after.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

/// All wsr subcommands.
#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Scan workflows, install git hook shims, and generate wsr.json
    Init,

    /// Run workflows locally
    Run(RunArgs),

    /// Watch workflow files and keep hooks in sync automatically
    Daemon(DaemonArgs),

    /// List all workflows and their current hook mappings
    List,

    /// Parse and pretty-print a workflow file, validate expressions
    Inspect {
        /// Path to the workflow file
        file: String,
    },

    /// Manage the compiled Wasm module cache
    Cache {
        #[command(subcommand)]
        action: CacheCmd,
    },

    /// Manually install or remove individual git hook shims
    Hook {
        #[command(subcommand)]
        action: HookCmd,
    },

    /// Show active hook map, daemon state, and last sync time
    Status,
}

/// Arguments for `wsr run`.
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Specific workflow file to run (default: all with workflow_dispatch)
    pub file: Option<String>,

    /// Force a specific trigger event (e.g. push, pull_request)
    #[arg(long)]
    pub event: Option<String>,

    /// Print the execution plan without running anything
    #[arg(long)]
    pub dry_run: bool,

    /// Show expression evaluation, sandbox grants, and context dump
    #[arg(long)]
    pub verbose: bool,

    /// Skip interactive prompts (for scripted use)
    #[arg(long)]
    pub yes: bool,

    /// Output format: human (default) or gha (GitHub Annotations)
    #[arg(long, default_value = "human")]
    pub format: String,
}

/// Arguments for `wsr daemon`.
#[derive(Parser, Debug)]
pub struct DaemonArgs {
    /// Register the daemon as a launchd (macOS) or systemd (Linux) service
    #[arg(long)]
    pub install: bool,
}

/// Subcommands for `wsr cache`.
#[derive(Subcommand, Debug)]
pub enum CacheCmd {
    /// List cached Wasm modules with their SHA-256 keys and on-disk sizes
    List,
    /// Verify the integrity of all cached modules
    Verify,
    /// Remove all cached modules
    Purge,
}

/// Subcommands for `wsr hook`.
#[derive(Subcommand, Debug)]
pub enum HookCmd {
    /// Install a shim for a specific git hook
    Install {
        /// Git hook name (e.g. pre-push, pre-commit)
        hook: String,
    },
    /// Remove a git hook shim
    Remove {
        /// Git hook name
        hook: String,
    },
}
