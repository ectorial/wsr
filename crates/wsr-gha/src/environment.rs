//! Job `environment`, retained as source values; never selected and no protection rule evaluated.

use serde_yaml::Value;
use wsr_types::inspection::{Environment, SourceText};

use crate::NON_STRING;
use crate::diagnostic::Error;

/// Decode `environment` at `route`. The pinned `job-environment` has an expression context that
/// its values inherit; its string form and mapping `name` are any string, and `url` a string.
pub(crate) fn decode(route: &str, value: Value) -> Result<Environment, Error> {
    let entries = match value {
        Value::Mapping(entries) => entries,
        value => {
            return Ok(Environment {
                name: text(route, value)?,
                url: None,
                deployment: None,
            });
        }
    };
    let mut name = None;
    let mut environment = Environment {
        name: SourceText::Literal(String::new()),
        url: None,
        deployment: None,
    };
    for (key, value) in entries {
        let key = match key {
            // The pinned converter asserts a literal string key before any expression guard.
            Value::String(key) if !key.contains("${{") => key,
            _ => return Err(Error::invalid(route, "expected a literal string key")),
        };
        let field = format!("{route}.{key}");
        match key.as_str() {
            "name" => name = Some(text(&field, value)?),
            "url" => environment.url = Some(text(&field, value)?),
            "deployment" => {
                environment.deployment = Some(match value {
                    Value::Bool(flag) => SourceText::Literal(flag.to_string()),
                    Value::String(text) if text.contains("${{") => SourceText::Unevaluated(text),
                    _ => return Err(Error::invalid(field, "expected a boolean")),
                });
            }
            _ => {
                return Err(Error::invalid(
                    field,
                    format_args!("unknown field `{key}`, expected `name`, `url` or `deployment`"),
                )
                .shown_at(route));
            }
        }
    }
    environment.name = name.ok_or_else(|| Error::invalid(route, "missing field `name`"))?;
    Ok(environment)
}

fn text(route: &str, value: Value) -> Result<SourceText, Error> {
    match value {
        Value::String(text) if text.contains("${{") => Ok(SourceText::Unevaluated(text)),
        // The schema accepts an empty string, which names no environment WSR can report.
        Value::String(text) if text.trim().is_empty() => {
            Err(Error::unsupported(route, "expected a nonempty string"))
        }
        Value::String(text) => Ok(SourceText::Literal(text)),
        // The pinned reader converts these to strings, which the schema accepts.
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            Err(Error::unsupported(route, NON_STRING))
        }
        _ => Err(Error::invalid(route, "expected a string")),
    }
}
