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

/// `budget` is shared by every matrix in the workflow; see [`within`].
pub(crate) fn decode(route: &str, value: Value, budget: &mut usize) -> Result<Strategy, Error> {
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
                if !within(&value, budget) {
                    return Err(Error::unsupported(
                        &field,
                        "matrix expands beyond twice the workflow source size through YAML aliases; not inspected yet",
                    ));
                }
                unique_matrix_keys(&field, &value)?;
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

/// Whether `value` fits in `budget` charge units, stopping as soon as it does not. A string is
/// charged its decoded length (at most 1.5 times its source, via `\L`/`\P`) and every other node,
/// empty string or null 1, so a literal matrix costs at most twice its source bytes (D-032: no size
/// cap); a costlier one is alias amplification, and is not rendered.
fn within(value: &Value, budget: &mut usize) -> bool {
    let size = match value {
        Value::String(text) => text.len().max(1),
        _ => 1,
    };
    let Some(rest) = budget.checked_sub(size) else {
        return false;
    };
    *budget = rest;
    match value {
        Value::Sequence(items) => items.iter().all(|item| within(item, budget)),
        Value::Mapping(entries) => entries
            .iter()
            .all(|(key, value)| within(key, budget) && within(value, budget)),
        Value::Tagged(tagged) => within(&tagged.value, budget),
        _ => true,
    }
}

/// A matrix or filter-item key: the pinned reader converts a boolean or number key to a string.
fn loose_key_text<'a>(route: &str, key: &'a Value) -> Result<&'a str, Error> {
    match key {
        Value::Bool(_) | Value::Number(_) => Err(Error::unsupported(
            route,
            "non-string key is not inspected yet",
        )),
        key => key_text(route, key),
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
        let key = loose_key_text(route, key)?;
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
                    if loose_key_text(&field, key)?.is_empty() {
                        return Err(Error::invalid(&field, "expected a nonempty string key"));
                    }
                }
            }
        }
    }
    Ok(())
}

// Match the pinned reader and env/with key normalization, including arbitrary nested values.
fn unique_matrix_keys(route: &str, value: &Value) -> Result<(), Error> {
    match value {
        Value::Mapping(entries) => {
            let mut spellings = std::collections::BTreeMap::new();
            for (key, value) in entries {
                let field = if let Value::String(key) = key {
                    let field = format!("{route}.{key}");
                    // The pinned reader does not compare expression keys.
                    if key.contains("${{") {
                        unique_matrix_keys(&field, value)?;
                        continue;
                    }
                    if let Some(first) = spellings.insert(key.to_uppercase(), key) {
                        return Err(Error::invalid(
                            field,
                            format_args!(
                                "duplicate key '{}'; already defined as '{}'",
                                key.escape_debug(),
                                first.escape_debug()
                            ),
                        ));
                    }
                    field
                } else if matches!(key, Value::Bool(_) | Value::Number(_)) {
                    // The pinned parser converts these keys to strings before comparing them.
                    return Err(Error::unsupported(
                        route,
                        "non-string key is not inspected yet",
                    ));
                } else {
                    route.to_owned()
                };
                unique_matrix_keys(&field, value)?;
            }
        }
        Value::Sequence(items) => {
            for (index, item) in items.iter().enumerate() {
                unique_matrix_keys(&format!("{route}[{}]", index + 1), item)?;
            }
        }
        Value::Tagged(tagged) => unique_matrix_keys(route, &tagged.value)?,
        _ => {}
    }
    Ok(())
}
