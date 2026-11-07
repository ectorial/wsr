//! Built-in GitHub Actions source inspection frontend.
//!
//! Temporarily linked for local inspection only. This is not the signed Component provider
//! implementation, an executable canonical IR, or a workflow compatibility guarantee.

use std::collections::BTreeMap;

use serde::Deserialize;
use wsr_types::inspection::{JobSummary, StepSummary, StepSummaryKind, WorkflowSummary};

mod needs;
use needs::Needs;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Workflow {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "on", default, deserialize_with = "present_string")]
    trigger: Option<String>,
    #[serde(deserialize_with = "unique_jobs")]
    jobs: BTreeMap<String, Job>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(rename = "runs-on", deserialize_with = "string")]
    runner: String,
    #[serde(default)]
    needs: Needs,
    steps: Vec<Step>,
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

/// Inspect a static GHA source subset without performing IO or executing workflow content.
///
/// Unsupported fields are rejected rather than silently omitted. The summary is intentionally
/// separate from the legacy execution model; the same compiler can later be packaged as a Component.
pub fn inspect(raw: &[u8]) -> anyhow::Result<WorkflowSummary> {
    let workflow: Workflow = serde_yaml::from_slice(raw)?;
    if workflow.jobs.is_empty() {
        anyhow::bail!("jobs: expected at least one job");
    }
    check_literal(workflow.name.as_deref(), "name")?;
    if let Some(trigger) = &workflow.trigger {
        check_literal(Some(trigger), "on")?;
        if trigger != "workflow_dispatch" {
            anyhow::bail!(
                "on: unsupported trigger; only the literal workflow_dispatch is inspected yet"
            );
        }
    }
    needs::validate(&workflow.jobs)?;
    let mut jobs = Vec::new();
    for (id, job) in workflow.jobs {
        if job.steps.is_empty() {
            anyhow::bail!("jobs.{id}.steps: expected at least one step");
        }
        check_literal(Some(&id), "jobs identifier")?;
        if !id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            anyhow::bail!(
                "jobs: invalid job identifier {:?}",
                id.escape_debug().to_string()
            );
        }
        check_literal(job.name.as_deref(), &format!("jobs.{id}.name"))?;
        check_required(&job.runner, &format!("jobs.{id}.runs-on"))?;
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
                    anyhow::bail!(
                        "{field}: invalid step identifier '{}'; expected fewer than 100 ASCII identifier characters and no '__' prefix",
                        step_id.escape_debug()
                    );
                }
                if let Some(first) = step_ids.insert(step_id.clone(), index + 1) {
                    anyhow::bail!(
                        "{field}: duplicate step identifier '{}'; first declared at jobs.{id}.steps[{first}].id",
                        step_id.escape_debug()
                    );
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
                _ => anyhow::bail!(
                    "jobs.{id}.steps[{}]: expected exactly one of run or uses",
                    index + 1
                ),
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
        trigger: workflow.trigger,
        jobs,
    })
}

fn check_literal(value: Option<&str>, location: &str) -> anyhow::Result<()> {
    if value.is_some_and(|value| value.contains("${{")) {
        anyhow::bail!(
            "{location}: expression is unevaluated; dynamic inspection is not supported yet"
        );
    }
    Ok(())
}

fn check_required(value: &str, location: &str) -> anyhow::Result<()> {
    check_literal(Some(value), location)?;
    if value.trim().is_empty() {
        anyhow::bail!("{location}: expected a nonempty string");
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
            while let Some((id, job)) = map.next_entry::<serde_yaml::Value, Job>()? {
                let serde_yaml::Value::String(id) = id else {
                    return Err(serde::de::Error::custom("expected a string job identifier"));
                };
                if jobs.insert(id, job).is_some() {
                    return Err(serde::de::Error::custom("duplicate job identifier"));
                }
            }
            Ok(jobs)
        }
    }
    deserializer.deserialize_map(Jobs)
}

// serde_yaml's String deserializer also accepts numbers and booleans. Source fields must
// retain their actual YAML type rather than silently turn malformed input into a static string.
fn string<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    match serde_yaml::Value::deserialize(deserializer)? {
        serde_yaml::Value::String(value) => Ok(value),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}

fn optional_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    match Option::<serde_yaml::Value>::deserialize(deserializer)? {
        Some(serde_yaml::Value::String(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}

fn present_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    string(deserializer).map(Some)
}
