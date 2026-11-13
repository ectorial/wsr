//! Step `env` and `with` mappings, retained as source values and never bound or passed.

use std::collections::BTreeMap;

use serde::Deserialize;
use wsr_types::inspection::{SourceText, StepValue};

use crate::diagnostic::{self, Error};

pub(crate) fn env<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, StepValue>>, D::Error> {
    decode("env", serde_yaml::Value::deserialize(deserializer)?)
        .map(Some)
        .map_err(diagnostic::raise)
}

pub(crate) fn with<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, StepValue>>, D::Error> {
    decode("with", serde_yaml::Value::deserialize(deserializer)?)
        .map(Some)
        .map_err(diagnostic::raise)
}

// The pinned schema's `step-env`/`step-with`: nonempty string keys and loose string values,
// so YAML numbers and booleans are valid and keep their type here.
fn decode(field: &str, value: serde_yaml::Value) -> Result<BTreeMap<String, StepValue>, Error> {
    let entries = match value {
        serde_yaml::Value::Mapping(entries) => entries,
        serde_yaml::Value::String(text) if text.contains("${{") => {
            return Err(Error::unsupported(
                field,
                "expression is unevaluated; dynamic inspection is not supported yet",
            ));
        }
        _ => return Err(Error::invalid(field, "expected a mapping")),
    };
    let mut values = BTreeMap::new();
    for (key, value) in entries {
        let key = match key {
            serde_yaml::Value::String(key) if !key.is_empty() => key,
            _ => return Err(Error::invalid(field, "expected a nonempty string key")),
        };
        let route = format!("{field}.{key}");
        if key.contains("${{") {
            return Err(Error::unsupported(
                route,
                "expression is unevaluated; dynamic inspection is not supported yet",
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
                    route,
                    "expected a string, number or boolean",
                ));
            }
        };
        values.insert(key, value);
    }
    Ok(values)
}
