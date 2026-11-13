//! Backlog B-19: job `environment` kept as source data, never selected.

use wsr_gha::Category;
use wsr_types::inspection::{Environment, SourceText};

fn job(environment: &str) -> String {
    format!(
        "jobs:\n  b:\n    runs-on: linux\n    environment: {environment}\n    steps:\n      - run: echo\n"
    )
}

fn literal(text: &str) -> SourceText {
    SourceText::Literal(text.into())
}

fn unevaluated(text: &str) -> SourceText {
    SourceText::Unevaluated(text.into())
}

#[test]
fn environments_are_kept_as_source_data() {
    for (value, expected) in [
        (
            "production",
            Environment {
                name: literal("production"),
                url: None,
                deployment: None,
            },
        ),
        (
            "'${{ inputs.target }}'",
            Environment {
                name: unevaluated("${{ inputs.target }}"),
                url: None,
                deployment: None,
            },
        ),
        (
            "{name: staging, url: 'https://example.test', deployment: false}",
            Environment {
                name: literal("staging"),
                url: Some(literal("https://example.test")),
                deployment: Some(literal("false")),
            },
        ),
        (
            "{name: '${{ x }}', url: '${{ steps.d.outputs.url }}', deployment: '${{ y }}'}",
            Environment {
                name: unevaluated("${{ x }}"),
                url: Some(unevaluated("${{ steps.d.outputs.url }}")),
                deployment: Some(unevaluated("${{ y }}")),
            },
        ),
    ] {
        let summary = wsr_gha::inspect(job(value).as_bytes())
            .unwrap_or_else(|error| panic!("{value}: {error}"));
        assert_eq!(
            summary.jobs[0].environment.as_ref(),
            Some(&expected),
            "{value}"
        );
    }
    let without = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    assert_eq!(
        wsr_gha::inspect(without.as_bytes()).unwrap().jobs[0].environment,
        None
    );
}

fn assert_category(environment: &str, category: Category, route: &str) {
    let error = wsr_gha::inspect(job(environment).as_bytes()).unwrap_err();
    assert_eq!(
        (error.category(), error.route()),
        (category, Some(route)),
        "{environment}: {error}"
    );
}

#[test]
fn schema_rejected_environments_are_invalid() {
    for (environment, route) in [
        ("[production]", "jobs.b.environment"),
        ("{url: x}", "jobs.b.environment"),
        ("{name: a, bogus: 1}", "jobs.b.environment.bogus"),
        // The pinned converter asserts a literal string key.
        ("{name: a, '${{ x }}': y}", "jobs.b.environment"),
        ("{name: [a]}", "jobs.b.environment.name"),
        ("{name: a, url: {b: c}}", "jobs.b.environment.url"),
        (
            "{name: a, deployment: 'false'}",
            "jobs.b.environment.deployment",
        ),
        ("{name: a, deployment: 0}", "jobs.b.environment.deployment"),
        ("{name: a, deployment: ~}", "jobs.b.environment.deployment"),
    ] {
        assert_category(environment, Category::Invalid, route);
    }
}

#[test]
fn schema_valid_environments_outside_the_subset_are_unsupported() {
    for (environment, route) in [
        // The pinned reader converts these scalars to strings, and the strings may be empty.
        ("5", "jobs.b.environment"),
        ("true", "jobs.b.environment"),
        ("~", "jobs.b.environment"),
        ("''", "jobs.b.environment"),
        ("{name: 5}", "jobs.b.environment.name"),
        ("{name: ''}", "jobs.b.environment.name"),
        ("{name: a, url: 5}", "jobs.b.environment.url"),
    ] {
        assert_category(environment, Category::Unsupported, route);
    }
}
