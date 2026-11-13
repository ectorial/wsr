//! Backlog B-14: literal `shell` and `uses` text the runner rejects is invalid source.

use wsr_gha::Category;
use wsr_types::inspection::StepSummaryKind;

fn run_step(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
}

fn uses_step(reference: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: {reference}\n")
}

fn defaults(level: &str, shell: &str) -> String {
    let body = format!("defaults: {{run: {{shell: {shell}}}}}");
    match level {
        "workflow" => {
            format!("{body}\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n")
        }
        _ => {
            format!("jobs:\n  b:\n    runs-on: linux\n    {body}\n    steps:\n      - run: echo\n")
        }
    }
}

#[test]
fn shells_that_are_neither_built_in_nor_format_strings_are_invalid() {
    for shell in ["'true'", "'5'", "perl", "bash -e", "Bash"] {
        for (source, route) in [
            (
                run_step(&format!("shell: {shell}")),
                "jobs.b.steps[1].shell",
            ),
            (defaults("workflow", shell), "defaults.run.shell"),
            (defaults("job", shell), "jobs.b.defaults.run.shell"),
        ] {
            let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
            assert_eq!(
                (error.category(), error.route()),
                (Category::Invalid, Some(route)),
                "{source}: {error}"
            );
        }
    }
}

#[test]
fn built_in_shells_and_format_strings_are_kept() {
    for shell in [
        "bash",
        "pwsh",
        "python",
        "sh",
        "cmd",
        "powershell",
        "perl {0}",
        "bash -e {0}",
    ] {
        let summary = wsr_gha::inspect(run_step(&format!("shell: '{shell}'")).as_bytes())
            .unwrap_or_else(|error| panic!("{shell}: {error}"));
        assert_eq!(summary.jobs[0].steps[0].shell.as_deref(), Some(shell));
        for level in ["workflow", "job"] {
            wsr_gha::inspect(defaults(level, &format!("'{shell}'")).as_bytes())
                .unwrap_or_else(|error| panic!("{level} {shell}: {error}"));
        }
    }
}

#[test]
fn malformed_action_references_are_invalid() {
    for reference in [
        "'5'",
        "actions/checkout",
        "checkout@v4",
        "actions/@v4",
        "actions/checkout@",
        "/checkout@v4",
        "actions//checkout@v4",
        "a/b@c@d",
        "docker://",
        "action",
    ] {
        let error = wsr_gha::inspect(uses_step(reference).as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some("jobs.b.steps[1].uses")),
            "{reference}: {error}"
        );
    }
}

#[test]
fn well_formed_action_references_are_reported_verbatim() {
    for reference in [
        "./action",
        "./.github/actions/build",
        "docker://alpine:3.20",
        "actions/checkout@v4",
        "actions/aws/ec2@main",
        "owner/repo@3d3c42e5aac5ba805825da76410c181273ba90b1",
    ] {
        let summary = wsr_gha::inspect(uses_step(reference).as_bytes())
            .unwrap_or_else(|error| panic!("{reference}: {error}"));
        assert!(
            matches!(&summary.jobs[0].steps[0].kind, StepSummaryKind::Uses(kept) if kept == reference),
            "{reference}"
        );
    }
}
