use std::fmt::Write as _;
use std::io::{ErrorKind, Read as _, Write as _};
use std::path::Path;

use wsr_types::inspection::{
    DispatchInputType, InputDefault, ManualTrigger, RunnerRequirement, SourceText, StepSummaryKind,
};

pub fn run(file: &Path) -> anyhow::Result<()> {
    // Workflow content is untrusted and reaches diagnostics (serde field names, `on:` keys),
    // so escape control characters before the message can be written to a terminal.
    let report = render(file)
        .map_err(|error| anyhow::anyhow!(escape_control_characters(&error.to_string())))?;
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

fn render(file: &Path) -> anyhow::Result<String> {
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
        }
    }
    Ok(out)
}
