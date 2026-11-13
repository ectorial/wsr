//! Temporary GHA-derived source summaries for D-031 inspection (D-033), not provider-neutral
//! model types and not an executable or stable serialized IR.

/// A workflow's statically inspected source structure.
#[derive(Debug)]
pub struct WorkflowSummary {
    /// Optional source display name.
    pub name: Option<String>,
    /// Optional source run-name text, never evaluated.
    pub run_name: Option<SourceText>,
    /// Optional static trigger name, not an evaluated invocation event.
    pub trigger: Option<ManualTrigger>,
    /// Optional workflow-level token permissions request, never granted by inspection.
    pub permissions: Option<Permissions>,
    /// Jobs in deterministic identifier order.
    pub jobs: Vec<JobSummary>,
}

/// Decoded source text, classified without evaluating or validating expressions.
#[derive(Debug, PartialEq, Eq)]
pub enum SourceText {
    /// Text without an expression opening fence.
    Literal(String),
    /// Entire raw text containing at least one `${{` fence; syntax is not validated.
    Unevaluated(String),
}

/// A job's source-level requirements and ordered steps.
#[derive(Debug)]
pub struct JobSummary {
    /// Source identifier.
    pub id: String,
    /// Optional display name.
    pub name: Option<String>,
    /// Literal runner requirement, not a resolved environment.
    pub runner: RunnerRequirement,
    /// Static source dependency identifiers, retaining declared order and repetitions.
    pub needs: Vec<String>,
    /// Optional job-level token permissions request, not merged with the workflow's.
    pub permissions: Option<Permissions>,
    /// Optional source timeout: a literal positive YAML number rendered as text, or unevaluated
    /// expression text. Never applied.
    pub timeout_minutes: Option<SourceText>,
    /// Steps in source order.
    pub steps: Vec<StepSummary>,
}

/// A step's source summary; it contains no executable script.
#[derive(Debug)]
pub struct StepSummary {
    /// Optional explicit source identifier; inspection never generates an ID.
    pub id: Option<String>,
    /// Optional display name.
    pub name: Option<String>,
    /// Optional literal shell of a run step, never selected or invoked.
    pub shell: Option<String>,
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

/// Literal source requirement, not a selected or provisioned runner.
#[derive(Debug, PartialEq, Eq)]
pub enum RunnerRequirement {
    /// One source label in scalar form.
    Label(String),
    /// Source list, retaining order, spelling and repetitions.
    Labels(Vec<String>),
}

/// Requested `GITHUB_TOKEN` permissions as declared, not an effective or granted token.
#[derive(Debug, PartialEq, Eq)]
pub enum Permissions {
    /// The `read-all` shorthand.
    ReadAll,
    /// The `write-all` shorthand.
    WriteAll,
    /// Declared scopes in name order; empty for an explicitly empty mapping.
    Scopes(std::collections::BTreeMap<String, PermissionLevel>),
}

/// A requested level for one permission scope.
#[derive(Debug, PartialEq, Eq)]
pub enum PermissionLevel {
    /// `read`.
    Read,
    /// `write`.
    Write,
    /// `none`.
    None,
}

/// Source form of the supported manual trigger, never an invocation.
#[derive(Debug, PartialEq, Eq)]
pub enum ManualTrigger {
    /// Scalar `workflow_dispatch`.
    Scalar,
    /// Mapping entry; None retains an explicitly null dispatch body.
    Mapping(Option<DispatchBody>),
}

/// Declared manual-trigger configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct DispatchBody {
    /// None means no inputs field; Some(empty) means an explicitly empty mapping.
    pub inputs: Option<Vec<DispatchInput>>,
}

/// One input declaration, without bound values or applied defaults.
#[derive(Debug, PartialEq, Eq)]
pub struct DispatchInput {
    /// Exact source identifier.
    pub id: String,
    /// Explicit description.
    pub description: Option<String>,
    /// Explicit type, retaining omission rather than injecting a default.
    pub input_type: Option<DispatchInputType>,
    /// Explicit required flag, not enforced by inspection.
    pub required: Option<bool>,
    /// Explicit scalar default, never applied.
    pub default: Option<InputDefault>,
    /// Explicit options in source order.
    pub options: Option<Vec<String>>,
}

/// Supported declared manual-input types.
#[derive(Debug, PartialEq, Eq)]
pub enum DispatchInputType {
    /// String declaration.
    String,
    /// Boolean declaration.
    Boolean,
    /// Number declaration.
    Number,
    /// Choice declaration.
    Choice,
    /// Environment declaration; no environment lookup occurs.
    Environment,
}

/// A typed scalar declaration, not an evaluated input value.
#[derive(Debug, PartialEq, Eq)]
pub enum InputDefault {
    /// Literal string.
    String(String),
    /// Literal boolean.
    Boolean(bool),
    /// YAML number represented as text, without conversion to a string default.
    Number(String),
}
