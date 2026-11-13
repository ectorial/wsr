//! Workflow and job `concurrency`, retained as source values and never applied.

use serde_yaml::Value;
use wsr_types::inspection::{Concurrency, SourceText};

use crate::NON_STRING;
use crate::diagnostic::{Category, Error};

/// Decode `concurrency` at `route`. Both pinned definitions have expression contexts. The string
/// form is the group: any string for workflow-concurrency, a non-empty string for
/// job-concurrency (`job`), as is the mapping's `group` at either level.
pub(crate) fn decode(route: &str, value: Value, job: bool) -> Result<Concurrency, Error> {
    let entries = match value {
        Value::Mapping(entries) => entries,
        value => {
            return Ok(Concurrency {
                group: group(route, value, job)?,
                cancel_in_progress: None,
                queue: None,
            });
        }
    };
    let mut concurrency = Concurrency {
        group: SourceText::Literal(String::new()),
        cancel_in_progress: None,
        queue: None,
    };
    let mut grouped = false;
    for (key, value) in entries {
        let Value::String(key) = key else {
            return Err(Error::invalid(route, "expected a string key"));
        };
        if key.contains("${{") {
            // The pinned reader admits the key, but the converter requires a literal string key.
            return Err(Error::invalid(
                route,
                "expression is unevaluated and not allowed as a concurrency key",
            ));
        }
        let field = format!("{route}.{key}");
        match key.as_str() {
            "group" => {
                concurrency.group = group(&field, value, true)?;
                grouped = true;
            }
            "cancel-in-progress" => {
                concurrency.cancel_in_progress = Some(match value {
                    Value::Bool(flag) => SourceText::Literal(flag.to_string()),
                    Value::String(text) if text.contains("${{") => SourceText::Unevaluated(text),
                    _ => return Err(Error::invalid(field, "expected a boolean")),
                });
            }
            "queue" => {
                concurrency.queue = Some(match value {
                    Value::String(text) if text.contains("${{") => SourceText::Unevaluated(text),
                    Value::String(text) if text == "single" || text == "max" => {
                        SourceText::Literal(text)
                    }
                    _ => return Err(Error::invalid(field, "expected single or max")),
                });
            }
            _ => {
                return Err(Error::invalid(
                    field,
                    format_args!(
                        "unknown field `{key}`, expected `group`, `cancel-in-progress` or `queue`"
                    ),
                )
                .shown_at(route));
            }
        }
    }
    if !grouped {
        return Err(Error::invalid(route, "missing field `group`"));
    }
    Ok(concurrency)
}

/// Group text at `route`; `nonempty` where the pinned definition is a non-empty string.
fn group(route: &str, value: Value, nonempty: bool) -> Result<SourceText, Error> {
    let text = match value {
        Value::String(text) => text,
        // The pinned reader converts these to strings; a null becomes an empty one.
        Value::Null if nonempty => {
            return Err(Error::invalid(route, "expected a nonempty string"));
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            return Err(Error::unsupported(route, NON_STRING));
        }
        _ => return Err(Error::invalid(route, "expected a string or mapping")),
    };
    if text.contains("${{") {
        return Ok(SourceText::Unevaluated(text));
    }
    if text.trim().is_empty() {
        // An empty group fails a non-empty definition; a blank one is schema-valid, not inspected.
        let category = if text.is_empty() && nonempty {
            Category::Invalid
        } else {
            Category::Unsupported
        };
        return Err(Error::new(category, route, "expected a nonempty string"));
    }
    Ok(SourceText::Literal(text))
}
