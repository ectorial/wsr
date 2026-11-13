//! Step `env` and `with` mappings, retained as source values and never bound or passed.

use std::collections::BTreeMap;

use wsr_types::inspection::{SourceText, StepValue};

use crate::diagnostic::Error;

const EXPRESSION: &str = "expression is unevaluated; dynamic inspection is not supported yet";

// The pinned schema's `step-env`/`step-with`: nonempty string keys, unique ignoring case, and loose
// string values, so YAML numbers and booleans are valid and keep their type here.
pub(crate) fn decode(
    route: &str,
    value: serde_yaml::Value,
) -> Result<BTreeMap<String, StepValue>, Error> {
    let entries = match value {
        serde_yaml::Value::Mapping(entries) => entries,
        serde_yaml::Value::String(text) if text.contains("${{") => {
            return Err(Error::unsupported(route, EXPRESSION));
        }
        _ => return Err(Error::invalid(route, "expected a mapping")),
    };
    let mut values = BTreeMap::new();
    let mut spellings = BTreeMap::new();
    for (key, value) in entries {
        let key = match key {
            serde_yaml::Value::String(key) if !key.is_empty() => key,
            // The pinned parser converts other scalar keys to strings.
            serde_yaml::Value::Bool(_) | serde_yaml::Value::Number(_) => {
                return Err(Error::unsupported(
                    route,
                    "non-string key is not inspected yet",
                ));
            }
            _ => return Err(Error::invalid(route, "expected a nonempty string key")),
        };
        let field = format!("{route}.{key}");
        if key.contains("${{") {
            return Err(Error::unsupported(field, EXPRESSION));
        }
        if let Some(first) = spellings.insert(key.to_uppercase(), key.clone()) {
            return Err(Error::invalid(
                field,
                format_args!(
                    "duplicate key '{}'; already defined as '{}'",
                    key.escape_debug(),
                    first.escape_debug()
                ),
            ));
        }
        let value = match value {
            serde_yaml::Value::String(text) if text.contains("${{") => {
                StepValue::Text(SourceText::Unevaluated(text))
            }
            serde_yaml::Value::String(text) => StepValue::Text(SourceText::Literal(text)),
            serde_yaml::Value::Bool(value) => StepValue::Boolean(value),
            serde_yaml::Value::Number(value) => StepValue::Number(value.to_string()),
            _ => {
                return Err(Error::invalid(
                    field,
                    "expected a string, number or boolean",
                ));
            }
        };
        values.insert(key, value);
    }
    Ok(values)
}
