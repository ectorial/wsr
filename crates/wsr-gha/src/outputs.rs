//! Job `outputs`, retained as source text and never resolved.

use std::collections::BTreeMap;

use serde_yaml::Value;
use wsr_types::inspection::SourceText;

use crate::NON_STRING;
use crate::diagnostic::Error;

const NOT_ALLOWED: &str = "expression is unevaluated and not allowed in job outputs";

// The pinned `job-outputs` has no expression context: nonempty string keys, unique ignoring case,
// with `string-runner-context` values (which do have one).
pub(crate) fn decode(route: &str, value: Value) -> Result<BTreeMap<String, SourceText>, Error> {
    let entries = match value {
        Value::Mapping(entries) => entries,
        Value::String(text) if text.contains("${{") => {
            return Err(Error::invalid(route, NOT_ALLOWED));
        }
        _ => return Err(Error::invalid(route, "expected a mapping")),
    };
    let mut outputs = BTreeMap::new();
    let mut spellings = BTreeMap::new();
    for (key, value) in entries {
        let key = match key {
            Value::String(key) if key.contains("${{") => {
                return Err(Error::invalid(route, NOT_ALLOWED));
            }
            Value::String(key) if !key.is_empty() => key,
            // The pinned YAML adapter keeps a boolean or number key raw, which the reader rejects.
            _ => return Err(Error::invalid(route, "expected a nonempty string key")),
        };
        let field = format!("{route}.{key}");
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
        let text = match value {
            Value::String(text) if text.contains("${{") => SourceText::Unevaluated(text),
            Value::String(text) => SourceText::Literal(text),
            // The pinned reader converts these to strings, which string-runner-context accepts.
            Value::Null | Value::Bool(_) | Value::Number(_) => {
                return Err(Error::unsupported(field, NON_STRING));
            }
            _ => return Err(Error::invalid(field, "expected a string")),
        };
        outputs.insert(key, text);
    }
    Ok(outputs)
}
