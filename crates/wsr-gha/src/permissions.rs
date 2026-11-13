//! Requested `GITHUB_TOKEN` permissions, decoded against the pinned schema's scopes and levels.

use std::collections::BTreeMap;

use serde::Deserialize;
use wsr_types::inspection::{PermissionLevel, Permissions};

use crate::diagnostic::{self, Error};

// The pinned schema's `permissions-mapping` scopes, each with its allowed levels.
const ANY: &[&str] = &["read", "write", "none"];
const READ: &[&str] = &["read", "none"];
const WRITE: &[&str] = &["write", "none"];
const SCOPES: &[(&str, &[&str])] = &[
    ("actions", ANY),
    ("artifact-metadata", ANY),
    ("attestations", ANY),
    ("checks", ANY),
    ("code-quality", ANY),
    ("contents", ANY),
    ("copilot-requests", WRITE),
    ("deployments", ANY),
    ("discussions", ANY),
    ("drives", ANY),
    ("id-token", WRITE),
    ("issues", ANY),
    ("models", READ),
    ("packages", ANY),
    ("pages", ANY),
    ("pull-requests", ANY),
    ("repository-projects", ANY),
    ("security-events", ANY),
    ("statuses", ANY),
    ("vulnerability-alerts", READ),
];

// The pinned schema gives `permissions` no expression context.
const EXPRESSION: &str = "expression is unevaluated and not allowed in permissions";

pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Permissions>, D::Error> {
    decode(serde_yaml::Value::deserialize(deserializer)?)
        .map(Some)
        .map_err(diagnostic::raise)
}

fn decode(value: serde_yaml::Value) -> Result<Permissions, Error> {
    let scopes = match value {
        serde_yaml::Value::String(text) if text == "read-all" => return Ok(Permissions::ReadAll),
        serde_yaml::Value::String(text) if text == "write-all" => return Ok(Permissions::WriteAll),
        serde_yaml::Value::String(text) if text.contains("${{") => {
            return Err(Error::invalid("permissions", EXPRESSION));
        }
        serde_yaml::Value::Mapping(scopes) => scopes,
        _ => {
            return Err(Error::invalid(
                "permissions",
                "expected read-all, write-all or a mapping of permission scopes",
            ));
        }
    };
    let mut requested = BTreeMap::new();
    for (scope, level) in scopes {
        let serde_yaml::Value::String(scope) = scope else {
            return Err(Error::invalid(
                "permissions",
                "expected a string permission scope",
            ));
        };
        let route = format!("permissions.{scope}");
        let Some((_, levels)) = SCOPES.iter().find(|(name, _)| *name == scope) else {
            return Err(Error::invalid(
                route,
                format_args!("unknown permission scope '{}'", scope.escape_debug()),
            ));
        };
        let level = match level.as_str() {
            Some(text) if text.contains("${{") => return Err(Error::invalid(route, EXPRESSION)),
            Some(text) if levels.contains(&text) => match text {
                "read" => PermissionLevel::Read,
                "write" => PermissionLevel::Write,
                _ => PermissionLevel::None,
            },
            _ => {
                return Err(Error::invalid(
                    route,
                    format_args!(
                        "expected {}",
                        match levels.len() {
                            3 => "read, write or none".to_owned(),
                            _ => levels.join(" or "),
                        }
                    ),
                ));
            }
        };
        requested.insert(scope, level);
    }
    Ok(Permissions::Scopes(requested))
}
