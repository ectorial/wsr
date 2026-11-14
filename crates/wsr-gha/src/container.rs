//! Job `container`, retained as source values; never pulled, created or started.

use serde_yaml::Value;
use wsr_types::inspection::{Container, ContainerCredentials, SourceText, StepValue};

use crate::NON_STRING;
use crate::diagnostic::Error;
use crate::step_inputs;

const EXPRESSION: &str = "expression is unevaluated; dynamic inspection is not supported yet";

/// Decode `container` at `route`. The pinned `container` has an expression context that its values
/// inherit, and is a string image or a `container-mapping`. `convertToJobContainer` skips its
/// checks, including the required image, when any token is an expression.
pub(crate) fn decode(route: &str, value: Value) -> Result<Container, Error> {
    let entries = match value {
        Value::Mapping(entries) => entries,
        // The runner treats an empty image as no container, which WSR does not report.
        Value::String(text) if text.trim().is_empty() => {
            return Err(Error::unsupported(route, "expected a nonempty string"));
        }
        Value::String(text) => {
            return Ok(Container {
                image: source_text(text),
                credentials: None,
                env: None,
                ports: None,
                volumes: None,
                options: None,
            });
        }
        // The pinned reader converts these to strings, which the schema accepts.
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            return Err(Error::unsupported(route, NON_STRING));
        }
        _ => return Err(Error::invalid(route, "expected a string or mapping")),
    };
    let mut image = None;
    let mut container = Container {
        image: SourceText::Literal(String::new()),
        credentials: None,
        env: None,
        ports: None,
        volumes: None,
        options: None,
    };
    for (key, value) in entries {
        let field = field(route, key)?;
        let key = &field[route.len() + 1..];
        match key {
            "image" => image = Some(required(&field, value)?),
            "options" => {
                container.options = Some(match value {
                    Value::String(text) => source_text(text),
                    Value::Null | Value::Bool(_) | Value::Number(_) => {
                        return Err(Error::unsupported(field, NON_STRING));
                    }
                    _ => return Err(Error::invalid(field, "expected a string")),
                });
            }
            "env" => container.env = Some(step_inputs::decode(&field, value)?),
            "ports" => container.ports = Some(entries_of(&field, value)?),
            "volumes" => container.volumes = Some(entries_of(&field, value)?),
            "credentials" => container.credentials = Some(credentials(&field, value)?),
            _ => {
                return Err(Error::invalid(
                    &field,
                    format_args!(
                        "unknown field `{key}`, expected one of `image`, `credentials`, `env`, `ports`, `volumes`, `options`"
                    ),
                )
                .shown_at(route));
            }
        }
    }
    container.image = match image {
        Some(image) => image,
        None if has_expression(&container) => {
            return Err(Error::unsupported(
                route,
                "missing field `image`; a container with expressions is not inspected without one",
            ));
        }
        None => return Err(Error::invalid(route, "missing field `image`")),
    };
    Ok(container)
}

/// The route of a mapping key. An expression key is valid in this expression context.
fn field(route: &str, key: Value) -> Result<String, Error> {
    match key {
        Value::String(key) if key.contains("${{") => {
            Err(Error::unsupported(format!("{route}.{key}"), EXPRESSION))
        }
        Value::String(key) => Ok(format!("{route}.{key}")),
        _ => Err(Error::invalid(route, "expected a string key")),
    }
}

/// `credentials`: a mapping of `username` and `password`, each a `non-empty-string`.
fn credentials(route: &str, value: Value) -> Result<ContainerCredentials, Error> {
    let entries = match value {
        Value::Mapping(entries) => entries,
        Value::String(text) if text.contains("${{") => {
            return Err(Error::unsupported(route, EXPRESSION));
        }
        _ => return Err(Error::invalid(route, "expected a mapping")),
    };
    let mut credentials = ContainerCredentials {
        username: None,
        password: None,
    };
    for (key, value) in entries {
        let field = field(route, key)?;
        match &field[route.len() + 1..] {
            "username" => credentials.username = Some(required(&field, value)?),
            "password" => credentials.password = Some(required(&field, value)?),
            key => {
                return Err(Error::invalid(
                    &field,
                    format_args!("unknown field `{key}`, expected `username` or `password`"),
                )
                .shown_at(route));
            }
        }
    }
    Ok(credentials)
}

/// A `non-empty-string`; the pinned reader converts null to an empty string.
fn required(route: &str, value: Value) -> Result<SourceText, Error> {
    match value {
        Value::String(text) if text.trim().is_empty() => {
            Err(Error::invalid(route, "expected a nonempty string"))
        }
        Value::String(text) => Ok(source_text(text)),
        Value::Null => Err(Error::invalid(route, "expected a nonempty string")),
        Value::Bool(_) | Value::Number(_) => Err(Error::unsupported(route, NON_STRING)),
        _ => Err(Error::invalid(route, "expected a string")),
    }
}

/// A `sequence-of-non-empty-string`; numbers and booleans convert to nonempty strings and keep
/// their YAML type here.
fn entries_of(route: &str, value: Value) -> Result<Vec<StepValue>, Error> {
    let items = match value {
        Value::Sequence(items) => items,
        Value::String(text) if text.contains("${{") => {
            return Err(Error::unsupported(route, EXPRESSION));
        }
        _ => return Err(Error::invalid(route, "expected a sequence")),
    };
    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let route = format!("{route}[{}]", index + 1);
            match item {
                Value::String(text) if text.is_empty() => {
                    Err(Error::invalid(route, "expected a nonempty string"))
                }
                Value::String(text) => Ok(StepValue::Text(source_text(text))),
                Value::Bool(flag) => Ok(StepValue::Boolean(flag)),
                Value::Number(number) => Ok(StepValue::Number(number.to_string())),
                Value::Null => Err(Error::invalid(route, "expected a nonempty string")),
                _ => Err(Error::invalid(route, "expected a string")),
            }
        })
        .collect()
}

fn source_text(text: String) -> SourceText {
    if text.contains("${{") {
        SourceText::Unevaluated(text)
    } else {
        SourceText::Literal(text)
    }
}

fn has_expression(container: &Container) -> bool {
    let text = |text: &Option<SourceText>| matches!(text, Some(SourceText::Unevaluated(_)));
    let value = |value: &StepValue| matches!(value, StepValue::Text(SourceText::Unevaluated(_)));
    text(&container.options)
        || container
            .credentials
            .as_ref()
            .is_some_and(|c| text(&c.username) || text(&c.password))
        || container.env.iter().flat_map(|env| env.values()).any(value)
        || container.ports.iter().flatten().any(value)
        || container.volumes.iter().flatten().any(value)
}
