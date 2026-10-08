//! Binary library root for wsr.
//!
//! Argument parsing, help/version, dispatch, and placeholder diagnostics are implemented.
//! Workflow execution is not implemented.
//!
//! `crates/wsr` is the entry point to the wsr command-line interface.
//! The Rust API exposed here is not considered public — it exists solely to
//! support the binary targets in `src/bin/` and the integration tests in
//! `tests/`.
//!
//! # Structure
//!
//! ```text
//! crates/wsr/
//!   src/
//!     bin/
//!       wsr.rs        ← forward OS-string arguments to main()
//!     commands/
//!       mod.rs        ← dispatch table
//!       init.rs       ← wsr init
//!       run.rs        ← wsr run
//!       daemon.rs     ← wsr daemon
//!       list.rs       ← wsr list
//!       inspect.rs    ← wsr inspect
//!       cache.rs      ← wsr cache
//!       hook.rs       ← wsr hook
//!       status.rs     ← wsr status
//!     lib.rs          ← parsing, diagnostics, and process exit status
//! ```
//!
//! CLI argument types live in `wsr-cli`. This crate dispatches to placeholders without
//! loading configuration, discovering repositories, or initializing execution subsystems.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;
use wsr_cli::Cli;

pub mod commands;

/// Internal process entrypoint for the binary; not a stable embedding API.
pub fn main(args: impl IntoIterator<Item = OsString>) -> ExitCode {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(error.exit_code() as u8);
        }
    };

    match commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
