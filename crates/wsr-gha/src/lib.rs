//! Built-in GitHub Actions source inspection frontend.
//!
//! Temporarily linked for local inspection only. This is not the signed Component provider
//! implementation, an executable canonical IR, or a workflow compatibility guarantee.

use std::collections::BTreeMap;

use serde::Deserialize;
use wsr_types::inspection::{
    JobSummary, RunnerRequirement, SourceText, StepSummary, StepSummaryKind, WorkflowSummary,
};

mod diagnostic;
mod dispatch;
mod needs;
use diagnostic::Fields;
pub use diagnostic::{Category, Error};
use needs::Needs;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Workflow {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "run-name", default, deserialize_with = "present_string")]
    run_name: Option<String>,
    #[serde(rename = "on", default, deserialize_with = "dispatch::deserialize")]
    trigger: Option<wsr_types::inspection::ManualTrigger>,
    #[serde(deserialize_with = "unique_jobs")]
    jobs: BTreeMap<String, Job>,
}

impl diagnostic::Level for Workflow {
    const NAME: &'static str = "Workflow";
    const SCHEMA: &'static [&'static str] = &[
        "concurrency",
        "defaults",
        "description",
        "env",
        "permissions",
    ];
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "runs-on", deserialize_with = "runner_requirement")]
    runner: RunnerRequirement,
    #[serde(default)]
    needs: Needs,
    #[serde(deserialize_with = "steps")]
    steps: Vec<Step>,
}

impl diagnostic::Level for Job {
    const NAME: &'static str = "Job";
    // The union of the pinned schema's job-factory and reusable workflow-job keys.
    const SCHEMA: &'static [&'static str] = &[
        "cancel-timeout-minutes",
        "concurrency",
        "container",
        "continue-on-error",
        "defaults",
        "env",
        "environment",
        "if",
        "outputs",
        "permissions",
        "secrets",
        "services",
        "snapshot",
        "strategy",
        "timeout-minutes",
        "uses",
        "with",
    ];
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    #[serde(default, deserialize_with = "present_string")]
    id: Option<String>,
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(default, deserialize_with = "present_string")]
    run: Option<String>,
    #[serde(default, deserialize_with = "present_string")]
    uses: Option<String>,
}

impl diagnostic::Level for Step {
    const NAME: &'static str = "Step";
    // The union of the pinned schema's run, regular, wait, wait-all, cancel and parallel steps.
    const SCHEMA: &'static [&'static str] = &[
        "background",
        "cancel",
        "continue-on-error",
        "env",
        "if",
        "parallel",
        "shell",
        "timeout-minutes",
        "wait",
        "wait-all",
        "with",
        "working-directory",
    ];
}

/// Inspect a static GHA source subset without performing IO or executing workflow content.
///
/// Unsupported fields are rejected rather than silently omitted. The summary is intentionally
/// separate from the legacy execution model; the same compiler can later be packaged as a Component.
pub fn inspect(raw: &[u8]) -> Result<WorkflowSummary, Error> {
    let Fields(workflow) = diagnostic::decode::<Fields<Workflow>>(raw)?;
    if workflow.jobs.is_empty() {
        return Err(Error::invalid("jobs", "expected at least one job"));
    }
    check_literal(workflow.name.as_deref(), "name")?;
    let run_name = match workflow.run_name {
        Some(text) if text.trim().is_empty() => {
            return Err(Error::invalid("run-name", "expected a nonempty string"));
        }
        Some(text) if text.contains("${{") => Some(SourceText::Unevaluated(text)),
        Some(text) => Some(SourceText::Literal(text)),
        None => None,
    };
    // Identifiers are checked before `needs` resolves against them.
    for id in workflow.jobs.keys() {
        let route = format!("jobs.{id}");
        check_literal(Some(id), &route)?;
        if !id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::invalid(
                route,
                format_args!("invalid job identifier '{}'", id.escape_debug()),
            ));
        }
    }
    needs::validate(&workflow.jobs)?;
    let mut jobs = Vec::new();
    for (id, job) in workflow.jobs {
        if job.steps.is_empty() {
            return Err(Error::invalid(
                format!("jobs.{id}.steps"),
                "expected at least one step",
            ));
        }
        check_literal(job.name.as_deref(), &format!("jobs.{id}.name"))?;
        match &job.runner {
            RunnerRequirement::Label(label) => {
                check_required(label, &format!("jobs.{id}.runs-on"))?
            }
            RunnerRequirement::Labels(labels) => {
                if labels.is_empty() {
                    return Err(Error::invalid(
                        format!("jobs.{id}.runs-on"),
                        "expected at least one label",
                    ));
                }
                for (index, label) in labels.iter().enumerate() {
                    check_required(label, &format!("jobs.{id}.runs-on[{}]", index + 1))?;
                }
            }
        }
        let mut steps = Vec::new();
        let mut step_ids = BTreeMap::new();
        for (index, step) in job.steps.into_iter().enumerate() {
            let location = format!("jobs.{id}.steps[{}]", index + 1);
            if let Some(step_id) = &step.id {
                let field = format!("{location}.id");
                check_required(step_id, &field)?;
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
            check_literal(step.name.as_deref(), &format!("{location}.name"))?;
            if let Some(script) = &step.run {
                check_required(script, &format!("{location}.run"))?;
            }
            if let Some(reference) = &step.uses {
                check_required(reference, &format!("{location}.uses"))?;
            }
            let kind = match (step.run, step.uses) {
                (Some(_), None) => StepSummaryKind::Run,
                (None, Some(reference)) => StepSummaryKind::Uses(reference),
                _ => {
                    return Err(Error::invalid(
                        location,
                        "expected exactly one of run or uses",
                    ));
                }
            };
            steps.push(StepSummary {
                id: step.id,
                name: step.name,
                kind,
            });
        }
        jobs.push(JobSummary {
            id,
            name: job.name,
            runner: job.runner,
            needs: job.needs.into_values(),
            steps,
        });
    }
    Ok(WorkflowSummary {
        name: workflow.name,
        run_name,
        trigger: workflow.trigger,
        jobs,
    })
}

fn check_literal(value: Option<&str>, location: &str) -> Result<(), Error> {
    if value.is_some_and(|value| value.contains("${{")) {
        return Err(Error::unsupported(
            location,
            "expression is unevaluated; dynamic inspection is not supported yet",
        ));
    }
    Ok(())
}

fn check_required(value: &str, location: &str) -> Result<(), Error> {
    check_literal(Some(value), location)?;
    if value.trim().is_empty() {
        return Err(Error::invalid(location, "expected a nonempty string"));
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
            while let Some(id) = map.next_key::<serde_yaml::Value>()? {
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

fn optional_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match Option::<serde_yaml::Value>::deserialize(deserializer)? {
        Some(serde_yaml::Value::String(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(diagnostic::raise_unrouted("expected a string")),
    }
}

fn present_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    string(deserializer).map(Some)
}

fn runner_requirement<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<RunnerRequirement, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(label) => Ok(RunnerRequirement::Label(label)),
        serde_yaml::Value::Sequence(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                serde_yaml::Value::String(label) => Ok(label),
                _ => Err(diagnostic::raise(Error::invalid(
                    format!("runs-on[{}]", index + 1),
                    "expected a string label",
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(RunnerRequirement::Labels),
        // The pinned schema's selector accepts a `group` string and a `labels` string or list.
        serde_yaml::Value::Mapping(selector) => Err(diagnostic::raise(Error::new(
            if !selector.is_empty()
                && selector.iter().all(|(key, value)| match key.as_str() {
                    Some("group") => value.is_string(),
                    Some("labels") => {
                        value.is_string()
                            || value.as_sequence().is_some_and(|labels| {
                                labels.iter().all(serde_yaml::Value::is_string)
                            })
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
