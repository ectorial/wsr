//! Static manual-trigger declarations; no event binding or default application.
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use wsr_types::inspection::{
    DispatchBody, DispatchInput, DispatchInputType, InputDefault, ManualTrigger,
};

pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ManualTrigger>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    parse(&value).map(Some).map_err(serde::de::Error::custom)
}

fn parse(value: &Value) -> anyhow::Result<ManualTrigger> {
    if let Value::String(event) = value {
        super::check_literal(Some(event), "on")?;
        anyhow::ensure!(
            event == "workflow_dispatch",
            "on: unsupported trigger; only workflow_dispatch is inspected yet"
        );
        return Ok(ManualTrigger::Scalar);
    }
    let events = mapping(value, "on")?;
    fields(events, &["workflow_dispatch"], "on")?;
    let body = events
        .get("workflow_dispatch")
        .ok_or_else(|| anyhow::anyhow!("on: expected workflow_dispatch entry"))?;
    if body.is_null() {
        return Ok(ManualTrigger::Mapping(None));
    }
    let body = mapping(body, "on.workflow_dispatch")?;
    fields(body, &["inputs"], "on.workflow_dispatch")?;
    let inputs = body
        .get("inputs")
        .map(|value| {
            let inputs = mapping(value, "on.workflow_dispatch.inputs")?;
            let mut result = Vec::new();
            for (id, value) in inputs {
                let id = text(id, "on.workflow_dispatch.inputs identifier")?;
                super::check_required(&id, "on.workflow_dispatch.inputs identifier")?;
                anyhow::ensure!(
                    id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                        && id
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                    "on.workflow_dispatch.inputs: invalid input identifier {id:?}"
                );
                let route = format!("on.workflow_dispatch.inputs.{id}");
                result.push(input(id, mapping(value, &route)?, &route)?);
            }
            result.sort_by(|a, b| a.id.cmp(&b.id));
            Ok::<_, anyhow::Error>(result)
        })
        .transpose()?;
    Ok(ManualTrigger::Mapping(Some(DispatchBody { inputs })))
}

fn input(id: String, value: &Mapping, route: &str) -> anyhow::Result<DispatchInput> {
    fields(
        value,
        &["description", "type", "required", "default", "options"],
        route,
    )?;
    let description = value
        .get("description")
        .map(|v| text(v, &format!("{route}.description")))
        .transpose()?;
    let input_type = value
        .get("type")
        .map(|v| {
            let name = text(v, &format!("{route}.type"))?;
            match name.as_str() {
                "string" => Ok(DispatchInputType::String),
                "boolean" => Ok(DispatchInputType::Boolean),
                "number" => Ok(DispatchInputType::Number),
                "choice" => Ok(DispatchInputType::Choice),
                "environment" => Ok(DispatchInputType::Environment),
                _ => anyhow::bail!("{route}.type: unsupported input type {name:?}"),
            }
        })
        .transpose()?;
    let required = value
        .get("required")
        .map(|v| {
            if let Value::String(text) = v {
                super::check_literal(Some(text), &format!("{route}.required"))?;
            }
            match v {
                Value::Bool(flag) => Ok(*flag),
                _ => anyhow::bail!("{route}.required: expected a boolean"),
            }
        })
        .transpose()?;
    let default = value
        .get("default")
        .map(|v| match v {
            Value::String(_) => text(v, &format!("{route}.default")).map(InputDefault::String),
            Value::Bool(flag) => Ok(InputDefault::Boolean(*flag)),
            Value::Number(number) => Ok(InputDefault::Number(number.to_string())),
            _ => anyhow::bail!("{route}.default: expected a string, boolean or number"),
        })
        .transpose()?;
    let options = value
        .get("options")
        .map(|v| {
            let Value::Sequence(values) = v else {
                anyhow::bail!("{route}.options: expected a string list");
            };
            values
                .iter()
                .enumerate()
                .map(|(index, v)| text(v, &format!("{route}.options[{}]", index + 1)))
                .collect::<anyhow::Result<Vec<_>>>()
        })
        .transpose()?;
    match input_type.as_ref().unwrap_or(&DispatchInputType::String) {
        DispatchInputType::Choice => anyhow::ensure!(
            options.as_ref().is_some_and(|values| !values.is_empty()),
            "{route}.options: expected at least one option for choice input"
        ),
        _ => anyhow::ensure!(
            options.is_none(),
            "{route}.options: only supported for choice input"
        ),
    }
    if let Some(default) = &default {
        match input_type.as_ref().unwrap_or(&DispatchInputType::String) {
            DispatchInputType::Boolean => anyhow::ensure!(
                matches!(default, InputDefault::Boolean(_)),
                "{route}.default: expected a boolean for boolean input"
            ),
            DispatchInputType::String
            | DispatchInputType::Choice
            | DispatchInputType::Environment => anyhow::ensure!(
                matches!(default, InputDefault::String(_)),
                "{route}.default: expected a string for string, choice or environment input"
            ),
            // The pinned converter does not add number-default type checks. Retain the
            // schema's scalar here rather than drop it or invent stricter compatibility.
            DispatchInputType::Number => {}
        }
    }
    Ok(DispatchInput {
        id,
        description,
        input_type,
        required,
        default,
        options,
    })
}

fn mapping<'a>(value: &'a Value, route: &str) -> anyhow::Result<&'a Mapping> {
    value.as_mapping().ok_or_else(|| {
        anyhow::anyhow!("{route}: expected a mapping (other trigger forms are unsupported)")
    })
}

fn text(value: &Value, route: &str) -> anyhow::Result<String> {
    let Value::String(value) = value else {
        anyhow::bail!("{route}: expected a string");
    };
    super::check_literal(Some(value), route)?;
    Ok(value.clone())
}

fn fields(value: &Mapping, allowed: &[&str], route: &str) -> anyhow::Result<()> {
    for key in value.keys() {
        let key = text(key, route)?;
        anyhow::ensure!(
            allowed.contains(&key.as_str()),
            "{route}.{key}: unsupported field or event"
        );
    }
    Ok(())
}
