//! Temporary GHA-derived source summaries for D-031 inspection (D-033), not provider-neutral
//! model types and not an executable or stable serialized IR.

/// A workflow's statically inspected source structure.
#[derive(Debug)]
pub struct WorkflowSummary {
    /// Optional source display name.
    pub name: Option<String>,
    /// Optional source run-name text, never evaluated.
    pub run_name: Option<SourceText>,
    /// `on` events in source order, never subscribed to or evaluated; empty without `on`.
    pub triggers: Vec<Trigger>,
    /// Optional workflow-level token permissions request, never granted by inspection.
    pub permissions: Option<Permissions>,
    /// Optional workflow-level environment entries by name, never bound.
    pub env: Option<BTreeMap<String, StepValue>>,
    /// Optional workflow-level `defaults.run`, never applied to jobs or steps.
    pub defaults: Option<RunDefaults>,
    /// Optional workflow-level `concurrency`, never applied.
    pub concurrency: Option<Concurrency>,
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
    /// Optional display name, literal or unevaluated (D-033 `SourceText`).
    pub name: Option<SourceText>,
    /// Literal runner requirement, not a resolved environment; None only for a reusable-workflow
    /// call job.
    pub runner: Option<RunnerRequirement>,
    /// Reusable-workflow call (D-033 temporary GHA-derived type); such a job has no runner and no
    /// steps. The called workflow is never loaded.
    pub call: Option<WorkflowCall>,
    /// Static source dependency identifiers, retaining declared order and repetitions.
    pub needs: Vec<String>,
    /// Optional `if` text as written, bare or fenced; never parsed or evaluated.
    pub condition: Option<String>,
    /// Optional job-level token permissions request, not merged with the workflow's.
    pub permissions: Option<Permissions>,
    /// Optional source timeout: a literal positive YAML number rendered as text, or unevaluated
    /// expression text. Never applied.
    pub timeout_minutes: Option<SourceText>,
    /// Optional `continue-on-error`: a literal boolean rendered as text, or unevaluated text.
    /// Never applied.
    pub continue_on_error: Option<SourceText>,
    /// Optional job-level environment entries by name, not merged with the workflow's; never bound.
    pub env: Option<BTreeMap<String, StepValue>>,
    /// Optional job-level `defaults.run`, not merged with the workflow's; never applied to steps.
    pub defaults: Option<RunDefaults>,
    /// Optional job-level `concurrency`, not merged with the workflow's; never applied.
    pub concurrency: Option<Concurrency>,
    /// Optional `outputs` text by name (D-033), never resolved or exposed to other jobs.
    pub outputs: Option<BTreeMap<String, SourceText>>,
    /// Optional `environment`, never selected; no protection rules are evaluated.
    pub environment: Option<Environment>,
    /// Optional `container`, never pulled or started.
    pub container: Option<Container>,
    /// Declared strategy, never expanded or applied (D-033).
    pub strategy: Option<Strategy>,
    /// Steps in source order.
    pub steps: Vec<StepSummary>,
}

/// A job's reusable-workflow call as written (D-033 temporary GHA-derived type); the referenced
/// workflow is never opened, loaded or resolved, and nothing is passed to it.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkflowCall {
    /// `uses` reference text.
    pub uses: String,
    /// Optional `with` inputs by name, never passed.
    pub with: Option<BTreeMap<String, StepValue>>,
    /// Optional `secrets`, never passed.
    pub secrets: Option<CallSecrets>,
}

/// Declared `secrets` of a reusable-workflow call (D-033 temporary GHA-derived type).
#[derive(Debug, PartialEq, Eq)]
pub enum CallSecrets {
    /// `secrets: inherit`.
    Inherit,
    /// Secret values by name, as source text; never resolved.
    Values(BTreeMap<String, StepValue>),
}

/// A step's source summary; it contains no executable script.
#[derive(Debug)]
pub struct StepSummary {
    /// Optional explicit source identifier; inspection never generates an ID.
    pub id: Option<String>,
    /// Optional display name, literal or unevaluated (D-033 `SourceText`).
    pub name: Option<SourceText>,
    /// Optional `if` text as written, bare or fenced; never parsed or evaluated.
    pub condition: Option<String>,
    /// Optional `continue-on-error`: a literal boolean rendered as text, or unevaluated text.
    /// Never applied.
    pub continue_on_error: Option<SourceText>,
    /// Optional step timeout, as for the job's. Never applied.
    pub timeout_minutes: Option<SourceText>,
    /// Optional literal shell of a run step, never selected or invoked.
    pub shell: Option<String>,
    /// Optional working directory of a run step, never resolved.
    pub working_directory: Option<SourceText>,
    /// Optional environment entries by name, never bound.
    pub env: Option<BTreeMap<String, StepValue>>,
    /// Optional action inputs by name, never passed to an action.
    pub with: Option<BTreeMap<String, StepValue>>,
    /// Source operation classification.
    pub kind: StepSummaryKind,
}

/// Declared `concurrency` (D-033 temporary GHA-derived type); a string form is the group. Never
/// applied: no group is formed, queued or cancelled.
#[derive(Debug, PartialEq, Eq)]
pub struct Concurrency {
    /// Group text, literal or unevaluated.
    pub group: SourceText,
    /// `cancel-in-progress`: a literal boolean rendered as text, or unevaluated text.
    pub cancel_in_progress: Option<SourceText>,
    /// `queue`: literal `single` or `max`, or unevaluated text.
    pub queue: Option<SourceText>,
}

/// Declared job `environment` (D-033 temporary GHA-derived type); a string form is the name.
#[derive(Debug, PartialEq, Eq)]
pub struct Environment {
    /// Name text, literal or unevaluated.
    pub name: SourceText,
    /// `url` text, literal or unevaluated; never opened.
    pub url: Option<SourceText>,
    /// `deployment`: a literal boolean rendered as text, or unevaluated text.
    pub deployment: Option<SourceText>,
}

/// Declared job `container` (D-033 temporary GHA-derived type); the string form is the image.
/// No image is pulled and no container is created, started, bound, exposed or mounted.
#[derive(Debug, PartialEq, Eq)]
pub struct Container {
    /// Image text, literal or unevaluated.
    pub image: SourceText,
    /// Optional registry credentials, never used.
    pub credentials: Option<ContainerCredentials>,
    /// Optional container environment by name, never bound.
    pub env: Option<BTreeMap<String, StepValue>>,
    /// Optional port entries in source order, never exposed.
    pub ports: Option<Vec<StepValue>>,
    /// Optional volume entries in source order, never mounted.
    pub volumes: Option<Vec<StepValue>>,
    /// Optional `docker create` options text, never passed.
    pub options: Option<SourceText>,
}

/// Declared container registry credentials as source text; never used to log in.
#[derive(Debug, PartialEq, Eq)]
pub struct ContainerCredentials {
    /// Username text, literal or unevaluated.
    pub username: Option<SourceText>,
    /// Password text, literal or unevaluated; hosts should not display it.
    pub password: Option<SourceText>,
}

/// Declared `defaults.run` settings; `defaults: {}` and `defaults: {run: {}}` both have neither.
#[derive(Debug, PartialEq, Eq)]
pub struct RunDefaults {
    /// Optional shell text, never selected or invoked.
    pub shell: Option<SourceText>,
    /// Optional working directory, never resolved.
    pub working_directory: Option<SourceText>,
}

/// A scalar `env`/`with` source value, retaining its YAML type; never evaluated.
#[derive(Debug, PartialEq, Eq)]
pub enum StepValue {
    /// String text, classified as literal or unevaluated.
    Text(SourceText),
    /// Literal boolean.
    Boolean(bool),
    /// YAML number represented as text.
    Number(String),
}

/// Source operation classification, not an execution capability claim.
#[derive(Debug)]
pub enum StepSummaryKind {
    /// A command is present, but is never executed by inspection. `unevaluated` records that the
    /// script holds `${{` text (D-033); the script itself is not retained.
    Run {
        /// The script contains an unevaluated expression fence.
        unevaluated: bool,
    },
    /// An unresolved action reference; inspection never fetches or opens it.
    Uses(String),
}

/// Literal source requirement, not a selected or provisioned runner.
#[derive(Debug, PartialEq, Eq)]
pub enum RunnerRequirement {
    /// One source label in scalar form, literal or unevaluated (D-033 `SourceText`).
    Label(SourceText),
    /// Source list, retaining order, spelling and repetitions; entries may be unevaluated.
    Labels(Vec<SourceText>),
}

use std::collections::BTreeMap;

/// Requested `GITHUB_TOKEN` permissions as declared, not an effective or granted token.
#[derive(Debug, PartialEq, Eq)]
pub enum Permissions {
    /// The `read-all` shorthand.
    ReadAll,
    /// The `write-all` shorthand.
    WriteAll,
    /// Declared scopes in name order; empty for an explicitly empty mapping.
    Scopes(BTreeMap<String, PermissionLevel>),
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

/// One `on` event as declared (D-033 temporary GHA-derived type), never an invocation.
#[derive(Debug, PartialEq, Eq)]
pub enum Trigger {
    /// `workflow_dispatch`, with its declared configuration.
    Manual(ManualTrigger),
    /// Any other pinned-schema event name, declared without configuration.
    Event(String),
    /// `push`, `pull_request` or `pull_request_target` with branch, tag or path filters in
    /// source order, never matched.
    Filtered {
        /// Event name.
        event: String,
        /// Declared filters.
        filters: Vec<EventFilter>,
    },
}

/// One declared event filter (D-033 temporary GHA-derived type), never matched.
#[derive(Debug, PartialEq, Eq)]
pub struct EventFilter {
    /// Filter key, such as `branches` or `paths-ignore`.
    pub name: String,
    /// Patterns as written; a single string is one pattern.
    pub patterns: Vec<String>,
}

/// Source form of the supported manual trigger, never an invocation.
#[derive(Debug, PartialEq, Eq)]
pub enum ManualTrigger {
    /// Bare `workflow_dispatch` (scalar or sequence entry).
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

/// Temporary GHA-derived job strategy (D-033), never expanded (D-012).
#[derive(Debug, PartialEq, Eq)]
pub enum Strategy {
    /// Whole strategy expression, never evaluated.
    Unevaluated(String),
    /// Declared strategy fields.
    Mapping {
        /// Boolean literal or expression text.
        fail_fast: Option<SourceText>,
        /// Number literal or expression text; no positivity constraint is imposed.
        max_parallel: Option<SourceText>,
        /// Matrix as deterministic YAML text, or a whole-matrix expression.
        /// Includes arbitrary nested values and include/exclude entries without combining them.
        matrix: Option<SourceText>,
    },
}
