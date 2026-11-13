use std::fmt::Write as _;
use std::io::{ErrorKind, Read as _, Write as _};
use std::path::Path;

use wsr_types::inspection::{
    DispatchInputType, InputDefault, ManualTrigger, PermissionLevel, Permissions,
    RunnerRequirement, SourceText, StepSummaryKind, StepValue,
};

// GitHub documents a 500 KB workflow file limit (D-032); 512,000 bytes is WSR's reading of it.
// WSR warns but imposes no limit of its own.
const GITHUB_WORKFLOW_LIMIT: usize = 512_000;

pub fn run(file: &Path) -> anyhow::Result<()> {
    // Workflow content is untrusted and reaches diagnostics (serde field names, `on:` keys),
    // so escape control characters before the message can be written to a terminal.
    let (size, report) = render(file)
        .map_err(|error| anyhow::anyhow!(escape_control_characters(&error.to_string())))?;
    if size > GITHUB_WORKFLOW_LIMIT {
        // Advisory only: a closed stderr must not prevent the report.
        let _ = writeln!(
            std::io::stderr().lock(),
            "warning: wsr inspect: {}: {size} bytes exceeds GitHub's documented 500 KB ({GITHUB_WORKFLOW_LIMIT}-byte) workflow file limit; GitHub would not run this workflow",
            escape_control_characters(&file.display().to_string())
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

fn escape_control_characters(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() {
                c.escape_debug().to_string()
            } else {
                c.to_string()
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
    if let Some(trigger) = summary.trigger {
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
    for job in summary.jobs {
        write!(out, "Job: {}", job.id.escape_debug())?;
        if let Some(name) = job.name {
            write!(out, " ({})", name.escape_debug())?;
        }
        match job.runner {
            RunnerRequirement::Label(label) => {
                writeln!(out, "\n  Runner: {}", label.escape_debug())?
            }
            RunnerRequirement::Labels(labels) => writeln!(
                out,
                "\n  Runner labels: [{}]",
                labels
                    .iter()
                    .map(|label| format!("{label:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
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
        for (index, step) in job.steps.into_iter().enumerate() {
            write!(out, "  Step {}", index + 1)?;
            if let Some(id) = step.id {
                write!(out, " [{}]", id.escape_debug())?;
            }
            write!(out, ": ")?;
            match step.kind {
                StepSummaryKind::Run => write!(out, "run")?,
                StepSummaryKind::Uses(reference) => {
                    write!(out, "uses {}", reference.escape_debug())?
                }
            }
            if let Some(name) = step.name {
                write!(out, " ({})", name.escape_debug())?;
            }
            writeln!(out)?;
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
                render_values(&mut out, "Env (not bound)", env)?;
            }
            if let Some(with) = step.with {
                render_values(&mut out, "With (not passed to any action)", with)?;
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

fn render_values(
    out: &mut String,
    label: &str,
    values: std::collections::BTreeMap<String, StepValue>,
) -> std::fmt::Result {
    if values.is_empty() {
        return writeln!(out, "    {label}: {{}}");
    }
    writeln!(out, "    {label}:")?;
    for (key, value) in values {
        write!(out, "      {}: ", key.escape_debug())?;
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
