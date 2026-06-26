//! GitHub Actions provider adapter — the reference implementation of [`WorkflowProvider`].
//!
//! Handles everything specific to GitHub Actions syntax so the engine and sandbox
//! remain provider-agnostic. GitLab CI and Bitbucket Pipelines will be separate
//! crates following the same pattern.
//!
//! # Responsibilities
//!
//! - **YAML parsing** — deserializes `.github/workflows/*.yml` via `serde_yaml`
//!   into typed structs, then compiles them into [`WorkflowIR`].
//! - **Context builder** — produces the `github.*`, `env.*`, `runner.*`, `secrets.*`,
//!   `needs.*`, `steps.*`, and `inputs.*` context maps for expression evaluation.
//! - **Trigger mapping** — implements `trigger_map()`, mapping `on: push`,
//!   `on: pull_request`, and `on: workflow_dispatch` to git hooks.
//! - **Workflow discovery** — scans `.github/workflows/` and returns all parseable
//!   workflow files.
//!
//! # Provider selection
//!
//! The active provider is declared in `wsr.json`:
//!
//! ```json
//! { "provider": "github" }
//! ```
//!
//! [`WorkflowProvider`]: wsr_types::WorkflowProvider
//! [`WorkflowIR`]: wsr_types::WorkflowIR
