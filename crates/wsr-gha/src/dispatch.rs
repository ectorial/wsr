//! Static manual-trigger declarations; no event binding or default application.
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use wsr_types::inspection::{
    DispatchBody, DispatchInput, DispatchInputType, InputDefault, ManualTrigger,
};

use super::{Category, Error, diagnostic};

/// Events the pinned schema accepts under `on`; only `workflow_dispatch` is inspected.
const EVENTS: &[&str] = &[
    "branch_protection_rule",
    "check_run",
    "check_suite",
    "create",
    "delete",
    "deployment",
    "deployment_status",
    "discussion",
    "discussion_comment",
    "fork",
    "gollum",
    "image_version",
    "issue_comment",
    "issues",
    "label",
    "merge_group",
    "milestone",
    "page_build",
    "project",
    "project_card",
    "project_column",
    "public",
    "pull_request",
    "pull_request_comment",
    "pull_request_review",
    "pull_request_review_comment",
    "pull_request_target",
    "push",
    "registry_package",
    "release",
    "repository_dispatch",
    "schedule",
    "status",
    "watch",
    "workflow_call",
    "workflow_dispatch",
    "workflow_run",
];

pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ManualTrigger>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    parse(&value).map(Some).map_err(diagnostic::raise)
}

fn parse(value: &Value) -> Result<ManualTrigger, Error> {
    if let Value::String(event) = value {
        super::check_literal(Some(event), "on")?;
        if event != "workflow_dispatch" {
            let detail = "unsupported trigger; only workflow_dispatch is inspected yet";
            return Err(if EVENTS.contains(&event.as_str()) {
                Error::unsupported("on", detail)
            } else {
                Error::invalid("on", detail)
            });
        }
        return Ok(ManualTrigger::Scalar);
    }
    let events = mapping(value, "on").map_err(|error| match value {
        Value::Sequence(_) => {
            Error::new(Category::Unsupported, Some("on".into()), error.to_string())
        }
        _ => error,
    })?;
    fields(events, &["workflow_dispatch"], EVENTS, "on")?;
    let body = events
        .get("workflow_dispatch")
        .ok_or_else(|| Error::invalid("on", "expected workflow_dispatch entry"))?;
    if body.is_null() {
        return Ok(ManualTrigger::Mapping(None));
    }
    let body = mapping(body, "on.workflow_dispatch")?;
    fields(body, &["inputs"], &[], "on.workflow_dispatch")?;
    let inputs = body
        .get("inputs")
        .map(|value| {
            let inputs = mapping(value, "on.workflow_dispatch.inputs")?;
            let mut result = Vec::new();
            for (id, value) in inputs {
                let label = "on.workflow_dispatch.inputs identifier";
                let id =
                    text(id, label).map_err(|error| error.routed("on.workflow_dispatch.inputs"))?;
                let route = format!("on.workflow_dispatch.inputs.{id}");
                super::check_required(&id, label).map_err(|error| error.routed(&route))?;
                if !(id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                {
                    return Err(Error::invalid(
                        "on.workflow_dispatch.inputs",
                        format_args!("invalid input identifier {id:?}"),
                    )
                    .routed(route));
                }
                result.push(input(id, mapping(value, &route)?, &route)?);
            }
            result.sort_by(|a, b| a.id.cmp(&b.id));
            Ok::<_, Error>(result)
        })
        .transpose()?;
    Ok(ManualTrigger::Mapping(Some(DispatchBody { inputs })))
}

fn input(id: String, value: &Mapping, route: &str) -> Result<DispatchInput, Error> {
    fields(
        value,
        &["description", "type", "required", "default", "options"],
        &[],
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
                _ => Err(Error::invalid(
                    format!("{route}.type"),
                    format_args!("unsupported input type {name:?}"),
                )),
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
                _ => Err(Error::invalid(
                    format!("{route}.required"),
                    "expected a boolean",
                )),
            }
        })
        .transpose()?;
    let default = value
        .get("default")
        .map(|v| match v {
            Value::String(_) => text(v, &format!("{route}.default")).map(InputDefault::String),
            Value::Bool(flag) => Ok(InputDefault::Boolean(*flag)),
            Value::Number(number) => Ok(InputDefault::Number(number.to_string())),
            _ => Err(Error::invalid(
                format!("{route}.default"),
                "expected a string, boolean or number",
            )),
        })
        .transpose()?;
    let options = value
        .get("options")
        .map(|v| {
            let Value::Sequence(values) = v else {
                return Err(Error::invalid(
                    format!("{route}.options"),
                    "expected a string list",
                ));
            };
            values
                .iter()
                .enumerate()
                .map(|(index, v)| text(v, &format!("{route}.options[{}]", index + 1)))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let options_error = match input_type.as_ref().unwrap_or(&DispatchInputType::String) {
        DispatchInputType::Choice if !options.as_ref().is_some_and(|values| !values.is_empty()) => {
            Some("expected at least one option for choice input")
        }
        DispatchInputType::Choice => None,
        _ => options
            .is_some()
            .then_some("only supported for choice input"),
    };
    if let Some(detail) = options_error {
        return Err(Error::invalid(format!("{route}.options"), detail));
    }
    if let Some(default) = &default {
        let mismatch = match input_type.as_ref().unwrap_or(&DispatchInputType::String) {
            DispatchInputType::Boolean => (!matches!(default, InputDefault::Boolean(_)))
                .then_some("expected a boolean for boolean input"),
            DispatchInputType::String
            | DispatchInputType::Choice
            | DispatchInputType::Environment => (!matches!(default, InputDefault::String(_)))
                .then_some("expected a string for string, choice or environment input"),
            // The pinned converter does not add number-default type checks. Retain the
            // schema's scalar here rather than drop it or invent stricter compatibility.
            DispatchInputType::Number => None,
        };
        if let Some(detail) = mismatch {
            return Err(Error::invalid(format!("{route}.default"), detail));
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

fn mapping<'a>(value: &'a Value, route: &str) -> Result<&'a Mapping, Error> {
    value.as_mapping().ok_or_else(|| {
        Error::invalid(
            route,
            "expected a mapping (other trigger forms are unsupported)",
        )
    })
}

fn text(value: &Value, route: &str) -> Result<String, Error> {
    let Value::String(value) = value else {
        return Err(Error::invalid(route, "expected a string"));
    };
    super::check_literal(Some(value), route)?;
    Ok(value.clone())
}

/// Reject keys outside `allowed`; those in `schema` are valid GHA that is not inspected yet.
fn fields(value: &Mapping, allowed: &[&str], schema: &[&str], route: &str) -> Result<(), Error> {
    for key in value.keys() {
        let key = text(key, route)?;
        if !allowed.contains(&key.as_str()) {
            let field = format!("{route}.{key}");
            return Err(if schema.contains(&key.as_str()) {
                Error::unsupported(field, "unsupported field or event")
            } else {
                Error::invalid(field, "unsupported field or event")
            });
        }
    }
    Ok(())
}
