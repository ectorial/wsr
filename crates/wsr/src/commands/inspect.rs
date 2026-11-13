use std::path::Path;

use wsr_types::inspection::{
    DispatchInputType, InputDefault, ManualTrigger, RunnerRequirement, SourceText, StepSummaryKind,
};

pub fn run(file: &Path) -> anyhow::Result<()> {
    let raw = std::fs::read(file)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    let summary = wsr_gha::inspect(&raw)
        .map_err(|error| anyhow::anyhow!("wsr inspect: {}: {error}", file.display()))?;
    println!(
        "Workflow: {}",
        summary
            .name
            .as_deref()
            .unwrap_or("(unnamed)")
            .escape_debug()
    );
    match summary.run_name {
        Some(SourceText::Literal(text)) => println!("Run name: {} (literal)", text.escape_debug()),
        Some(SourceText::Unevaluated(text)) => println!(
            "Run name: {} (unevaluated; expression syntax not validated)",
            text.escape_debug()
        ),
        None => {}
    }
    println!("Inspection only; execution compatibility is not assessed.");
    if let Some(trigger) = summary.trigger {
        println!("Trigger: workflow_dispatch (not evaluated)");
        if let ManualTrigger::Mapping(Some(body)) = trigger {
            for input in body.inputs.into_iter().flatten() {
                println!("  Input: {}", input.id.escape_debug());
                if let Some(kind) = input.input_type {
                    let name = match kind {
                        DispatchInputType::String => "string",
                        DispatchInputType::Boolean => "boolean",
                        DispatchInputType::Number => "number",
                        DispatchInputType::Choice => "choice",
                        DispatchInputType::Environment => "environment",
                    };
                    println!("    Type: {name}");
                }
                if let Some(description) = input.description {
                    println!("    Description: {}", description.escape_debug());
                }
                if let Some(required) = input.required {
                    println!("    Required: {required}");
                }
                if let Some(default) = input.default {
                    let value = match default {
                        InputDefault::String(value) => format!("{value:?}"),
                        InputDefault::Boolean(value) => value.to_string(),
                        InputDefault::Number(value) => value,
                    };
                    println!("    Default: {value} (not applied)");
                }
                if let Some(options) = input.options {
                    println!(
                        "    Options: [{}]",
                        options
                            .iter()
                            .map(|v| format!("{v:?}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }
    for job in summary.jobs {
        print!("Job: {}", job.id.escape_debug());
        if let Some(name) = job.name {
            print!(" ({})", name.escape_debug());
        }
        match job.runner {
            RunnerRequirement::Label(label) => println!("\n  Runner: {}", label.escape_debug()),
            RunnerRequirement::Labels(labels) => println!(
                "\n  Runner labels: [{}]",
                labels
                    .iter()
                    .map(|label| format!("{label:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
        if !job.needs.is_empty() {
            println!(
                "  Needs: {}",
                job.needs
                    .iter()
                    .map(|id| id.escape_debug().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        for (index, step) in job.steps.into_iter().enumerate() {
            print!("  Step {}", index + 1);
            if let Some(id) = step.id {
                print!(" [{}]", id.escape_debug());
            }
            print!(": ");
            match step.kind {
                StepSummaryKind::Run => print!("run"),
                StepSummaryKind::Uses(reference) => print!("uses {}", reference.escape_debug()),
            }
            if let Some(name) = step.name {
                print!(" ({})", name.escape_debug());
            }
            println!();
        }
    }
    Ok(())
}
