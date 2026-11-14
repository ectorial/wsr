//! Reusable-workflow call jobs, retained as source text. The referenced workflow is never opened,
//! loaded or resolved, and no input or secret is passed.

use wsr_types::inspection::{CallSecrets, WorkflowCall};

use super::{Category, Error, JobFields, step_inputs};

/// Keys of the pinned `job-factory` that `workflow-job` lacks.
pub(super) const FACTORY_ONLY: &[&str] = &[
    "runs-on",
    "timeout-minutes",
    "cancel-timeout-minutes",
    "continue-on-error",
    "container",
    "services",
    "env",
    "environment",
    "outputs",
    "defaults",
    "steps",
    "snapshot",
];

/// A call job's `uses`, `with` and `secrets`; None for an ordinary job, which may not use them.
pub(super) fn decode(id: &str, job: &JobFields) -> Result<Option<WorkflowCall>, Error> {
    let route = |field: &str| format!("jobs.{id}.{field}");
    // `Job::placement` normally rejects these keys when they are read; the checks below cover a
    // source whose pre-scan failed.
    let Some(uses) = &job.uses else {
        for (field, present) in [
            ("with", job.with.is_some()),
            ("secrets", job.secrets.is_some()),
        ] {
            if present {
                return Err(Error::invalid(
                    route(field),
                    format_args!("`{field}` applies only to a reusable workflow call"),
                ));
            }
        }
        return Ok(None);
    };
    let present = [
        ("runs-on", job.runner.is_some()),
        ("timeout-minutes", job.timeout_minutes.is_some()),
        ("continue-on-error", job.continue_on_error.is_some()),
        ("env", job.env.is_some()),
        ("defaults", job.defaults.is_some()),
        ("outputs", job.outputs.is_some()),
        ("environment", job.environment.is_some()),
        ("steps", job.steps.is_some()),
    ];
    if let Some((field, _)) = present.iter().find(|(_, present)| *present) {
        return Err(Error::invalid(
            route(field),
            format_args!("`{field}` cannot be combined with `uses`"),
        ));
    }
    reference(uses, &route("uses"))?;
    let with = job
        .with
        .clone()
        .map(|value| values(&route("with"), value))
        .transpose()?;
    let secrets = job
        .secrets
        .clone()
        .map(|value| match value {
            serde_yaml::Value::String(text) if text == "inherit" => Ok(CallSecrets::Inherit),
            serde_yaml::Value::Mapping(_) => {
                values(&route("secrets"), value).map(CallSecrets::Values)
            }
            _ => Err(Error::invalid(
                route("secrets"),
                "expected `inherit` or a mapping",
            )),
        })
        .transpose()?;
    Ok(Some(WorkflowCall {
        uses: uses.clone(),
        with,
        secrets,
    }))
}

/// `workflow-job-with` and `workflow-job-secrets-mapping` have no expression context of their own;
/// their values are scalars that may hold unevaluated expressions.
fn values(
    route: &str,
    value: serde_yaml::Value,
) -> Result<std::collections::BTreeMap<String, wsr_types::inspection::StepValue>, Error> {
    let serde_yaml::Value::Mapping(entries) = &value else {
        return Err(Error::invalid(route, "expected a mapping"));
    };
    for key in entries.keys() {
        if let serde_yaml::Value::String(key) = key {
            super::check_literal(Some(key), &format!("{route}.{key}"), Category::Invalid)?;
        }
    }
    step_inputs::decode(route, value)
}

/// The pinned `parseFileReference`: `./path`, `$/path` (self repository) or
/// `owner/repo[/path]@ref`, where the ref is the text between the first and any second `@`.
fn reference(uses: &str, route: &str) -> Result<(), Error> {
    super::check_literal(Some(uses), route, Category::Invalid)?;
    if uses.starts_with("./") {
        return Ok(());
    }
    if uses.starts_with("$/") {
        return Err(Error::unsupported(
            route,
            "self-repository workflow references are not inspected yet",
        ));
    }
    let mut parts = uses.split('@');
    let path = parts.next().unwrap_or_default();
    let version = parts.next().unwrap_or_default();
    if !version.is_empty() && path.split('/').filter(|s| !s.is_empty()).count() >= 2 {
        return Ok(());
    }
    Err(Error::invalid(
        route,
        format_args!(
            "invalid workflow reference '{}'; expected ./path or owner/repo/path@ref",
            uses.escape_debug()
        ),
    ))
}
