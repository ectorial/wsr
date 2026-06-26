//! Binary library root for wsr.
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
//!       wsr.rs        ← binary entry point: parse args, call run()
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
//!     lib.rs          ← this file
//! ```
//!
//! CLI argument types live in `wsr-cli` — this crate bridges them to the
//! implementations in `wsr-engine`, `wsr-git`, and the rest of the workspace.

pub mod commands;
