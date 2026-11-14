//! Backlog B-17: job and step `continue-on-error` and step `timeout-minutes`, never applied.

mod common;

use wsr_gha::Category;
use wsr_types::inspection::SourceText;

fn job(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
}

fn step(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./a\n        {field}\n")
}

fn literal(text: &str) -> Option<SourceText> {
    Some(SourceText::Literal(text.into()))
}

fn unevaluated(text: &str) -> Option<SourceText> {
    Some(SourceText::Unevaluated(text.into()))
}

#[test]
fn values_are_kept_as_source_data() {
    for (value, expected) in [
        ("true", literal("true")),
        ("false", literal("false")),
        (
            "${{ matrix.experimental }}",
            unevaluated("${{ matrix.experimental }}"),
        ),
    ] {
        let field = format!("continue-on-error: {value}");
        let summary = common::inspect(job(&field).as_bytes()).unwrap();
        assert_eq!(summary.jobs[0].continue_on_error, expected, "{value}");
        let summary = common::inspect(step(&field).as_bytes()).unwrap();
        assert_eq!(
            summary.jobs[0].steps[0].continue_on_error, expected,
            "{value}"
        );
    }
    for (value, expected) in [
        ("5", literal("5")),
        ("0.5", literal("0.5")),
        (
            "${{ inputs.minutes }}",
            unevaluated("${{ inputs.minutes }}"),
        ),
    ] {
        let summary =
            common::inspect(step(&format!("timeout-minutes: {value}")).as_bytes()).unwrap();
        assert_eq!(
            summary.jobs[0].steps[0].timeout_minutes, expected,
            "{value}"
        );
    }
    let summary = common::inspect(job("name: x").as_bytes()).unwrap();
    assert_eq!(summary.jobs[0].continue_on_error, None);
    assert_eq!(summary.jobs[0].steps[0].continue_on_error, None);
    assert_eq!(summary.jobs[0].steps[0].timeout_minutes, None);
}

fn assert_category(source: String, category: Category, route: &str) {
    let error = common::inspect(source.as_bytes()).unwrap_err();
    assert_eq!(
        (error.category(), error.route()),
        (category, Some(route)),
        "{source}: {error}"
    );
}

#[test]
fn non_boolean_and_non_number_values_are_invalid() {
    for value in ["5", "'true'", "~", "[true]", "yes-please"] {
        let field = format!("continue-on-error: {value}");
        assert_category(job(&field), Category::Invalid, "jobs.b.continue-on-error");
        assert_category(
            step(&field),
            Category::Invalid,
            "jobs.b.steps[1].continue-on-error",
        );
    }
    for value in ["true", "'5'", "~", ".inf", "[1]"] {
        assert_category(
            step(&format!("timeout-minutes: {value}")),
            Category::Invalid,
            "jobs.b.steps[1].timeout-minutes",
        );
    }
    // As for the job timeout, a non-positive number is schema-valid but outside the subset.
    assert_category(
        step("timeout-minutes: 0"),
        Category::Unsupported,
        "jobs.b.steps[1].timeout-minutes",
    );
}
