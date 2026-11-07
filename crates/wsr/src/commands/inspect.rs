use std::path::Path;

use wsr_types::inspection::StepSummaryKind;

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
    println!("Inspection only; execution compatibility is not assessed.");
    if let Some(trigger) = summary.trigger {
        println!("Trigger: {} (not evaluated)", trigger.escape_debug());
    }
    for job in summary.jobs {
        print!("Job: {}", job.id.escape_debug());
        if let Some(name) = job.name {
            print!(" ({})", name.escape_debug());
        }
        println!("\n  Runner: {}", job.runner.escape_debug());
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
            print!("  Step {}: ", index + 1);
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
