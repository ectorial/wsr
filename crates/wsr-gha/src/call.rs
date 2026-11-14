//! Reusable-workflow call jobs, retained as source text. The referenced workflow is never opened,
//! loaded or resolved, and no input or secret is passed.

use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, Visitor};
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

/// For each job of the source, in position order, whether its mapping has a `uses` key, so the job
/// form is known before any of its keys is decoded. Values are skipped, never materialized, and keys
/// are read as the typed decoder reads them. The scan stops at the first failure; a job it did not
/// finish without having read `uses` is absent, and the typed decode fails at the same point or
/// earlier. Aliases can repeat job bodies, so the scan also stops after one job key per source byte;
/// later jobs are then checked after decoding, in key order.
pub(super) fn scan(raw: &[u8]) -> Vec<bool> {
    let mut forms = Vec::new();
    let mut budget = raw.len();
    let _ =
        Scan::Root(&mut forms, &mut budget).deserialize(serde_yaml::Deserializer::from_slice(raw));
    forms
}

/// A scan level with the remaining job-key budget.
enum Scan<'a> {
    Root(&'a mut Vec<bool>, &'a mut usize),
    Jobs(&'a mut Vec<bool>, &'a mut usize),
    Job(&'a mut bool, &'a mut usize),
}

impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a mapping")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        match self {
            Scan::Root(forms, budget) => {
                let mut scanned = false;
                while let Some(Name(name)) = map.next_key()? {
                    // The typed decoder reads the first `jobs` and rejects any repeat.
                    if name.as_deref() == Some("jobs") && !scanned {
                        scanned = true;
                        map.next_value_seed(Scan::Jobs(&mut *forms, &mut *budget))?;
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Scan::Jobs(forms, budget) => {
                while map.next_key::<IgnoredAny>()?.is_some() {
                    let mut call = false;
                    let scanned = map.next_value_seed(Scan::Job(&mut call, &mut *budget));
                    // A `uses` key already read decides the form even if a later key fails.
                    if scanned.is_ok() || call {
                        forms.push(call);
                    }
                    scanned?;
                }
            }
            Scan::Job(call, budget) => {
                while let Some(Name(name)) = map.next_key()? {
                    *budget = budget
                        .checked_sub(1)
                        .ok_or_else(|| serde::de::Error::custom("scan budget exhausted"))?;
                    *call |= name.as_deref() == Some("uses");
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

/// A mapping key read as a struct field identifier; None for a key that is not a string.
struct Name(Option<String>);

impl<'de> serde::Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Identifier;
        impl Visitor<'_> for Identifier {
            type Value = Name;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an identifier")
            }

            fn visit_str<E: serde::de::Error>(self, name: &str) -> Result<Name, E> {
                Ok(Name(Some(name.to_owned())))
            }

            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Name, E> {
                Ok(Name(None))
            }

            fn visit_bytes<E: serde::de::Error>(self, _: &[u8]) -> Result<Name, E> {
                Ok(Name(None))
            }
        }
        deserializer.deserialize_identifier(Identifier)
    }
}

/// A call job's `uses`, `with` and `secrets`; None for an ordinary job, which may not use them.
pub(super) fn decode(id: &str, job: &JobFields) -> Result<Option<WorkflowCall>, Error> {
    let route = |field: &str| format!("jobs.{id}.{field}");
    // `Job::placement` rejects these keys when they are read; the checks below cover a job the
    // scan did not finish.
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
        (
            "cancel-timeout-minutes",
            job.cancel_timeout_minutes.is_some(),
        ),
        ("continue-on-error", job.continue_on_error.is_some()),
        ("env", job.env.is_some()),
        ("defaults", job.defaults.is_some()),
        ("outputs", job.outputs.is_some()),
        ("environment", job.environment.is_some()),
        ("container", job.container.is_some()),
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
