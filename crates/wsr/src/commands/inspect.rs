use std::fmt::Write as _;
use std::io::{ErrorKind, Read as _, Write as _};
use std::path::Path;

use wsr_types::inspection::{
    CallSecrets, Concurrency, Container, DispatchInputType, InputDefault, ManualTrigger,
    PermissionLevel, Permissions, RunDefaults, RunnerRequirement, SourceText, StepSummaryKind,
    StepValue, Trigger,
};

// GitHub documents a 500 KB workflow file limit (D-032); 512,000 bytes is WSR's reading of it.
// WSR warns but imposes no limit of its own.
const GITHUB_WORKFLOW_LIMIT: usize = 512_000;

pub fn run(file: &Path) -> anyhow::Result<()> {
    // Workflow content is untrusted and reaches diagnostics (serde field names, `on:` keys),
    // so escape unprintable characters before the message can be written to a terminal.
    let (size, report) =
        render(file).map_err(|error| anyhow::anyhow!(escape_unprintable(&error.to_string())))?;
    if size > GITHUB_WORKFLOW_LIMIT {
        // Advisory only: a closed stderr must not prevent the report.
        let _ = writeln!(
            std::io::stderr().lock(),
            "warning: wsr inspect: {}: {size} bytes exceeds GitHub's documented 500 KB ({GITHUB_WORKFLOW_LIMIT}-byte) workflow file limit; GitHub would not run this workflow",
            escape_unprintable(&file.display().to_string())
        );
    }
    let mut stdout = std::io::stdout().lock();
    match stdout
        .write_all(report.as_bytes())
        .and_then(|()| stdout.flush())
    {
        // The reader closed the pipe (for example `| head -1`); there is nobody left to tell.
        Err(error) if error.kind() == ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| anyhow::anyhow!("wsr inspect: stdout: {error}")),
    }
}

fn escape_unprintable(text: &str) -> String {
    text.chars()
        .map(|c| {
            // `escape_debug` also covers Unicode format characters (bidi overrides and isolates,
            // zero-width spaces), as stdout does; quotes and backslashes stay as written.
            if matches!(c, '\'' | '"' | '\\') {
                c.to_string()
            } else {
                c.escape_debug().to_string()
            }
        })
        .collect()
}

fn render(file: &Path) -> anyhow::Result<(usize, String)> {
    let metadata = std::fs::metadata(file)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    anyhow::ensure!(
        metadata.is_file(),
        "wsr inspect: {}: not a regular file",
        file.display()
    );
    // Re-check the opened handle so a path swapped after the check above is never read.
    let mut handle = std::fs::File::open(file)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    let opened = handle
        .metadata()
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    anyhow::ensure!(
        opened.is_file(),
        "wsr inspect: {}: not a regular file",
        file.display()
    );
    let mut raw = Vec::new();
    handle
        .read_to_end(&mut raw)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    let summary = wsr_gha::inspect(&raw)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    let mut out = String::new();
    writeln!(
        out,
        "Workflow: {}",
        summary
            .name
            .as_deref()
            .unwrap_or("(unnamed)")
            .escape_debug()
    )?;
    if let Some(description) = &summary.description {
        writeln!(out, "Description: {}", description.escape_debug())?;
    }
    match summary.run_name {
        Some(SourceText::Literal(text)) => {
            writeln!(out, "Run name: {} (literal)", text.escape_debug())?
        }
        Some(SourceText::Unevaluated(text)) => writeln!(
            out,
            "Run name: {} (unevaluated; expression syntax not validated)",
            text.escape_debug()
        )?,
        None => {}
    }
    writeln!(
        out,
        "Inspection only; execution compatibility is not assessed."
    )?;
    for trigger in summary.triggers {
        let trigger = match trigger {
            Trigger::Event(name) => {
                writeln!(out, "Trigger: {} (not evaluated)", name.escape_debug())?;
                continue;
            }
            Trigger::Filtered { event, filters } => {
                writeln!(out, "Trigger: {} (not evaluated)", event.escape_debug())?;
                writeln!(out, "  Filters (not matched):")?;
                for filter in filters {
                    writeln!(
                        out,
                        "    {}: [{}]",
                        filter.name.escape_debug(),
                        filter
                            .patterns
                            .iter()
                            .map(|v| format!("{v:?}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
                continue;
            }
            Trigger::Manual(trigger) => trigger,
        };
        writeln!(out, "Trigger: workflow_dispatch (not evaluated)")?;
        if let ManualTrigger::Mapping(Some(body)) = trigger {
            for input in body.inputs.into_iter().flatten() {
                writeln!(out, "  Input: {}", input.id.escape_debug())?;
                if let Some(kind) = input.input_type {
                    let name = match kind {
                        DispatchInputType::String => "string",
                        DispatchInputType::Boolean => "boolean",
                        DispatchInputType::Number => "number",
                        DispatchInputType::Choice => "choice",
                        DispatchInputType::Environment => "environment",
                    };
                    writeln!(out, "    Type: {name}")?;
                }
                if let Some(description) = input.description {
                    writeln!(out, "    Description: {}", description.escape_debug())?;
                }
                if let Some(required) = input.required {
                    writeln!(out, "    Required: {required}")?;
                }
                if let Some(default) = input.default {
                    let value = match default {
                        InputDefault::String(value) => format!("{value:?}"),
                        InputDefault::Boolean(value) => value.to_string(),
                        InputDefault::Number(value) => value,
                    };
                    writeln!(out, "    Default: {value} (not applied)")?;
                }
                if let Some(options) = input.options {
                    writeln!(
                        out,
                        "    Options: [{}]",
                        options
                            .iter()
                            .map(|v| format!("{v:?}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
            }
        }
    }
    if let Some(permissions) = summary.permissions {
        render_permissions(&mut out, "", permissions)?;
    }
    if let Some(env) = summary.env {
        render_values(&mut out, "", "Env (not bound)", env)?;
    }
    if let Some(defaults) = summary.defaults {
        render_defaults(&mut out, "", defaults)?;
    }
    if let Some(concurrency) = summary.concurrency {
        render_concurrency(&mut out, "", concurrency)?;
    }
    for job in summary.jobs {
        write!(out, "Job: {}", job.id.escape_debug())?;
        if let Some(name) = job.name {
            write!(out, " ({})", display_name(name))?;
        }
        writeln!(out)?;
        if let Some(runner) = job.runner {
            render_runner(&mut out, runner)?;
        }
        if !job.needs.is_empty() {
            writeln!(
                out,
                "  Needs: {}",
                job.needs
                    .iter()
                    .map(|id| id.escape_debug().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if let Some(condition) = job.condition {
            writeln!(
                out,
                "  Condition: {} (not evaluated; expression syntax not validated)",
                condition.escape_debug()
            )?;
        }
        if let Some(permissions) = job.permissions {
            render_permissions(&mut out, "  ", permissions)?;
        }
        match job.timeout_minutes {
            Some(SourceText::Literal(minutes)) => {
                writeln!(out, "  Timeout minutes: {minutes} (not applied)")?
            }
            Some(SourceText::Unevaluated(text)) => writeln!(
                out,
                "  Timeout minutes: {} (unevaluated; expression syntax not validated; not applied)",
                text.escape_debug()
            )?,
            None => {}
        }
        render_unapplied(
            &mut out,
            "  ",
            "Cancel timeout minutes",
            job.cancel_timeout_minutes,
        )?;
        render_unapplied(&mut out, "  ", "Continue on error", job.continue_on_error)?;
        if let Some(env) = job.env {
            render_values(&mut out, "  ", "Env (not bound)", env)?;
        }
        if let Some(defaults) = job.defaults {
            render_defaults(&mut out, "  ", defaults)?;
        }
        if let Some(concurrency) = job.concurrency {
            render_concurrency(&mut out, "  ", concurrency)?;
        }
        if let Some(environment) = job.environment {
            render_fields(
                &mut out,
                "  ",
                "Environment (not selected; no protection rules evaluated)",
                [
                    ("Name", Some(environment.name)),
                    ("URL", environment.url),
                    ("Deployment", environment.deployment),
                ],
            )?;
        }
        if let Some(strategy) = job.strategy {
            use wsr_types::inspection::Strategy;
            match strategy {
                Strategy::Unevaluated(text) => writeln!(
                    out,
                    "  Strategy (not expanded): {text:?} (unevaluated; expression syntax not validated)"
                )?,
                Strategy::Mapping {
                    fail_fast,
                    max_parallel,
                    matrix,
                } => render_fields(
                    &mut out,
                    "  ",
                    "Strategy (not expanded)",
                    [
                        ("Fail-fast", fail_fast),
                        ("Max-parallel", max_parallel),
                        ("Matrix (normalized YAML)", matrix),
                    ],
                )?,
            }
        }
        if let Some(container) = job.container {
            render_container(&mut out, container)?;
        }
        if let Some(outputs) = job.outputs {
            let outputs = outputs
                .into_iter()
                .map(|(name, text)| (name, StepValue::Text(text)))
                .collect();
            render_values(&mut out, "  ", "Outputs (not resolved)", outputs)?;
        }
        if let Some(call) = job.call {
            writeln!(
                out,
                "  Calls workflow: {} (not loaded; not resolved)",
                call.uses.escape_debug()
            )?;
            if let Some(with) = call.with {
                render_values(&mut out, "  ", "With (not passed)", with)?;
            }
            match call.secrets {
                Some(CallSecrets::Inherit) => writeln!(out, "  Secrets: inherit (not passed)")?,
                Some(CallSecrets::Values(values)) => {
                    render_values(&mut out, "  ", "Secrets (not passed)", values)?
                }
                None => {}
            }
        }
        for (index, step) in job.steps.into_iter().enumerate() {
            write!(out, "  Step {}", index + 1)?;
            if let Some(id) = step.id {
                write!(out, " [{}]", id.escape_debug())?;
            }
            write!(out, ": ")?;
            match step.kind {
                StepSummaryKind::Run { unevaluated: false } => write!(out, "run")?,
                StepSummaryKind::Run { unevaluated: true } => write!(
                    out,
                    "run (script has unevaluated expressions; not evaluated)"
                )?,
                StepSummaryKind::Uses(reference) => {
                    write!(out, "uses {}", reference.escape_debug())?
                }
            }
            if let Some(name) = step.name {
                write!(out, " ({})", display_name(name))?;
            }
            writeln!(out)?;
            if let Some(condition) = step.condition {
                writeln!(
                    out,
                    "    Condition: {} (not evaluated; expression syntax not validated)",
                    condition.escape_debug()
                )?;
            }
            render_unapplied(
                &mut out,
                "    ",
                "Continue on error",
                step.continue_on_error,
            )?;
            render_unapplied(&mut out, "    ", "Timeout minutes", step.timeout_minutes)?;
            if let Some(shell) = step.shell {
                writeln!(out, "    Shell: {} (not selected)", shell.escape_debug())?;
            }
            match step.working_directory {
                Some(SourceText::Literal(path)) => writeln!(
                    out,
                    "    Working directory: {} (not resolved)",
                    path.escape_debug()
                )?,
                Some(SourceText::Unevaluated(text)) => writeln!(
                    out,
                    "    Working directory: {} (unevaluated; expression syntax not validated; not resolved)",
                    text.escape_debug()
                )?,
                None => {}
            }
            if let Some(env) = step.env {
                render_values(&mut out, "    ", "Env (not bound)", env)?;
            }
            if let Some(with) = step.with {
                render_values(&mut out, "    ", "With (not passed to any action)", with)?;
            }
        }
    }
    Ok((raw.len(), out))
}

fn render_permissions(
    out: &mut String,
    indent: &str,
    permissions: Permissions,
) -> std::fmt::Result {
    match permissions {
        Permissions::ReadAll => writeln!(
            out,
            "{indent}Permissions: read-all (requested; not granted)"
        ),
        Permissions::WriteAll => {
            writeln!(
                out,
                "{indent}Permissions: write-all (requested; not granted)"
            )
        }
        Permissions::Scopes(scopes) if scopes.is_empty() => {
            writeln!(out, "{indent}Permissions: {{}} (requested; not granted)")
        }
        Permissions::Scopes(scopes) => {
            writeln!(out, "{indent}Permissions (requested; not granted):")?;
            for (scope, level) in scopes {
                let level = match level {
                    PermissionLevel::Read => "read",
                    PermissionLevel::Write => "write",
                    PermissionLevel::None => "none",
                };
                writeln!(out, "{indent}  {scope}: {level}")?;
            }
            Ok(())
        }
    }
}

fn render_defaults(out: &mut String, indent: &str, defaults: RunDefaults) -> std::fmt::Result {
    if defaults.shell.is_none() && defaults.working_directory.is_none() {
        return writeln!(out, "{indent}Run defaults (not applied): {{}}");
    }
    writeln!(out, "{indent}Run defaults (not applied):")?;
    for (label, text) in [
        ("Shell", defaults.shell),
        ("Working directory", defaults.working_directory),
    ] {
        match text {
            Some(SourceText::Literal(text)) => {
                writeln!(out, "{indent}  {label}: {}", text.escape_debug())?
            }
            Some(SourceText::Unevaluated(text)) => writeln!(
                out,
                "{indent}  {label}: {} (unevaluated; expression syntax not validated)",
                text.escape_debug()
            )?,
            None => {}
        }
    }
    Ok(())
}

fn render_runner(out: &mut String, runner: RunnerRequirement) -> std::fmt::Result {
    match runner {
        RunnerRequirement::Label(SourceText::Literal(label)) => {
            writeln!(out, "  Runner: {}", label.escape_debug())
        }
        RunnerRequirement::Label(SourceText::Unevaluated(label)) => writeln!(
            out,
            "  Runner: {} (unevaluated; expression syntax not validated)",
            label.escape_debug()
        ),
        RunnerRequirement::Labels(labels) => {
            let unevaluated = labels
                .iter()
                .any(|label| matches!(label, SourceText::Unevaluated(_)));
            write!(
                out,
                "  Runner labels: [{}]",
                labels
                    .iter()
                    .map(
                        |(SourceText::Literal(label) | SourceText::Unevaluated(label))| {
                            format!("{label:?}")
                        }
                    )
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
            if unevaluated {
                write!(out, " (some unevaluated; expression syntax not validated)")?;
            }
            writeln!(out)
        }
        RunnerRequirement::Selector { group, labels } => {
            match group {
                Some(SourceText::Literal(group)) => writeln!(
                    out,
                    "  Runner group: {} (not selected)",
                    group.escape_debug()
                )?,
                Some(SourceText::Unevaluated(group)) => writeln!(
                    out,
                    "  Runner group: {} (unevaluated; expression syntax not validated; not selected)",
                    group.escape_debug()
                )?,
                None => {}
            }
            match labels {
                Some(labels) => render_runner(out, *labels),
                None => Ok(()),
            }
        }
    }
}

fn render_unapplied(
    out: &mut String,
    indent: &str,
    label: &str,
    text: Option<SourceText>,
) -> std::fmt::Result {
    match text {
        Some(SourceText::Literal(text)) => writeln!(
            out,
            "{indent}{label}: {} (not applied)",
            text.escape_debug()
        ),
        Some(SourceText::Unevaluated(text)) => writeln!(
            out,
            "{indent}{label}: {} (unevaluated; expression syntax not validated; not applied)",
            text.escape_debug()
        ),
        None => Ok(()),
    }
}

fn render_concurrency(
    out: &mut String,
    indent: &str,
    concurrency: Concurrency,
) -> std::fmt::Result {
    render_fields(
        out,
        indent,
        "Concurrency (not applied)",
        [
            ("Group", Some(concurrency.group)),
            ("Cancel in progress", concurrency.cancel_in_progress),
            ("Queue", concurrency.queue),
        ],
    )
}

fn render_container(out: &mut String, container: Container) -> std::fmt::Result {
    render_fields(
        out,
        "  ",
        "Container (not pulled; not started)",
        [("Image", Some(container.image))],
    )?;
    if let Some(credentials) = container.credentials {
        render_fields(
            out,
            "    ",
            "Credentials (not used)",
            [("Username", credentials.username)],
        )?;
        // A password is never displayed, even as written.
        if credentials.password.is_some() {
            writeln!(out, "      Password: set (not shown)")?;
        }
    }
    if let Some(env) = container.env {
        render_values(out, "    ", "Env (not bound)", env)?;
    }
    for (label, entries) in [
        ("Ports (not exposed)", container.ports),
        ("Volumes (not mounted)", container.volumes),
    ] {
        let Some(entries) = entries else { continue };
        let unevaluated = entries
            .iter()
            .any(|entry| matches!(entry, StepValue::Text(SourceText::Unevaluated(_))));
        let entries = entries
            .into_iter()
            .map(|entry| match entry {
                StepValue::Text(SourceText::Literal(text) | SourceText::Unevaluated(text)) => {
                    format!("{text:?}")
                }
                StepValue::Boolean(value) => value.to_string(),
                StepValue::Number(value) => value,
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(out, "    {label}: [{entries}]")?;
        if unevaluated {
            write!(out, " (some unevaluated; expression syntax not validated)")?;
        }
        writeln!(out)?;
    }
    match container.options {
        Some(SourceText::Literal(text)) => writeln!(out, "    Options: {text:?} (not passed)")?,
        Some(SourceText::Unevaluated(text)) => writeln!(
            out,
            "    Options: {text:?} (unevaluated; expression syntax not validated; not passed)"
        )?,
        None => {}
    }
    Ok(())
}

/// A titled block of optional source-text lines.
fn render_fields<const N: usize>(
    out: &mut String,
    indent: &str,
    title: &str,
    fields: [(&str, Option<SourceText>); N],
) -> std::fmt::Result {
    writeln!(out, "{indent}{title}:")?;
    for (label, text) in fields {
        match text {
            Some(SourceText::Literal(text)) => {
                writeln!(out, "{indent}  {label}: {}", text.escape_debug())?
            }
            Some(SourceText::Unevaluated(text)) => writeln!(
                out,
                "{indent}  {label}: {} (unevaluated; expression syntax not validated)",
                text.escape_debug()
            )?,
            None => {}
        }
    }
    Ok(())
}

/// A job or step name as shown in parentheses after its heading.
fn display_name(name: SourceText) -> String {
    match name {
        SourceText::Literal(name) => name.escape_debug().to_string(),
        SourceText::Unevaluated(name) => format!(
            "{}; unevaluated; expression syntax not validated",
            name.escape_debug()
        ),
    }
}

fn render_values(
    out: &mut String,
    indent: &str,
    label: &str,
    values: std::collections::BTreeMap<String, StepValue>,
) -> std::fmt::Result {
    if values.is_empty() {
        return writeln!(out, "{indent}{label}: {{}}");
    }
    writeln!(out, "{indent}{label}:")?;
    for (key, value) in values {
        write!(out, "{indent}  {}: ", key.escape_debug())?;
        match value {
            StepValue::Text(SourceText::Literal(text)) => writeln!(out, "{text:?}")?,
            StepValue::Text(SourceText::Unevaluated(text)) => {
                writeln!(out, "{text:?} (unevaluated)")?
            }
            StepValue::Boolean(value) => writeln!(out, "{value}")?,
            StepValue::Number(value) => writeln!(out, "{value}")?,
        }
    }
    Ok(())
}
