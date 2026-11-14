//! Strategy source inspection; never expands or schedules jobs.
use crate::diagnostic::Error;
use serde_yaml::Value;
use wsr_types::inspection::{SourceText, Strategy};

/// Arbitrary matrix values may contain sensitive text; withhold decoder detail.
pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<Value>, D::Error> {
    use serde::Deserialize as _;
    Value::deserialize(d).map(Some).map_err(|_| {
        serde::de::Error::custom("invalid YAML strategy value; decoder detail withheld")
    })
}

pub(crate) fn decode(route: &str, value: Value) -> Result<Strategy, Error> {
    if let Value::String(text) = &value {
        if text.contains("${{") {
            return Ok(Strategy::Unevaluated(text.clone()));
        }
    }
    let Value::Mapping(entries) = value else {
        return Err(Error::invalid(route, "expected a mapping"));
    };
    let mut fail_fast = None;
    let mut max_parallel = None;
    let mut matrix = None;
    for (key, value) in entries {
        let key = key_text(route, &key)?;
        let field = format!("{route}.{key}");
        match key {
            "fail-fast" => fail_fast = Some(scalar(&field, value, true)?),
            "max-parallel" => max_parallel = Some(scalar(&field, value, false)?),
            "matrix" => {
                validate_matrix(&field, &value)?;
                matrix = Some(match value {
                    Value::String(text) => SourceText::Unevaluated(text),
                    value => crate::source_text(
                        serde_yaml::to_string(&value)
                            .map_err(|_| Error::invalid(&field, "cannot retain YAML value"))?,
                    ),
                });
            }
            _ => {
                return Err(Error::invalid(
                    &field,
                    format_args!(
                        "unknown field `{key}`, expected `fail-fast`, `max-parallel` or `matrix`"
                    ),
                )
                .shown_at(route));
            }
        }
    }
    Ok(Strategy::Mapping {
        fail_fast,
        max_parallel,
        matrix,
    })
}

fn scalar(route: &str, value: Value, boolean: bool) -> Result<SourceText, Error> {
    match value {
        Value::String(text) if text.contains("${{") => Ok(SourceText::Unevaluated(text)),
        Value::Bool(value) if boolean => Ok(SourceText::Literal(value.to_string())),
        Value::Number(value) if !boolean => Ok(SourceText::Literal(value.to_string())),
        _ => Err(Error::invalid(
            route,
            if boolean {
                "expected a boolean"
            } else {
                "expected a number"
            },
        )),
    }
}

fn key_text<'a>(route: &str, key: &'a Value) -> Result<&'a str, Error> {
    match key {
        Value::String(text) if text.contains("${{") => Err(Error::unsupported(
            format!("{route}.{text}"),
            "expression key is not inspected yet",
        )),
        Value::String(text) => Ok(text),
        _ => Err(Error::invalid(route, "expected a string key")),
    }
}

fn expression(value: &Value) -> bool {
    matches!(value, Value::String(text) if text.contains("${{"))
}

fn validate_matrix(route: &str, value: &Value) -> Result<(), Error> {
    if expression(value) {
        return Ok(());
    }
    let Value::Mapping(entries) = value else {
        return Err(Error::invalid(route, "expected a mapping"));
    };
    for (key, value) in entries {
        let key = key_text(route, key)?;
        let field = format!("{route}.{key}");
        if key.is_empty() {
            return Err(Error::invalid(field, "expected a nonempty string key"));
        }
        if expression(value) {
            continue;
        }
        let Value::Sequence(items) = value else {
            return Err(Error::invalid(field, "expected a sequence"));
        };
        if matches!(key, "include" | "exclude") {
            for (i, item) in items.iter().enumerate() {
                let field = format!("{field}[{}]", i + 1);
                if expression(item) {
                    continue;
                }
                let Value::Mapping(entries) = item else {
                    return Err(Error::invalid(field, "expected a mapping"));
                };
                for key in entries.keys() {
                    if key_text(&field, key)?.is_empty() {
                        return Err(Error::invalid(&field, "expected a nonempty string key"));
                    }
                }
            }
        }
    }
    Ok(())
}
