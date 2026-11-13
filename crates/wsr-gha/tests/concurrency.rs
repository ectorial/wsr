//! Backlog B-16: workflow and job `concurrency` kept as source data, never applied.

use wsr_gha::Category;
use wsr_types::inspection::{Concurrency, SourceText};

const STEPS: &str = "    runs-on: linux\n    steps:\n      - run: echo\n";

fn workflow(value: &str) -> String {
    format!("concurrency: {value}\njobs:\n  b:\n{STEPS}")
}

fn job(value: &str) -> String {
    format!("jobs:\n  b:\n    concurrency: {value}\n{STEPS}")
}

fn literal(text: &str) -> SourceText {
    SourceText::Literal(text.into())
}

fn unevaluated(text: &str) -> SourceText {
    SourceText::Unevaluated(text.into())
}

#[test]
fn concurrency_is_kept_at_both_levels() {
    let cases = [
        (
            "ci",
            Concurrency {
                group: literal("ci"),
                cancel_in_progress: None,
                queue: None,
            },
        ),
        (
            "'ci-${{ github.ref }}'",
            Concurrency {
                group: unevaluated("ci-${{ github.ref }}"),
                cancel_in_progress: None,
                queue: None,
            },
        ),
        (
            "{group: ci, cancel-in-progress: true, queue: max}",
            Concurrency {
                group: literal("ci"),
                cancel_in_progress: Some(literal("true")),
                queue: Some(literal("max")),
            },
        ),
        (
            "{group: '${{ x }}', cancel-in-progress: '${{ y }}', queue: '${{ z }}'}",
            Concurrency {
                group: unevaluated("${{ x }}"),
                cancel_in_progress: Some(unevaluated("${{ y }}")),
                queue: Some(unevaluated("${{ z }}")),
            },
        ),
        (
            "{group: ci, cancel-in-progress: false, queue: single}",
            Concurrency {
                group: literal("ci"),
                cancel_in_progress: Some(literal("false")),
                queue: Some(literal("single")),
            },
        ),
    ];
    for (value, expected) in cases {
        let summary = wsr_gha::inspect(workflow(value).as_bytes())
            .unwrap_or_else(|error| panic!("{value}: {error}"));
        assert_eq!(summary.concurrency.as_ref(), Some(&expected), "{value}");
        assert_eq!(summary.jobs[0].concurrency, None);
        let summary = wsr_gha::inspect(job(value).as_bytes())
            .unwrap_or_else(|error| panic!("job {value}: {error}"));
        assert_eq!(
            summary.jobs[0].concurrency.as_ref(),
            Some(&expected),
            "{value}"
        );
        assert_eq!(summary.concurrency, None);
    }
}

fn assert_category(source: String, category: Category, route: &str) {
    let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
    assert_eq!(
        (error.category(), error.route()),
        (category, Some(route)),
        "{source}: {error}"
    );
}

#[test]
fn schema_rejected_concurrency_is_invalid() {
    for (value, field) in [
        ("[ci]", ""),
        ("{cancel-in-progress: true}", ""),
        ("{group: ''}", ".group"),
        ("{group: ~}", ".group"),
        ("{group: [a]}", ".group"),
        (
            "{group: ci, cancel-in-progress: 'true'}",
            ".cancel-in-progress",
        ),
        ("{group: ci, cancel-in-progress: 1}", ".cancel-in-progress"),
        ("{group: ci, cancel-in-progress: ~}", ".cancel-in-progress"),
        ("{group: ci, queue: many}", ".queue"),
        ("{group: ci, queue: 5}", ".queue"),
        ("{group: ci, bogus: 1}", ".bogus"),
        // The pinned converter asserts a literal string key before any expression guard.
        ("{group: ci, '${{ x }}': y}", ""),
    ] {
        assert_category(
            workflow(value),
            Category::Invalid,
            &format!("concurrency{field}"),
        );
        assert_category(
            job(value),
            Category::Invalid,
            &format!("jobs.b.concurrency{field}"),
        );
    }
    // job-concurrency's string form is a non-empty-string; workflow-concurrency's is any string.
    assert_category(job("''"), Category::Invalid, "jobs.b.concurrency");
    assert_category(job("~"), Category::Invalid, "jobs.b.concurrency");
}

#[test]
fn schema_valid_concurrency_outside_the_subset_is_unsupported() {
    for (value, field) in [
        ("5", ""),
        ("true", ""),
        ("' '", ""),
        ("{group: 5}", ".group"),
        ("{group: ' '}", ".group"),
    ] {
        assert_category(
            workflow(value),
            Category::Unsupported,
            &format!("concurrency{field}"),
        );
        assert_category(
            job(value),
            Category::Unsupported,
            &format!("jobs.b.concurrency{field}"),
        );
    }
    assert_category(workflow("''"), Category::Unsupported, "concurrency");
    assert_category(workflow("~"), Category::Unsupported, "concurrency");
}
