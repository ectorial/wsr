//! Static manual-trigger declarations; no event binding or default application.
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use wsr_types::inspection::{
    DispatchBody, DispatchInput, DispatchInputType, EventFilter, InputDefault, ManualTrigger,
    Trigger,
};

use super::{Category, Error, diagnostic};

/// Events the pinned schema accepts under `on`; only `workflow_dispatch` configuration is inspected.
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

/// Events whose pinned definition admits only a null body.
const NULL_ONLY: &[&str] = &[
    "create",
    "delete",
    "deployment",
    "deployment_status",
    "fork",
    "gollum",
    "page_build",
    "public",
    "status",
];

pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Trigger>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    parse(&value).map_err(diagnostic::raise)
}

/// `on` in source order. Bare event names are reported; event configuration is not inspected.
fn parse(value: &Value) -> Result<Vec<Trigger>, Error> {
    match value {
        Value::String(_) => Ok(vec![bare(value, "on")?]),
        Value::Sequence(events) => {
            let mut triggers = Vec::new();
            for (index, value) in events.iter().enumerate() {
                let route = format!("on[{}]", index + 1);
                let trigger = bare(value, &route)?;
                if triggers.contains(&trigger) {
                    return Err(Error::unsupported(
                        route,
                        "repeated event is not inspected yet",
                    ));
                }
                triggers.push(trigger);
            }
            if triggers.is_empty() {
                return Err(Error::unsupported("on", EMPTY));
            }
            Ok(triggers)
        }
        Value::Mapping(events) => {
            fields(events, EVENTS, "on")?;
            if events.is_empty() {
                return Err(Error::unsupported("on", EMPTY));
            }
            events
                .iter()
                .map(|(event, body)| {
                    let event = text(event, "on")?;
                    let route = format!("on.{event}");
                    match event.as_str() {
                        "workflow_dispatch" => dispatch(body).map(Trigger::Manual),
                        // The pinned `schedule` is a sequence of cron mappings, never null.
                        "schedule" => Err(match body {
                            Value::Sequence(_) => Error::unsupported(route, CONFIGURATION),
                            _ => Error::invalid(route, "expected a sequence"),
                        }),
                        _ => match body {
                            Value::Null => Ok(Trigger::Event(event)),
                            Value::Mapping(body)
                                if !body.is_empty() && FILTERED.contains(&event.as_str()) =>
                            {
                                let filters = event_filters(&event, body, &route)?;
                                Ok(Trigger::Filtered { event, filters })
                            }
                            _ if NULL_ONLY.contains(&event.as_str()) => {
                                Err(Error::invalid(route, "expected null"))
                            }
                            Value::Mapping(body) if body.is_empty() => Ok(Trigger::Event(event)),
                            Value::Mapping(_) => Err(Error::unsupported(route, CONFIGURATION)),
                            _ => Err(Error::invalid(route, "expected null or a mapping")),
                        },
                    }
                })
                .collect()
        }
        _ => Err(Error::invalid(
            "on",
            "expected an event name, sequence or mapping",
        )),
    }
}

const CONFIGURATION: &str = "event configuration is not inspected yet";

/// Events whose branch, tag and path filters are inspected.
const FILTERED: &[&str] = &["push", "pull_request", "pull_request_target"];
/// Filter keys of `push-mapping`; the pull request mappings have `types` and no tag filters.
const PUSH_FILTERS: &[&str] = &[
    "branches",
    "branches-ignore",
    "tags",
    "tags-ignore",
    "paths",
    "paths-ignore",
];
const PULL_REQUEST_FILTERS: &[&str] = &[
    "types",
    "branches",
    "branches-ignore",
    "paths",
    "paths-ignore",
];

/// Filters in source order. Each is a non-empty string or a sequence of them, with no expression
/// context; a filter and its `-ignore` counterpart cannot both be given.
fn event_filters(event: &str, body: &Mapping, route: &str) -> Result<Vec<EventFilter>, Error> {
    let allowed = if event == "push" {
        PUSH_FILTERS
    } else {
        PULL_REQUEST_FILTERS
    };
    fields(body, allowed, route)?;
    let mut filters: Vec<EventFilter> = Vec::new();
    for (name, value) in body {
        let name = text(name, route)?;
        let route = format!("{route}.{name}");
        if name == "types" {
            return Err(Error::unsupported(
                route,
                "unsupported field `types`; this field is not inspected yet",
            ));
        }
        let counterpart = match name.strip_suffix("-ignore") {
            Some(base) => base.to_owned(),
            None => format!("{name}-ignore"),
        };
        if filters.iter().any(|filter| filter.name == counterpart) {
            return Err(Error::invalid(
                route,
                format_args!("cannot be combined with `{counterpart}`"),
            ));
        }
        let patterns = match value {
            Value::Sequence(values) => values
                .iter()
                .enumerate()
                .map(|(index, value)| pattern(value, &format!("{route}[{}]", index + 1)))
                .collect::<Result<_, _>>()?,
            _ => vec![pattern(value, &route)?],
        };
        filters.push(EventFilter { name, patterns });
    }
    Ok(filters)
}

/// One `non-empty-string` pattern; the reader converts numbers and booleans, never null.
fn pattern(value: &Value, route: &str) -> Result<String, Error> {
    match value {
        Value::Bool(_) | Value::Number(_) => Err(Error::unsupported(route, super::NON_STRING)),
        Value::String(text) if text.is_empty() => {
            Err(Error::invalid(route, "expected a non-empty string"))
        }
        Value::String(_) => text(value, route),
        _ => Err(Error::invalid(
            route,
            "expected a non-empty string or a sequence of them",
        )),
    }
}
// The strict schema sets no minimum size; whether GitHub accepts no events is not established.
const EMPTY: &str = "an empty event list is not inspected yet";

/// A bare event name at `route`; `on-string-strict` admits every pinned event.
fn bare(value: &Value, route: &str) -> Result<Trigger, Error> {
    let event = text(value, route)?;
    if !EVENTS.contains(&event.as_str()) {
        return Err(Error::invalid(
            route,
            format_args!("unknown event '{}'", event.escape_debug()),
        ));
    }
    Ok(if event == "workflow_dispatch" {
        Trigger::Manual(ManualTrigger::Scalar)
    } else {
        Trigger::Event(event)
    })
}

fn dispatch(body: &Value) -> Result<ManualTrigger, Error> {
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
            let mut spellings = std::collections::BTreeMap::new();
            for (id, value) in inputs {
                let id = text(id, "on.workflow_dispatch.inputs")?;
                let route = format!("on.workflow_dispatch.inputs.{id}");
                super::check_required(&id, &route, Category::Invalid)?;
                if !(id.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                {
                    return Err(Error::invalid(
                        route,
                        format_args!("invalid input identifier '{}'", id.escape_debug()),
                    ));
                }
                // The pinned reader rejects identifiers repeated ignoring case.
                if let Some(first) = spellings.insert(id.to_uppercase(), id.clone()) {
                    return Err(Error::invalid(
                        route,
                        format_args!(
                            "duplicate input identifier '{}'; already defined as '{}'",
                            id.escape_debug(),
                            first.escape_debug()
                        ),
                    ));
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
        route,
    )?;
    let description = value
        .get("description")
        .map(|v| converted_text(v, &format!("{route}.description")))
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
                super::check_literal(Some(text), &format!("{route}.required"), Category::Invalid)?;
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
            // The reader converts a null to an empty string, which only a boolean input rejects.
            Value::Null if input_type != Some(DispatchInputType::Boolean) => Err(
                Error::unsupported(format!("{route}.default"), super::NON_STRING),
            ),
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
                .map(|(index, v)| converted_text(v, &format!("{route}.options[{}]", index + 1)))
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
    super::check_literal(Some(value), route, Category::Invalid)?;
    Ok(value.clone())
}

/// Like [`text`], for a field whose null, boolean and number literals the pinned reader converts
/// to strings that the schema accepts.
fn converted_text(value: &Value, route: &str) -> Result<String, Error> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            Err(Error::unsupported(route, super::NON_STRING))
        }
        _ => text(value, route),
    }
}

/// Reject keys outside `allowed`; an event name under `on`, a field elsewhere.
fn fields(value: &Mapping, allowed: &[&str], route: &str) -> Result<(), Error> {
    for key in value.keys() {
        let key = text(key, route)?;
        if !allowed.contains(&key.as_str()) {
            let detail = if route == "on" {
                format!("unknown event '{}'", key.escape_debug())
            } else {
                format!("unknown field `{key}`")
            };
            return Err(Error::invalid(format!("{route}.{key}"), detail).shown_at(route));
        }
    }
    Ok(())
}
