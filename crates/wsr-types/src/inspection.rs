//! Provider-neutral source summaries for inspection, not an executable or stable serialized IR.

/// A workflow's statically inspected source structure.
#[derive(Debug)]
pub struct WorkflowSummary {
    /// Optional source display name.
    pub name: Option<String>,
    /// Optional static trigger name, not an evaluated invocation event.
    pub trigger: Option<String>,
    /// Jobs in deterministic identifier order.
    pub jobs: Vec<JobSummary>,
}

/// A job's source-level requirements and ordered steps.
#[derive(Debug)]
pub struct JobSummary {
    /// Source identifier.
    pub id: String,
    /// Optional display name.
    pub name: Option<String>,
    /// Literal runner requirement, not a resolved environment.
    pub runner: String,
    /// Static source dependency identifiers, retaining declared order and repetitions.
    pub needs: Vec<String>,
    /// Steps in source order.
    pub steps: Vec<StepSummary>,
}

/// A step's source summary; it contains no executable script.
#[derive(Debug)]
pub struct StepSummary {
    /// Optional display name.
    pub name: Option<String>,
    /// Source operation classification.
    pub kind: StepSummaryKind,
}

/// Source operation classification, not an execution capability claim.
#[derive(Debug)]
pub enum StepSummaryKind {
    /// A command is present, but is never executed by inspection.
    Run,
    /// An unresolved action reference; inspection never fetches or opens it.
    Uses(String),
}
