use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

use super::{Error, diagnostic};

/// Preserve source shape so diagnostics can distinguish a scalar from a list entry.
#[derive(Default)]
pub(super) enum Needs {
    #[default]
    Missing,
    Scalar(String),
    List(Vec<String>),
}

impl Needs {
    fn location(&self, id: &str, index: usize) -> String {
        match self {
            Self::List(_) => format!("jobs.{id}.needs[{}]", index + 1),
            _ => format!("jobs.{id}.needs"),
        }
    }
    fn values(&self) -> &[String] {
        match self {
            Self::Missing => &[],
            Self::Scalar(value) => std::slice::from_ref(value),
            Self::List(values) => values,
        }
    }

    pub(super) fn into_values(self) -> Vec<String> {
        match self {
            Self::Missing => Vec::new(),
            Self::Scalar(value) => vec![value],
            Self::List(values) => values,
        }
    }
}

pub(super) fn validate(jobs: &std::collections::BTreeMap<String, super::Job>) -> Result<(), Error> {
    for (id, job) in jobs {
        for (index, target) in job.needs.values().iter().enumerate() {
            let location = job.needs.location(id, index);
            super::check_required(target, &location)?;
            if !jobs.contains_key(target) {
                return Err(Error::invalid(
                    location,
                    format_args!("unknown job '{}'", target.escape_debug()),
                ));
            }
        }
    }
    for (id, job) in jobs {
        for (index, target) in job.needs.values().iter().enumerate() {
            if target == id {
                return Err(Error::invalid(
                    job.needs.location(id, index),
                    format_args!("job '{}' depends on itself", id.escape_debug()),
                ));
            }
        }
    }
    // An explicit DFS stack bounds traversal memory by graph size rather than native stack depth.
    // Completed nodes make diamonds and repeated edges harmless; active nodes identify real cycles.
    enum Visit {
        Active(usize),
        Complete,
    }
    let mut visits = std::collections::BTreeMap::new();
    for root in jobs.keys().map(String::as_str) {
        if matches!(visits.get(root), Some(Visit::Complete)) {
            continue;
        }
        let mut stack = vec![(root, 0)];
        visits.insert(root, Visit::Active(0));
        while let Some((current, next)) = stack.last_mut() {
            let id = *current;
            let needs = &jobs[id].needs;
            let Some(target) = needs.values().get(*next) else {
                visits.insert(id, Visit::Complete);
                stack.pop();
                continue;
            };
            let index = *next;
            *next += 1;
            match visits.get(target.as_str()) {
                Some(Visit::Complete) => (),
                Some(Visit::Active(start)) => {
                    let cycle = stack[*start..]
                        .iter()
                        .map(|(id, _)| id.escape_debug().to_string())
                        .chain(std::iter::once(target.escape_debug().to_string()))
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    return Err(Error::invalid(
                        needs.location(id, index),
                        format_args!("dependency cycle: {cycle}"),
                    ));
                }
                None => {
                    visits.insert(target.as_str(), Visit::Active(stack.len()));
                    stack.push((target.as_str(), 0));
                }
            }
        }
    }
    Ok(())
}

impl<'de> Deserialize<'de> for Needs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Value::deserialize(deserializer)? {
            Value::String(value) => Ok(Self::Scalar(value)),
            Value::Sequence(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| match value {
                    Value::String(value) => Ok(value),
                    _ => Err(diagnostic::raise(Error::invalid(
                        format!("needs[{}]", index + 1),
                        "expected a string",
                    ))),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Self::List),
            _ => Err(diagnostic::raise(Error::invalid(
                "needs",
                "expected a string or a sequence of strings",
            ))),
        }
    }
}
