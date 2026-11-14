//! Backlog B-13: the pinned reader converts an unmatched null, boolean or number to a string and
//! validates that string, so a schema-valid conversion is `Unsupported` (valid GHA WSR does not
//! inspect) and a value the schema or the runner rejects is `Invalid`.

mod common;

use std::collections::BTreeMap;

use wsr_gha::Category::{self, Invalid, Unsupported};
use wsr_types::inspection::{SourceText, StepValue};

const JOB: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
const SCALARS: [&str; 4] = ["5", "true", "false", "~"];

fn in_workflow(field: &str) -> String {
    format!("{field}\n{JOB}")
}

fn in_job(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
}

fn in_run_step(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
}

fn in_uses_step(field: &str) -> String {
    format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./a\n        {field}\n")
}

fn dispatch(body: &str) -> String {
    format!("on:\n  workflow_dispatch:\n    inputs:\n      a:\n        {body}\n{JOB}")
}

/// Every case whose category or route differs from the expectation, so one run shows them all.
fn mismatches(cases: Vec<(String, Category, String)>) -> Vec<String> {
    cases
        .into_iter()
        .filter_map(|(source, category, route)| {
            let outcome = match common::inspect(source.as_bytes()) {
                Ok(_) => "accepted".to_owned(),
                Err(error) => format!("{:?} at {:?}", error.category(), error.route()),
            };
            let expected = format!("{category:?} at {:?}", Some(&route));
            (outcome != expected).then(|| format!("{source:?}: expected {expected}, got {outcome}"))
        })
        .collect()
}

fn case(source: String, category: Category, route: &str) -> (String, Category, String) {
    (source, category, route.to_owned())
}

fn assert_all(cases: Vec<(String, Category, String)>) {
    let failures = mismatches(cases);
    assert!(failures.is_empty(), "{:#?}", failures);
}

#[test]
fn name_and_run_name_conversions_are_unsupported_and_null_name_is_absent() {
    let mut cases = Vec::new();
    for value in ["5", "true", "false"] {
        cases.push(case(
            in_workflow(&format!("name: {value}")),
            Unsupported,
            "name",
        ));
        cases.push(case(
            in_job(&format!("name: {value}")),
            Unsupported,
            "jobs.b.name",
        ));
        cases.push(case(
            in_run_step(&format!("name: {value}")),
            Unsupported,
            "jobs.b.steps[1].name",
        ));
    }
    for value in SCALARS {
        cases.push(case(
            in_workflow(&format!("run-name: {value}")),
            Unsupported,
            "run-name",
        ));
    }
    assert_all(cases);
    for source in [
        in_workflow("name: ~"),
        in_job("name: ~"),
        in_run_step("name: ~"),
    ] {
        let summary = common::inspect(source.as_bytes()).unwrap();
        assert_eq!(summary.name, None, "{source}");
        assert_eq!(summary.jobs[0].name, None, "{source}");
        assert_eq!(summary.jobs[0].steps[0].name, None, "{source}");
    }
}

#[test]
fn step_run_and_working_directory_conversions_are_unsupported() {
    let mut cases = Vec::new();
    for value in SCALARS {
        cases.push(case(
            format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: {value}\n"),
            Unsupported,
            "jobs.b.steps[1].run",
        ));
        cases.push(case(
            in_run_step(&format!("working-directory: {value}")),
            Unsupported,
            "jobs.b.steps[1].working-directory",
        ));
    }
    assert_all(cases);
}

#[test]
fn step_shell_conversions_stay_invalid_because_the_runner_rejects_them() {
    assert_all(
        SCALARS
            .iter()
            .map(|value| {
                case(
                    in_run_step(&format!("shell: {value}")),
                    Invalid,
                    "jobs.b.steps[1].shell",
                )
            })
            .collect(),
    );
}

#[test]
fn run_defaults_shell_conversions_are_invalid_and_directory_conversions_unsupported() {
    let mut cases = Vec::new();
    for value in ["5", "true"] {
        cases.push(case(
            in_workflow(&format!("defaults: {{run: {{shell: {value}}}}}")),
            Invalid,
            "defaults.run.shell",
        ));
        cases.push(case(
            in_job(&format!("defaults: {{run: {{shell: {value}}}}}")),
            Invalid,
            "jobs.b.defaults.run.shell",
        ));
        cases.push(case(
            in_workflow(&format!(
                "defaults: {{run: {{working-directory: {value}}}}}"
            )),
            Unsupported,
            "defaults.run.working-directory",
        ));
        cases.push(case(
            in_job(&format!(
                "defaults: {{run: {{working-directory: {value}}}}}"
            )),
            Unsupported,
            "jobs.b.defaults.run.working-directory",
        ));
    }
    // A null is an empty string, and both fields require a nonempty one.
    for key in ["shell", "working-directory"] {
        cases.push(case(
            in_workflow(&format!("defaults: {{run: {{{key}: ~}}}}")),
            Invalid,
            &format!("defaults.run.{key}"),
        ));
        cases.push(case(
            in_job(&format!("defaults: {{run: {{{key}: ~}}}}")),
            Invalid,
            &format!("jobs.b.defaults.run.{key}"),
        ));
    }
    assert_all(cases);
}

#[test]
fn runner_conversions_are_unsupported_and_nulls_stay_invalid() {
    let mut cases = Vec::new();
    for value in ["5", "true", "false"] {
        for (field, route) in [
            (format!("runs-on: {value}"), "jobs.b.runs-on"),
            (format!("runs-on: [linux, {value}]"), "jobs.b.runs-on[2]"),
            (format!("runs-on: {{group: {value}}}"), "jobs.b.runs-on"),
            (format!("runs-on: {{labels: {value}}}"), "jobs.b.runs-on"),
            (
                format!("runs-on: {{labels: [a, {value}]}}"),
                "jobs.b.runs-on",
            ),
            (
                format!("runs-on: {{group: g, labels: [{value}]}}"),
                "jobs.b.runs-on",
            ),
        ] {
            cases.push(case(
                format!("jobs:\n  b:\n    {field}\n    steps:\n      - run: echo\n"),
                Unsupported,
                route,
            ));
        }
    }
    for (field, route) in [
        ("runs-on: ~", "jobs.b.runs-on"),
        ("runs-on: [linux, ~]", "jobs.b.runs-on[2]"),
        ("runs-on: {group: ~}", "jobs.b.runs-on"),
        ("runs-on: {labels: ~}", "jobs.b.runs-on"),
        ("runs-on: {labels: [a, ~]}", "jobs.b.runs-on"),
    ] {
        cases.push(case(
            format!("jobs:\n  b:\n    {field}\n    steps:\n      - run: echo\n"),
            Invalid,
            route,
        ));
    }
    assert_all(cases);
}

#[test]
fn dispatch_description_and_choice_option_conversions_are_unsupported() {
    let mut cases = Vec::new();
    for value in SCALARS {
        cases.push(case(
            dispatch(&format!("description: {value}")),
            Unsupported,
            "on.workflow_dispatch.inputs.a.description",
        ));
        cases.push(case(
            dispatch(&format!("type: choice\n        options: [x, {value}]")),
            Unsupported,
            "on.workflow_dispatch.inputs.a.options[2]",
        ));
    }
    assert_all(cases);
}

#[test]
fn dispatch_type_and_required_conversions_stay_invalid() {
    let mut cases = Vec::new();
    for value in SCALARS {
        cases.push(case(
            dispatch(&format!("type: {value}")),
            Invalid,
            "on.workflow_dispatch.inputs.a.type",
        ));
    }
    for value in ["5", "~", "'true'"] {
        cases.push(case(
            dispatch(&format!("required: {value}")),
            Invalid,
            "on.workflow_dispatch.inputs.a.required",
        ));
    }
    assert_all(cases);
}

#[test]
fn dispatch_default_conversions_depend_on_the_declared_input_type() {
    let route = "on.workflow_dispatch.inputs.a.default";
    let choice = "type: choice\n        options: [x]\n        ";
    let mut cases = Vec::new();
    // A number or boolean is a type mismatch for a string-like input.
    for value in ["5", "true"] {
        for prefix in [
            "",
            "type: string\n        ",
            choice,
            "type: environment\n        ",
        ] {
            cases.push(case(
                dispatch(&format!("{prefix}default: {value}")),
                Invalid,
                route,
            ));
        }
    }
    // A null converts to an empty string, which a string-like or number input holds.
    for prefix in [
        "",
        "type: string\n        ",
        choice,
        "type: environment\n        ",
        "type: number\n        ",
    ] {
        cases.push(case(
            dispatch(&format!("{prefix}default: ~")),
            Unsupported,
            route,
        ));
    }
    for value in ["~", "5", "'false'"] {
        cases.push(case(
            dispatch(&format!("type: boolean\n        default: {value}")),
            Invalid,
            route,
        ));
    }
    assert_all(cases);
}

#[test]
fn env_and_with_null_values_are_unsupported_and_numbers_and_booleans_stay_accepted() {
    assert_all(vec![
        case(in_workflow("env: {A: ~}"), Unsupported, "env.A"),
        case(in_job("env: {A: ~}"), Unsupported, "jobs.b.env.A"),
        case(
            in_run_step("env: {A: ~}"),
            Unsupported,
            "jobs.b.steps[1].env.A",
        ),
        case(
            in_uses_step("with: {a: ~}"),
            Unsupported,
            "jobs.b.steps[1].with.a",
        ),
        case(
            in_run_step("env: {A: [1]}"),
            Invalid,
            "jobs.b.steps[1].env.A",
        ),
        case(
            in_uses_step("with: {a: {b: c}}"),
            Invalid,
            "jobs.b.steps[1].with.a",
        ),
    ]);
    let summary = common::inspect(
        in_uses_step("with: {n: 5, s: '5', b: true}\n        env: {n: 5, s: '5', b: true}")
            .as_bytes(),
    )
    .unwrap();
    let step = &summary.jobs[0].steps[0];
    let expected = || {
        Some(BTreeMap::from([
            ("b".to_owned(), StepValue::Boolean(true)),
            ("n".to_owned(), StepValue::Number("5".into())),
            (
                "s".to_owned(),
                StepValue::Text(SourceText::Literal("5".into())),
            ),
        ]))
    };
    assert_eq!(step.with, expected());
    assert_eq!(step.env, expected());
}

#[test]
fn step_id_booleans_convert_to_identifiers_but_numbers_and_null_stay_invalid() {
    assert_all(vec![
        case(in_run_step("id: true"), Unsupported, "jobs.b.steps[1].id"),
        case(in_run_step("id: false"), Unsupported, "jobs.b.steps[1].id"),
        case(in_run_step("id: 5"), Invalid, "jobs.b.steps[1].id"),
        case(in_run_step("id: ~"), Invalid, "jobs.b.steps[1].id"),
    ]);
}

#[test]
fn conditions_identifiers_references_and_timeouts_keep_their_categories() {
    let mut cases = Vec::new();
    for value in SCALARS {
        cases.push(case(
            in_job(&format!("if: {value}")),
            Unsupported,
            "jobs.b.if",
        ));
        cases.push(case(
            in_run_step(&format!("if: {value}")),
            Unsupported,
            "jobs.b.steps[1].if",
        ));
        cases.push(case(
            format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: {value}\n"),
            Invalid,
            "jobs.b.steps[1].uses",
        ));
    }
    cases.push(case(
        in_job("timeout-minutes: '5'"),
        Invalid,
        "jobs.b.timeout-minutes",
    ));
    for id in ["5", "~"] {
        cases.push(case(
            format!("jobs:\n  {id}:\n    runs-on: linux\n    steps:\n      - run: echo\n"),
            Invalid,
            "jobs",
        ));
    }
    assert_all(cases);
}

#[test]
fn quoted_scalars_keep_their_source_text_apart_from_unquoted_ones() {
    let summary = common::inspect(
        format!(
            "run-name: '5'\nname: '5'\n{}",
            "jobs:\n  b:\n    name: '1'\n    runs-on: '2'\n    steps:\n      - run: '3'\n        name: '4'\n        working-directory: '6'\n"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(summary.run_name, Some(SourceText::Literal("5".into())));
    assert_eq!(summary.name.as_deref(), Some("5"));
    let job = &summary.jobs[0];
    assert_eq!(job.name, Some(SourceText::Literal("1".into())));
    assert_eq!(job.steps[0].name, Some(SourceText::Literal("4".into())));
    assert_eq!(
        job.steps[0].working_directory,
        Some(SourceText::Literal("6".into()))
    );
}
