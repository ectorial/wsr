use std::collections::BTreeMap;

use wsr_gha::Category;
use wsr_types::inspection::{SourceText, StepValue};

#[test]
fn step_inputs_keep_source_values_and_types() {
    let summary = wsr_gha::inspect(
        b"jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./a\n        with: {flag: false, n: 3, v: '3', t: '${{ x }}'}\n        env: {}\n      - run: echo\n        working-directory: '${{ y }}'\n        env: {A: b}\n",
    )
    .unwrap();
    let steps = &summary.jobs[0].steps;
    assert_eq!(
        steps[0].with,
        Some(BTreeMap::from([
            ("flag".to_owned(), StepValue::Boolean(false)),
            ("n".to_owned(), StepValue::Number("3".into())),
            (
                "t".to_owned(),
                StepValue::Text(SourceText::Unevaluated("${{ x }}".into()))
            ),
            (
                "v".to_owned(),
                StepValue::Text(SourceText::Literal("3".into()))
            ),
        ]))
    );
    assert_eq!(steps[0].env, Some(BTreeMap::new()));
    assert_eq!(steps[0].working_directory, None);
    assert_eq!(steps[1].with, None);
    assert_eq!(
        steps[1].env,
        Some(BTreeMap::from([(
            "A".to_owned(),
            StepValue::Text(SourceText::Literal("b".into()))
        )]))
    );
    assert_eq!(
        steps[1].working_directory,
        Some(SourceText::Unevaluated("${{ y }}".into()))
    );
}

#[test]
fn step_input_failures_have_category_and_route() {
    let step = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
    };
    for (source, expected) in [
        (
            step("env: '${{ x }}'"),
            (Category::Unsupported, "jobs.b.steps[1].env"),
        ),
        (
            // B-13: the pinned reader converts a null value to an empty string.
            step("env: {A: ~}"),
            (Category::Unsupported, "jobs.b.steps[1].env.A"),
        ),
        (
            step("env: {1: x}"),
            (Category::Unsupported, "jobs.b.steps[1].env"),
        ),
        (
            step("env: {a: x, A: y}"),
            (Category::Invalid, "jobs.b.steps[1].env.A"),
        ),
        (
            step("with: '${{ x }}'"),
            (Category::Invalid, "jobs.b.steps[1].with"),
        ),
        (
            step("with: {a: b}"),
            (Category::Invalid, "jobs.b.steps[1].with"),
        ),
    ] {
        let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (expected.0, Some(expected.1)),
            "{source}: {error}"
        );
    }
}

#[test]
fn duplicate_step_input_keys_keep_decoder_evidence() {
    let error = wsr_gha::inspect(
        b"jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        env: {A: b, A: c}\n",
    )
    .unwrap_err();
    assert_eq!(error.category(), Category::Invalid);
    assert!(
        error.to_string().contains("duplicate entry with key \"A\""),
        "{error}"
    );
}

// The pinned schema types `run` and `working-directory` as string-steps-context, which allows
// empty strings; they are valid GHA that WSR does not inspect, not invalid source.
#[test]
fn empty_run_and_working_directory_are_schema_valid_but_unsupported() {
    for (source, route) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: ''\n",
            "jobs.b.steps[1].run",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: '  '\n",
            "jobs.b.steps[1].run",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        working-directory: ''\n",
            "jobs.b.steps[1].working-directory",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        working-directory: '  '\n",
            "jobs.b.steps[1].working-directory",
        ),
    ] {
        let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Unsupported, Some(route)),
            "{source}: {error}"
        );
    }
}
