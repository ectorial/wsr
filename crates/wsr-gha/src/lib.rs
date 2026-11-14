//! Built-in GitHub Actions source inspection frontend.
//!
//! Temporarily linked for local inspection only. This is not the signed Component provider
//! implementation, an executable canonical IR, or a workflow compatibility guarantee.

use std::collections::BTreeMap;

use serde::Deserialize;
use wsr_types::inspection::{
    JobSummary, RunnerRequirement, SourceText, StepSummary, StepSummaryKind, WorkflowSummary,
};

mod call;
mod concurrency;
mod container;
mod defaults;
mod diagnostic;
mod dispatch;
mod environment;
mod needs;
mod outputs;
mod permissions;
mod step_inputs;
use diagnostic::Fields;
pub use diagnostic::{Category, Error};
use needs::Needs;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Workflow {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "run-name", default, deserialize_with = "converted_string")]
    run_name: Option<String>,
    #[serde(rename = "on", deserialize_with = "dispatch::deserialize")]
    triggers: Vec<wsr_types::inspection::Trigger>,
    #[serde(default, deserialize_with = "permissions::deserialize")]
    permissions: Option<wsr_types::inspection::Permissions>,
    #[serde(default, deserialize_with = "present_value")]
    env: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    defaults: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    concurrency: Option<serde_yaml::Value>,
    #[serde(deserialize_with = "unique_jobs")]
    jobs: BTreeMap<String, Job>,
}

impl diagnostic::Level for Workflow {
    const NAME: &'static str = "Workflow";
    const SCHEMA: &'static [&'static str] = &["description"];
}

/// The pinned `job` is `job-factory` or `workflow-job`; `runs-on` and `steps` are required unless
/// `uses` selects a reusable-workflow call.
#[derive(Deserialize)]
#[serde(try_from = "JobFields")]
struct Job(JobFields);

impl TryFrom<JobFields> for Job {
    type Error = &'static str;

    fn try_from(job: JobFields) -> Result<Self, Self::Error> {
        if job.uses.is_none() {
            // The derived decoder's wording and location for an ordinary job.
            if job.runner.is_none() {
                return Err("missing field `runs-on`");
            }
            if job.steps.is_none() {
                return Err("missing field `steps`");
            }
        }
        Ok(Self(job))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JobFields {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "runs-on", default, deserialize_with = "present_runner")]
    runner: Option<RunnerRequirement>,
    #[serde(default)]
    needs: Needs,
    #[serde(rename = "if", default, deserialize_with = "condition")]
    condition: Option<String>,
    #[serde(default, deserialize_with = "permissions::deserialize")]
    permissions: Option<wsr_types::inspection::Permissions>,
    #[serde(
        rename = "timeout-minutes",
        default,
        deserialize_with = "timeout_minutes"
    )]
    timeout_minutes: Option<SourceText>,
    #[serde(
        rename = "continue-on-error",
        default,
        deserialize_with = "continue_on_error"
    )]
    continue_on_error: Option<SourceText>,
    #[serde(default, deserialize_with = "present_value")]
    env: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    defaults: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    concurrency: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    outputs: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    environment: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "container::deserialize")]
    container: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_steps")]
    steps: Option<Vec<Step>>,
    #[serde(default, deserialize_with = "present_string")]
    uses: Option<String>,
    #[serde(default, deserialize_with = "present_value")]
    with: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    secrets: Option<serde_yaml::Value>,
}

thread_local! {
    // Per job, in source position, whether its mapping has `uses` (`call::scan`).
    static CALLS: std::cell::RefCell<Vec<bool>> = const { std::cell::RefCell::new(Vec::new()) };
    // Whether the job being decoded is a reusable-workflow call; None if the scan did not finish it.
    static CALL: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

impl diagnostic::Level for Job {
    const NAME: &'static str = "Job";

    fn placement(key: &str) -> Option<String> {
        // A job the scan did not finish is a call from `uses` on, so later keys are placed.
        if CALL.get().is_none() && key == "uses" {
            CALL.set(Some(true));
            return None;
        }
        match CALL.get()? {
            true if call::FACTORY_ONLY.contains(&key) => {
                Some(format!("`{key}` cannot be combined with `uses`"))
            }
            false if matches!(key, "with" | "secrets") => {
                Some(format!("`{key}` applies only to a reusable workflow call"))
            }
            _ => None,
        }
    }

    // The union of the pinned schema's job-factory and reusable workflow-job keys.
    const SCHEMA: &'static [&'static str] =
        &["cancel-timeout-minutes", "services", "snapshot", "strategy"];
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    #[serde(default, deserialize_with = "step_id")]
    id: Option<String>,
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "if", default, deserialize_with = "condition")]
    condition: Option<String>,
    #[serde(default, deserialize_with = "converted_string")]
    run: Option<String>,
    #[serde(default, deserialize_with = "present_string")]
    shell: Option<String>,
    #[serde(
        rename = "working-directory",
        default,
        deserialize_with = "converted_string"
    )]
    working_directory: Option<String>,
    // Interpreted after the step-kind checks, so a misplaced field reports its placement first.
    #[serde(default, deserialize_with = "present_value")]
    env: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_value")]
    with: Option<serde_yaml::Value>,
    #[serde(default, deserialize_with = "present_string")]
    uses: Option<String>,
    #[serde(
        rename = "continue-on-error",
        default,
        deserialize_with = "continue_on_error"
    )]
    continue_on_error: Option<SourceText>,
    #[serde(
        rename = "timeout-minutes",
        default,
        deserialize_with = "timeout_minutes"
    )]
    timeout_minutes: Option<SourceText>,
}

impl diagnostic::Level for Step {
    const NAME: &'static str = "Step";
    // `parallel` is the only pinned step key beyond the run and regular steps that the reader
    // accepts by default; it rejects background, wait, wait-all and cancel unless
    // `allowBackgroundSteps` is set, and the default is false.
    const SCHEMA: &'static [&'static str] = &["parallel"];
}

/// Inspect a static GHA source subset without performing IO or executing workflow content.
///
/// Unsupported fields are rejected rather than silently omitted. The summary is intentionally
/// separate from the legacy execution model; the same compiler can later be packaged as a Component.
pub fn inspect(raw: &[u8]) -> Result<WorkflowSummary, Error> {
    CALLS.set(call::scan(raw));
    CALL.set(None);
    let decoded = diagnostic::decode::<Fields<Workflow>>(raw);
    CALLS.take();
    CALL.set(None);
    let Fields(workflow) = decoded?;
    if workflow.jobs.is_empty() {
        return Err(Error::invalid("jobs", "expected at least one job"));
    }
    check_literal(workflow.name.as_deref(), "name", Category::Invalid)?;
    let run_name = match workflow.run_name {
        Some(text) if text.trim().is_empty() => {
            return Err(Error::invalid("run-name", "expected a nonempty string"));
        }
        Some(text) if text.contains("${{") => Some(SourceText::Unevaluated(text)),
        Some(text) => Some(SourceText::Literal(text)),
        None => None,
    };
    // The pinned workflow-env has an expression context; workflow-defaults(-run) has none.
    let env = workflow
        .env
        .map(|value| step_inputs::decode("env", value))
        .transpose()?;
    let defaults = workflow
        .defaults
        .map(|value| defaults::decode("defaults", value, Category::Invalid))
        .transpose()?;
    let concurrency = workflow
        .concurrency
        .map(|value| concurrency::decode("concurrency", value, false))
        .transpose()?;
    needs::validate(&workflow.jobs)?;
    let mut jobs = Vec::new();
    for (id, Job(job)) in workflow.jobs {
        let call = call::decode(&id, &job)?;
        if job.steps.as_ref().is_some_and(Vec::is_empty) {
            return Err(Error::invalid(
                format!("jobs.{id}.steps"),
                "expected at least one step",
            ));
        }
        // string-strategy-context (`name`) and runs-on have expression contexts: `${{` text is
        // kept unevaluated (B-24).
        let labels = match &job.runner {
            None => Vec::new(),
            Some(RunnerRequirement::Label(label)) => vec![(label, format!("jobs.{id}.runs-on"))],
            Some(RunnerRequirement::Labels(labels)) => {
                if labels.is_empty() {
                    return Err(Error::invalid(
                        format!("jobs.{id}.runs-on"),
                        "expected at least one label",
                    ));
                }
                labels
                    .iter()
                    .enumerate()
                    .map(|(index, label)| (label, format!("jobs.{id}.runs-on[{}]", index + 1)))
                    .collect()
            }
        };
        for (label, route) in labels {
            if written(label).trim().is_empty() {
                return Err(Error::invalid(route, "expected a nonempty string"));
            }
        }
        // job-env and job-defaults-run have expression contexts; job-defaults has none.
        let env = job
            .env
            .map(|value| step_inputs::decode(&format!("jobs.{id}.env"), value))
            .transpose()?;
        let defaults = job
            .defaults
            .map(|value| {
                defaults::decode(&format!("jobs.{id}.defaults"), value, Category::Unsupported)
            })
            .transpose()?;
        let concurrency = job
            .concurrency
            .map(|value| concurrency::decode(&format!("jobs.{id}.concurrency"), value, true))
            .transpose()?;
        let outputs = job
            .outputs
            .map(|value| outputs::decode(&format!("jobs.{id}.outputs"), value))
            .transpose()?;
        let environment = job
            .environment
            .map(|value| environment::decode(&format!("jobs.{id}.environment"), value))
            .transpose()?;
        let container = job
            .container
            .map(|value| container::decode(&format!("jobs.{id}.container"), value))
            .transpose()?;
        let mut steps = Vec::new();
        let mut step_ids = BTreeMap::new();
        for (index, step) in job.steps.into_iter().flatten().enumerate() {
            let location = format!("jobs.{id}.steps[{}]", index + 1);
            if let Some(step_id) = &step.id {
                let field = format!("{location}.id");
                check_required(step_id, &field, Category::Invalid)?;
                if step_id.len() >= 100
                    || step_id.starts_with("__")
                    || !step_id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    || !step_id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    return Err(Error::invalid(
                        field,
                        format_args!(
                            "invalid step identifier '{}'; expected fewer than 100 ASCII identifier characters and no '__' prefix",
                            step_id.escape_debug()
                        ),
                    ));
                }
                if let Some(first) = step_ids.insert(step_id.clone(), index + 1) {
                    return Err(Error::invalid(
                        field,
                        format_args!(
                            "duplicate step identifier '{}'; first declared at jobs.{id}.steps[{first}].id",
                            step_id.escape_debug()
                        ),
                    ));
                }
            }
            if let Some(script) = &step.run {
                let field = format!("{location}.run");
                // string-steps-context allows an empty script; WSR does not inspect one.
                if script.trim().is_empty() {
                    return Err(Error::unsupported(field, "expected a nonempty string"));
                }
            }
            if let Some(reference) = &step.uses {
                check_required(reference, &format!("{location}.uses"), Category::Invalid)?;
            }
            let kind = match (step.run, step.uses) {
                // string-steps-context: `${{` text is valid GHA, kept unevaluated (B-28); the script
                // itself is not retained.
                (Some(script), None) => StepSummaryKind::Run {
                    unevaluated: script.contains("${{"),
                },
                (None, Some(reference)) => {
                    check_action_reference(&reference, &format!("{location}.uses"))?;
                    StepSummaryKind::Uses(reference)
                }
                _ => {
                    return Err(Error::invalid(
                        location,
                        "expected exactly one of run or uses",
                    ));
                }
            };
            if let Some(shell) = &step.shell {
                let field = format!("{location}.shell");
                // The pinned schema gives `shell` no expression context and only run steps a shell.
                if shell.contains("${{") {
                    return Err(Error::invalid(
                        field,
                        "expression is unevaluated and not allowed in shell",
                    ));
                }
                if shell.trim().is_empty() {
                    return Err(Error::invalid(field, "expected a nonempty string"));
                }
                if !matches!(kind, StepSummaryKind::Run { .. }) {
                    return Err(Error::invalid(field, "shell applies only to run steps"));
                }
                check_shell(shell, &field)?;
            }
            let working_directory = match step.working_directory {
                Some(_) if !matches!(kind, StepSummaryKind::Run { .. }) => {
                    return Err(Error::invalid(
                        format!("{location}.working-directory"),
                        "working-directory applies only to run steps",
                    ));
                }
                // string-steps-context allows an empty directory; WSR does not inspect one.
                Some(text) if text.trim().is_empty() => {
                    return Err(Error::unsupported(
                        format!("{location}.working-directory"),
                        "expected a nonempty string",
                    ));
                }
                Some(text) if text.contains("${{") => Some(SourceText::Unevaluated(text)),
                Some(text) => Some(SourceText::Literal(text)),
                None => None,
            };
            if step.with.is_some() && matches!(kind, StepSummaryKind::Run { .. }) {
                return Err(Error::invalid(
                    format!("{location}.with"),
                    "with applies only to uses steps",
                ));
            }
            let env = step
                .env
                .map(|value| step_inputs::decode(&format!("{location}.env"), value))
                .transpose()?;
            let with = step
                .with
                .map(|value| step_inputs::decode(&format!("{location}.with"), value))
                .transpose()?;
            steps.push(StepSummary {
                id: step.id,
                name: step.name.map(source_text),
                condition: step.condition,
                continue_on_error: step.continue_on_error,
                timeout_minutes: step.timeout_minutes,
                shell: step.shell,
                working_directory,
                env,
                with,
                kind,
            });
        }
        jobs.push(JobSummary {
            id,
            name: job.name.map(source_text),
            runner: job.runner,
            call,
            needs: job.needs.into_values(),
            condition: job.condition,
            permissions: job.permissions,
            timeout_minutes: job.timeout_minutes,
            continue_on_error: job.continue_on_error,
            env,
            defaults,
            concurrency,
            outputs,
            environment,
            container,
            steps,
        });
    }
    Ok(WorkflowSummary {
        name: workflow.name,
        run_name,
        triggers: workflow.triggers,
        permissions: workflow.permissions,
        env,
        defaults,
        concurrency,
        jobs,
    })
}

/// Text as written: unevaluated when it holds `${{`, literal otherwise. Never evaluated.
fn source_text(text: String) -> SourceText {
    if text.contains("${{") {
        SourceText::Unevaluated(text)
    } else {
        SourceText::Literal(text)
    }
}

fn written(text: &SourceText) -> &str {
    match text {
        SourceText::Literal(text) | SourceText::Unevaluated(text) => text,
    }
}

/// Reject an unevaluated expression. `expression` is the category the pinned schema gives it:
/// `Unsupported` where the field has an expression context, `Invalid` where it has none.
fn check_literal(value: Option<&str>, location: &str, expression: Category) -> Result<(), Error> {
    if value.is_some_and(|value| value.contains("${{")) {
        return Err(Error::new(
            expression,
            location,
            "expression is unevaluated; dynamic inspection is not supported yet",
        ));
    }
    Ok(())
}

fn check_required(value: &str, location: &str, expression: Category) -> Result<(), Error> {
    check_literal(Some(value), location, expression)?;
    if value.trim().is_empty() {
        return Err(Error::invalid(location, "expected a nonempty string"));
    }
    Ok(())
}

/// The runner splits a shell at its first space into a command and arguments, and rejects one
/// without `{0}` unless the command is a built-in shell (case-insensitive) with no arguments.
pub(crate) fn check_shell(shell: &str, location: &str) -> Result<(), Error> {
    const BUILT_IN: &[&str] = &["bash", "pwsh", "python", "sh", "cmd", "powershell"];
    let (command, arguments) = shell.split_once(' ').unwrap_or((shell, ""));
    if shell.contains("{0}")
        || (arguments.trim().is_empty()
            && BUILT_IN
                .iter()
                .any(|built_in| command.eq_ignore_ascii_case(built_in)))
    {
        return Ok(());
    }
    Err(Error::invalid(
        location,
        format_args!(
            "unknown shell '{}'; expected bash, pwsh, python, sh, cmd, powershell or a command containing {{0}}",
            shell.escape_debug()
        ),
    ))
}

/// Accept a local `./` or `.\` path, a `docker://` image or `owner/repo[/path]@ref`, whose path
/// the runner splits on either separator, dropping empty segments. Nothing is resolved.
fn check_action_reference(reference: &str, location: &str) -> Result<(), Error> {
    if reference.starts_with("./") || reference.starts_with(".\\") {
        return Ok(());
    }
    if reference.starts_with("$/") {
        // The pinned parser reads `$/` as a self-repository reference.
        return Err(Error::unsupported(
            location,
            "self-repository action references are not inspected yet",
        ));
    }
    let valid = match reference.strip_prefix("docker://") {
        Some(image) => !image.is_empty(),
        None => reference.split_once('@').is_some_and(|(path, version)| {
            !version.is_empty()
                && !version.contains('@')
                && path
                    .split(['/', '\\'])
                    .filter(|segment| !segment.is_empty())
                    .count()
                    >= 2
        }),
    };
    if valid {
        return Ok(());
    }
    Err(Error::invalid(
        location,
        format_args!(
            "invalid action reference '{}'; expected ./path, docker://image or owner/repo[/path]@ref",
            reference.escape_debug()
        ),
    ))
}

/// A job ID has no schema expression context and must be an identifier; the pinned builder also
/// rejects 100 or more characters and a `__` prefix.
fn check_job_id(id: &str) -> Result<(), Error> {
    let route = format!("jobs.{id}");
    check_literal(Some(id), &route, Category::Invalid)?;
    if id.len() >= 100
        || id.starts_with("__")
        || !id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::invalid(
            route,
            format_args!("invalid job identifier '{}'", id.escape_debug()),
        ));
    }
    Ok(())
}

fn unique_jobs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Job>, D::Error> {
    struct Jobs;
    impl<'de> serde::de::Visitor<'de> for Jobs {
        type Value = BTreeMap<String, Job>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a mapping of unique job identifiers")
        }

        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut jobs = BTreeMap::new();
            let mut spellings = BTreeMap::new();
            let mut position = 0;
            while let Some(id) = map.next_key::<serde_yaml::Value>()? {
                // The identifier is validated before its body is decoded, so it cannot forge
                // the route of a body diagnostic.
                if let serde_yaml::Value::String(id) = &id {
                    check_job_id(id).map_err(diagnostic::raise_unlocated)?;
                }
                CALL.set(CALLS.with_borrow(|calls| calls.get(position).copied()));
                position += 1;
                let Fields(job) = map.next_value::<Fields<Job>>().inspect_err(|_| match &id {
                    serde_yaml::Value::String(id) => diagnostic::within(&format!("jobs.{id}")),
                    _ => diagnostic::within("jobs"),
                })?;
                let serde_yaml::Value::String(id) = id else {
                    return Err(diagnostic::raise(Error::invalid(
                        "jobs",
                        "expected a string job identifier",
                    )));
                };
                if jobs.contains_key(&id) {
                    return Err(diagnostic::raise(Error::invalid(
                        format!("jobs.{id}"),
                        format_args!("duplicate job identifier '{}'", id.escape_debug()),
                    )));
                }
                // The pinned reader also rejects identifiers repeated ignoring case.
                if let Some(first) = spellings.insert(id.to_uppercase(), id.clone()) {
                    return Err(diagnostic::raise(Error::invalid(
                        format!("jobs.{id}"),
                        format_args!(
                            "duplicate job identifier '{}'; already defined as '{}'",
                            id.escape_debug(),
                            first.escape_debug()
                        ),
                    )));
                }
                jobs.insert(id, job);
            }
            Ok(jobs)
        }
    }
    deserializer.deserialize_map(Jobs)
}

fn steps<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<Step>, D::Error> {
    struct Steps;
    impl<'de> serde::de::Visitor<'de> for Steps {
        type Value = Vec<Step>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<S: serde::de::SeqAccess<'de>>(
            self,
            mut seq: S,
        ) -> Result<Self::Value, S::Error> {
            let mut steps = Vec::new();
            while let Some(Fields(step)) = seq
                .next_element::<Fields<Step>>()
                .inspect_err(|_| diagnostic::within(&format!("steps[{}]", steps.len() + 1)))?
            {
                steps.push(step);
            }
            Ok(steps)
        }
    }
    deserializer.deserialize_seq(Steps)
}

// serde_yaml's String deserializer also accepts numbers and booleans. Source fields must
// retain their actual YAML type rather than silently turn malformed input into a static string.
fn string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(value) => Ok(value),
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

/// The pinned reader converts an unmatched null, boolean or number to a string (a null to "") and
/// validates that string. A conversion the schema accepts is valid GHA that WSR does not inspect.
pub(crate) const NON_STRING: &str = "non-string value is not inspected yet";

/// A string field whose null, boolean and number literals the pinned reader converts to strings
/// that the schema accepts. The converted text is never rendered.
fn converted_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(value) => Ok(Some(value)),
        serde_yaml::Value::Null | serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_) => Err(
            diagnostic::raise_unrouted_as(Category::Unsupported, NON_STRING),
        ),
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

/// Like [`converted_string`], except that a null is an absent value.
fn optional_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match Option::<serde_yaml::Value>::deserialize(deserializer)? {
        Some(serde_yaml::Value::String(value)) => Ok(Some(value)),
        None => Ok(None),
        Some(serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_)) => Err(
            diagnostic::raise_unrouted_as(Category::Unsupported, NON_STRING),
        ),
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

/// A step ID. Only a boolean converts to a valid identifier (`true`, `false`); a number does not
/// start with a letter and a null is empty.
fn step_id<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(value) => Ok(Some(value)),
        serde_yaml::Value::Bool(_) => Err(diagnostic::raise_unrouted_as(
            Category::Unsupported,
            NON_STRING,
        )),
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

fn present_runner<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<RunnerRequirement>, D::Error> {
    runner_requirement(deserializer).map(Some)
}

fn present_steps<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<Step>>, D::Error> {
    steps(deserializer).map(Some)
}

fn present_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    string(deserializer).map(Some)
}

fn present_value<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<serde_yaml::Value>, D::Error> {
    serde_yaml::Value::deserialize(deserializer).map(Some)
}

// The pinned job-if and step-if are strings with expression contexts, implicitly expressions
// whether bare or fenced; inspection keeps the text and never parses or evaluates it.
fn condition<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        // The pinned reader admits a null or empty condition, which GitHub treats as `success()`.
        serde_yaml::Value::Null => Err(diagnostic::raise_unrouted_as(
            Category::Unsupported,
            "expected a nonempty condition",
        )),
        serde_yaml::Value::String(text) if text.trim().is_empty() => Err(
            diagnostic::raise_unrouted_as(Category::Unsupported, "expected a nonempty condition"),
        ),
        serde_yaml::Value::String(text) => Ok(Some(text)),
        // The pinned reader converts number and boolean literals to condition strings.
        serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_) => {
            Err(diagnostic::raise_unrouted_as(
                Category::Unsupported,
                "non-string condition is not inspected yet",
            ))
        }
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

fn timeout_minutes<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SourceText>, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(text) if text.contains("${{") => {
            Ok(Some(SourceText::Unevaluated(text)))
        }
        serde_yaml::Value::Number(minutes) if minutes.as_f64().is_some_and(|m| m.is_finite()) => {
            // number-strategy-context has no minimum, so a non-positive number is valid GHA that
            // WSR's inspection subset does not accept.
            if minutes.as_f64().is_some_and(|m| m > 0.0) {
                Ok(Some(SourceText::Literal(minutes.to_string())))
            } else {
                Err(diagnostic::raise_unrouted_as(
                    Category::Unsupported,
                    "expected a positive number",
                ))
            }
        }
        _ => Err(diagnostic::raise_unrouted("expected a positive number")),
    }
}

// boolean-strategy-context and step-continue-on-error take a boolean or an expression. The pinned
// reader's string conversion of a number or null cannot yield a boolean, so those are invalid.
fn continue_on_error<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SourceText>, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::Bool(flag) => Ok(Some(SourceText::Literal(flag.to_string()))),
        serde_yaml::Value::String(text) if text.contains("${{") => {
            Ok(Some(SourceText::Unevaluated(text)))
        }
        _ => Err(diagnostic::raise_unrouted("expected a boolean")),
    }
}

fn runner_requirement<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<RunnerRequirement, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(label) => Ok(RunnerRequirement::Label(source_text(label))),
        serde_yaml::Value::Sequence(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                serde_yaml::Value::String(label) => Ok(source_text(label)),
                serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_) => {
                    Err(diagnostic::raise(Error::unsupported(
                        format!("runs-on[{}]", index + 1),
                        NON_STRING,
                    )))
                }
                _ => Err(diagnostic::raise(Error::invalid(
                    format!("runs-on[{}]", index + 1),
                    "expected a string label",
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(RunnerRequirement::Labels),
        serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_) => {
            Err(diagnostic::raise(Error::unsupported("runs-on", NON_STRING)))
        }
        // The pinned schema's selector accepts a `group` string and a `labels` string or list;
        // the reader converts a number or boolean in either to a string.
        serde_yaml::Value::Mapping(selector) => Err(diagnostic::raise(Error::new(
            if !selector.is_empty()
                && selector.iter().all(|(key, value)| match key.as_str() {
                    Some("group") => is_label(value),
                    Some("labels") => {
                        is_label(value)
                            || value
                                .as_sequence()
                                .is_some_and(|labels| labels.iter().all(is_label))
                    }
                    _ => false,
                })
            {
                Category::Unsupported
            } else {
                Category::Invalid
            },
            "runs-on",
            "unsupported runner-selector mapping",
        ))),
        _ => Err(diagnostic::raise(Error::invalid(
            "runs-on",
            "expected a string or list of strings",
        ))),
    }
}

fn is_label(value: &serde_yaml::Value) -> bool {
    matches!(
        value,
        serde_yaml::Value::String(_) | serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_)
    )
}
