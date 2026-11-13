//! Workflow and job `defaults.run`, retained as source values and never applied.

use wsr_types::inspection::{RunDefaults, SourceText};

use crate::diagnostic::{Category, Error};

const EXPRESSION: &str = "expression is unevaluated; dynamic inspection is not supported yet";

/// Decode `defaults` at `route`. `run` is the category the pinned schema gives an expression in
/// `defaults.run` and its values: `Unsupported` for job-defaults-run (it has an expression context),
/// `Invalid` for workflow-defaults-run (it has none). `defaults` itself never has one.
pub(crate) fn decode(
    route: &str,
    value: serde_yaml::Value,
    run: Category,
) -> Result<RunDefaults, Error> {
    let mut defaults = RunDefaults {
        shell: None,
        working_directory: None,
    };
    for (key, value) in entries(route, value, Category::Invalid, "defaults")? {
        if key != "run" {
            return Err(unknown(route, &key, "`run`"));
        }
        let route = format!("{route}.run");
        for (key, value) in entries(&route, value, run, "workflow defaults")? {
            let slot = match key.as_str() {
                "shell" => &mut defaults.shell,
                "working-directory" => &mut defaults.working_directory,
                _ => return Err(unknown(&route, &key, "`shell` or `working-directory`")),
            };
            let field = format!("{route}.{key}");
            let serde_yaml::Value::String(text) = value else {
                return Err(Error::invalid(field, "expected a string"));
            };
            *slot = Some(if text.contains("${{") {
                if run == Category::Invalid {
                    return Err(Error::invalid(field, not_allowed("workflow defaults")));
                }
                SourceText::Unevaluated(text)
            } else if text.trim().is_empty() {
                // The pinned `shell` and `working-directory` require a nonempty string. A blank
                // shell is rejected by the runner, as for a step; a blank directory is
                // schema-valid but not inspected, as for a step.
                let category = if text.is_empty() || key == "shell" {
                    Category::Invalid
                } else {
                    Category::Unsupported
                };
                return Err(Error::new(category, field, "expected a nonempty string"));
            } else {
                SourceText::Literal(text)
            });
        }
    }
    Ok(defaults)
}

/// The string-keyed entries of the mapping at `route`. `expression` is the category of an
/// expression in place of the mapping; `place` names where an `Invalid` one is not allowed.
fn entries(
    route: &str,
    value: serde_yaml::Value,
    expression: Category,
    place: &str,
) -> Result<Vec<(String, serde_yaml::Value)>, Error> {
    match value {
        serde_yaml::Value::Mapping(entries) => entries
            .into_iter()
            .map(|(key, value)| match key {
                serde_yaml::Value::String(key) => Ok((key, value)),
                _ => Err(Error::invalid(route, "expected a string key")),
            })
            .collect(),
        serde_yaml::Value::String(text) if text.contains("${{") => Err(match expression {
            Category::Unsupported => Error::unsupported(route, EXPRESSION),
            Category::Invalid => Error::invalid(route, not_allowed(place)),
        }),
        _ => Err(Error::invalid(route, "expected a mapping")),
    }
}

fn unknown(route: &str, key: &str, expected: &str) -> Error {
    Error::invalid(
        route,
        format_args!("unknown field `{key}`, expected {expected}"),
    )
}

fn not_allowed(place: &str) -> String {
    format!("expression is unevaluated and not allowed in {place}")
}
