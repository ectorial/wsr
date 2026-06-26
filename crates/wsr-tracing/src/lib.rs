//! Structured logging and output formatting for wsr.
//!
//! Provides two output formats:
//!
//! - **human** (default) — compact, coloured, git-hook-friendly terminal output
//! - **gha** (`--format=gha`) — GitHub Annotations format for steps consumed by
//!   GitHub Actions (`::error::`, `::notice::`, `::group::`, etc.)
//!
//! All wsr crates emit spans and events via the [`tracing`] crate. This crate
//! owns subscriber initialization and format selection. No other crate should
//! configure a global subscriber.
