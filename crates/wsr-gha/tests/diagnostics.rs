use wsr_gha::Category;

const JOB: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

fn diagnose(source: &str) -> (Category, Option<String>) {
    let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
    (error.category(), error.route().map(str::to_owned))
}

#[test]
fn valid_github_actions_source_outside_the_subset_is_unsupported_at_its_route() {
    for (source, route) in [
        (
            "jobs:\n  b:\n    if: true\n    runs-on: linux\n    steps:\n      - run: echo\n"
                .to_owned(),
            "jobs.b.if",
        ),
        (format!("on: {{push: null}}\n{JOB}"), "on.push"),
        (format!("on: [push]\n{JOB}"), "on"),
        (format!("on: push\n{JOB}"), "on"),
        (format!("concurrency: ci\n{JOB}"), "concurrency"),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n      - timeout-minutes: 1\n        run: b\n"
                .to_owned(),
            "jobs.b.steps[2].timeout-minutes",
        ),
        (
            "jobs:\n  b:\n    runs-on: {group: fleet}\n    steps:\n      - run: echo\n".to_owned(),
            "jobs.b.runs-on",
        ),
        (
            "jobs:\n  b:\n    runs-on: {group: fleet, labels: [a]}\n    steps:\n      - run: echo\n"
                .to_owned(),
            "jobs.b.runs-on",
        ),
    ] {
        assert_eq!(
            diagnose(&source),
            (Category::Unsupported, Some(route.to_owned())),
            "{source}"
        );
    }
}

fn dispatch_input(body: &str) -> String {
    format!("on:\n  workflow_dispatch:\n    inputs:\n      a:\n        {body}\n{JOB}")
}

// The pinned schema (actions/languageservices @ 4043eda, workflow-v1.0.json) gives these
// definitions no `context`, so no expression is allowed: workflow-name, `on`, job-id, `needs`,
// step-id, step-uses and workflow-dispatch-input(-name). This corrects an earlier assertion that
// classified them as Unsupported; the fence itself still does not parse expressions.
#[test]
fn expressions_in_fields_without_a_schema_expression_context_are_invalid_at_their_route() {
    for (source, route) in [
        (format!("name: ${{{{ x }}}}\n{JOB}"), "name"),
        (format!("on: ${{{{ x }}}}\n{JOB}"), "on"),
        (format!("on: {{'${{{{ x }}}}': null}}\n{JOB}"), "on"),
        (
            "jobs:\n  ${{ x }}:\n    runs-on: linux\n    steps:\n      - run: a\n".to_owned(),
            "jobs.${{ x }}",
        ),
        (
            "jobs:\n  a:\n    runs-on: linux\n    steps:\n      - run: a\n  b:\n    needs: ${{ x }}\n    runs-on: linux\n    steps:\n      - run: a\n"
                .to_owned(),
            "jobs.b.needs",
        ),
        (
            "jobs:\n  a:\n    runs-on: linux\n    steps:\n      - run: a\n  b:\n    needs: [a, '${{ x }}']\n    runs-on: linux\n    steps:\n      - run: a\n"
                .to_owned(),
            "jobs.b.needs[2]",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: ${{ x }}\n        run: a\n"
                .to_owned(),
            "jobs.b.steps[1].id",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ${{ x }}\n".to_owned(),
            "jobs.b.steps[1].uses",
        ),
        (
            format!("on:\n  workflow_dispatch:\n    inputs:\n      ${{{{ x }}}}: {{}}\n{JOB}"),
            // The key is fenced as text at its enclosing mapping, as before.
            "on.workflow_dispatch.inputs",
        ),
        (
            dispatch_input("description: ${{ x }}"),
            "on.workflow_dispatch.inputs.a.description",
        ),
        (
            dispatch_input("type: ${{ x }}"),
            "on.workflow_dispatch.inputs.a.type",
        ),
        (
            dispatch_input("required: ${{ x }}"),
            "on.workflow_dispatch.inputs.a.required",
        ),
        (
            dispatch_input("default: ${{ x }}"),
            "on.workflow_dispatch.inputs.a.default",
        ),
        (
            dispatch_input("type: choice\n        options: ['${{ x }}']"),
            "on.workflow_dispatch.inputs.a.options[1]",
        ),
    ] {
        assert_eq!(
            diagnose(&source),
            (Category::Invalid, Some(route.to_owned())),
            "{source}"
        );
    }
}

// Definitions with a schema expression context hold valid GHA that WSR does not evaluate:
// string-strategy-context (job `name`), step-name, string-steps-context (`run`) and runs-on.
#[test]
fn expressions_in_fields_with_a_schema_expression_context_are_unsupported_at_their_route() {
    for (source, route) in [
        (
            "jobs:\n  b:\n    name: ${{ x }}\n    runs-on: linux\n    steps:\n      - run: a\n"
                .to_owned(),
            "jobs.b.name",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n        name: ${{ x }}\n"
                .to_owned(),
            "jobs.b.steps[1].name",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: ${{ x }}\n".to_owned(),
            "jobs.b.steps[1].run",
        ),
        (
            "jobs:\n  b:\n    runs-on: ${{ x }}\n    steps:\n      - run: a\n".to_owned(),
            "jobs.b.runs-on",
        ),
        (
            "jobs:\n  b:\n    runs-on: [linux, '${{ x }}']\n    steps:\n      - run: a\n"
                .to_owned(),
            "jobs.b.runs-on[2]",
        ),
    ] {
        assert_eq!(
            diagnose(&source),
            (Category::Unsupported, Some(route.to_owned())),
            "{source}"
        );
    }
}

#[test]
fn keys_and_shapes_outside_the_github_actions_schema_stay_invalid() {
    for (source, route) in [
        (
            "jobs:\n  b:\n    bogus: true\n    runs-on: linux\n    steps:\n      - run: echo\n"
                .to_owned(),
            Some("jobs.b.bogus"),
        ),
        (format!("bogus: 1\n{JOB}"), Some("bogus")),
        (format!("on: {{bogus: null}}\n{JOB}"), Some("on.bogus")),
        (format!("on: bogus\n{JOB}"), Some("on")),
        (format!("on: [push, bogus]\n{JOB}"), Some("on")),
        (format!("on: [1]\n{JOB}"), Some("on")),
        (
            "jobs:\n  b:\n    runs-on: {bogus: linux}\n    steps:\n      - run: echo\n".to_owned(),
            Some("jobs.b.runs-on"),
        ),
        (
            "jobs:\n  b:\n    runs-on: {group: 1}\n    steps:\n      - run: echo\n".to_owned(),
            Some("jobs.b.runs-on"),
        ),
        (
            format!("on: {{workflow_dispatch: {{inputs: {{a: {{type: object}}}}}}}}\n{JOB}"),
            Some("on.workflow_dispatch.inputs.a.type"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n        bogus: x\n"
                .to_owned(),
            Some("jobs.b.steps[1].bogus"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    needs: c\n    steps:\n      - run: a\n"
                .to_owned(),
            Some("jobs.b.needs"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: ~\n        run: a\n"
                .to_owned(),
            Some("jobs.b.steps[1].id"),
        ),
        (format!("name: 5\n{JOB}"), Some("name")),
        ("jobs: {}".to_owned(), Some("jobs")),
        ("jobs: [".to_owned(), None),
    ] {
        assert_eq!(
            diagnose(&source),
            (Category::Invalid, route.map(str::to_owned)),
            "{source}"
        );
    }
}

#[test]
fn typed_errors_keep_parser_locations_and_duplicate_key_evidence() {
    let error = wsr_gha::inspect(
        b"jobs:\n  b:\n    runs-on: linux\n    if: true\n    steps:\n      - run: echo\n",
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "jobs.b: unknown field `if`, expected one of `name`, `runs-on`, `needs`, `permissions`, `timeout-minutes`, `steps` at line 4 column 5"
    );
    let duplicate = wsr_gha::inspect(
        format!("on: {{workflow_dispatch: null, workflow_dispatch: {{}}}}\n{JOB}").as_bytes(),
    )
    .unwrap_err();
    assert_eq!(duplicate.category(), Category::Invalid);
    assert_eq!(
        duplicate.to_string(),
        "on: duplicate entry with key \"workflow_dispatch\" at line 1 column 5"
    );
}

#[test]
fn a_job_identifier_is_validated_before_its_body_so_it_cannot_forge_a_route() {
    let error = wsr_gha::inspect(
        b"jobs:\n  a.steps[9]:\n    bogus: 1\n    runs-on: linux\n    steps:\n      - run: echo\n",
    )
    .unwrap_err();
    assert_eq!(error.category(), Category::Invalid);
    assert_eq!(error.route(), Some("jobs.a.steps[9]"));
    assert_eq!(
        error.to_string(),
        "jobs.a.steps[9]: invalid job identifier 'a.steps[9]'"
    );
}
