//! Shared types, traits, and error definitions used across all wsr crates.
//!
//! `wsr-types` is the dependency-free foundation of the workspace. Every other crate
//! depends on this one; it must not depend on any sibling crates.
//!
//! # Contents
//!
//! - [`WorkflowProvider`] — the trait every CI provider adapter must implement
//! - [`WorkflowIR`] — normalized internal representation that all providers compile to
//! - [`TriggerEvent`] — provider-agnostic event passed to the engine
//! - [`GitHook`] — the set of git hooks wsr can manage
//! - [`ExecutionTier`] — Tier 1 (Wasmtime/WASI 3) vs Tier 2 (Wasmer/WASIX)
//! - [`WsrError`] — top-level error type

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The normalized internal representation every provider compiles its workflow into.
///
/// This is the interchange format between the provider layer and the engine. It is
/// designed to serialize to JSON without loss of information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowIR {
    pub name: String,
    pub jobs: HashMap<String, JobIR>,
    pub triggers: Vec<Trigger>,
}

/// A single job within a workflow, post-normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobIR {
    pub id: String,
    pub needs: Vec<String>,
    pub steps: Vec<StepIR>,
    pub matrix: Option<MatrixIR>,
}

/// A single step within a job, post-normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepIR {
    pub id: Option<String>,
    pub name: Option<String>,
    pub kind: StepKind,
    pub condition: Option<String>,
    pub env: HashMap<String, String>,
    pub continue_on_error: bool,
}

/// How a step executes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepKind {
    /// `run:` directive — shell script
    Run { script: String, shell: ShellKind },
    /// `uses:` directive — action reference
    Uses { reference: ActionRef },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellKind {
    Bash,
    Sh,
    Pwsh,
}

/// A fully-resolved action reference (`owner/action@ref` or `./local/path`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRef {
    pub spec: String,
    pub sha: Option<String>,
}

/// Matrix strategy definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixIR {
    pub axes: HashMap<String, Vec<serde_json::Value>>,
    pub include: Vec<HashMap<String, serde_json::Value>>,
    pub exclude: Vec<HashMap<String, serde_json::Value>>,
}

/// A workflow trigger before provider-specific normalization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    Push,
    PullRequest,
    WorkflowDispatch,
    WorkflowCall,
    Schedule,
    Other(String),
}

/// A git hook that wsr can install and manage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitHook {
    PreCommit,
    PrePush,
    CommitMsg,
    PostCheckout,
    PostMerge,
    PostRewrite,
}

/// A provider-agnostic event passed to the engine at execution time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub trigger: Trigger,
    pub payload: serde_json::Value,
}

/// Which execution sandbox to use for a given job or step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionTier {
    /// Wasmtime + WASI Preview 3. Default for all jobs. ~1–3 ms cold start.
    Vault,
    /// Wasmer + WASIX. POSIX-compatible. Used for toolchains that require it. Transitional.
    Workshop,
}

/// The trait every CI provider adapter must implement.
///
/// The engine and sandbox never know which provider is active — they always work
/// with [`WorkflowIR`] and a resolved [`ContextMap`].
pub trait WorkflowProvider: Send + Sync {
    /// Parse raw workflow file bytes into the normalized IR.
    fn parse(&self, raw: &[u8]) -> anyhow::Result<WorkflowIR>;

    /// Build the context object used for expression evaluation.
    fn context(&self, event: &TriggerEvent) -> anyhow::Result<ContextMap>;

    /// Map provider-specific trigger names to git hook names.
    fn trigger_map(&self) -> HashMap<Trigger, GitHook>;
}

/// A flat, JSON-compatible map of context values for expression evaluation.
pub type ContextMap = HashMap<String, serde_json::Value>;

/// Top-level error type for wsr.
#[derive(Debug, Error)]
pub enum WsrError {
    #[error("workflow parse error: {0}")]
    Parse(String),

    #[error("expression evaluation error: {0}")]
    Expr(String),

    #[error("sandbox capability denied: {0}")]
    CapabilityDenied(String),

    #[error("action resolution failed for {reference}: {reason}")]
    Resolution { reference: String, reason: String },

    #[error("git hook error: {0}")]
    GitHook(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
