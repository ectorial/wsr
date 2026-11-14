//! B-35: `runs-on` group/labels mappings are retained as written; no runner is selected.
mod common;

use wsr_gha::Category::{Invalid, Unsupported};
use wsr_types::inspection::{RunnerRequirement, SourceText};

fn job(runs_on: &str) -> String {
    format!("jobs:\n  a:\n    runs-on: {runs_on}\n    steps: [{{run: echo}}]\n")
}

fn literal(text: &str) -> SourceText {
    SourceText::Literal(text.into())
}

fn unevaluated(text: &str) -> SourceText {
    SourceText::Unevaluated(text.into())
}

fn selector(group: Option<SourceText>, labels: Option<RunnerRequirement>) -> RunnerRequirement {
    RunnerRequirement::Selector {
        group,
        labels: labels.map(Box::new),
    }
}

#[test]
fn selectors_are_retained_as_written() {
    for (source, expected) in [
        ("{group: fleet}", selector(Some(literal("fleet")), None)),
        (
            "{group: organization/fleet, labels: gpu}",
            selector(
                Some(literal("organization/fleet")),
                Some(RunnerRequirement::Label(literal("gpu"))),
            ),
        ),
        (
            "{labels: [linux, x64, linux]}",
            selector(
                None,
                Some(RunnerRequirement::Labels(vec![
                    literal("linux"),
                    literal("x64"),
                    literal("linux"),
                ])),
            ),
        ),
        (
            "{group: ent/a, labels: []}",
            selector(
                Some(literal("ent/a")),
                Some(RunnerRequirement::Labels(vec![])),
            ),
        ),
        (
            "{group: '${{ vars.g }}', labels: ['${{ matrix.os }}', x]}",
            selector(
                Some(unevaluated("${{ vars.g }}")),
                Some(RunnerRequirement::Labels(vec![
                    unevaluated("${{ matrix.os }}"),
                    literal("x"),
                ])),
            ),
        ),
        // The converter skips an expression group, so its slashes are not checked.
        (
            "{group: 'a/b/${{ x }}', labels: '${{ x }}'}",
            selector(
                Some(unevaluated("a/b/${{ x }}")),
                Some(RunnerRequirement::Label(unevaluated("${{ x }}"))),
            ),
        ),
    ] {
        let summary = common::inspect(job(source).as_bytes())
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(summary.jobs[0].runner, Some(expected), "{source}");
    }
}

#[test]
fn selector_categories_follow_the_pinned_schema_and_converter() {
    for (source, category, route) in [
        ("{}", Unsupported, "jobs.a.runs-on"),
        ("{group: 1}", Unsupported, "jobs.a.runs-on.group"),
        ("{labels: true}", Unsupported, "jobs.a.runs-on.labels"),
        ("{labels: [a, 5]}", Unsupported, "jobs.a.runs-on.labels[2]"),
        ("{group: g, os: linux}", Invalid, "jobs.a.runs-on.os"),
        ("{1: g}", Invalid, "jobs.a.runs-on"),
        ("{group: ~}", Invalid, "jobs.a.runs-on.group"),
        ("{group: ''}", Invalid, "jobs.a.runs-on.group"),
        ("{group: '  '}", Invalid, "jobs.a.runs-on.group"),
        ("{group: [a]}", Invalid, "jobs.a.runs-on.group"),
        ("{labels: ~}", Invalid, "jobs.a.runs-on.labels"),
        ("{labels: ''}", Invalid, "jobs.a.runs-on.labels"),
        ("{labels: {a: b}}", Invalid, "jobs.a.runs-on.labels"),
        ("{labels: [a, ~]}", Invalid, "jobs.a.runs-on.labels[2]"),
        ("{labels: [a, ' ']}", Invalid, "jobs.a.runs-on.labels[2]"),
        ("{labels: [[a]]}", Invalid, "jobs.a.runs-on.labels[1]"),
        ("{group: team/fleet}", Invalid, "jobs.a.runs-on.group"),
        ("{group: org/}", Invalid, "jobs.a.runs-on.group"),
        ("{group: org/a/b}", Invalid, "jobs.a.runs-on.group"),
    ] {
        let Err(error) = common::inspect(job(source).as_bytes()) else {
            panic!("accepted runs-on: {source}");
        };
        assert_eq!(
            (error.category(), error.route()),
            (category, Some(route)),
            "{source}: {error}"
        );
    }
}
