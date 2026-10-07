//! Early shared types, traits, and errors for the current CLI/workspace scaffold.
//!
//! These declarations predate the accepted design in the repository PLAN.md.
//! No provider compiler, execution engine, or runtime is implemented by these types.
//! The legacy tier enum does not select production runtimes, and the model is not
//! a complete or stable GitHub Actions compatibility/serialization contract.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Early serializable workflow model in the scaffold.
///
/// No provider compiler is implemented, and this model does not cover all source semantics.
/// The accepted canonical model and its stability boundary still require specification.
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
    /// Legacy component-tier identifier; no runtime implementation or startup measurement.
    Vault,
    /// Legacy compatibility-tier identifier; no WASIX or system backend is implemented.
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
