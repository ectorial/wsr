mod common;
use wsr_gha::Category;

const JOB: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

fn diagnose(source: &str) -> (Category, Option<String>) {
    let error = common::inspect(source.as_bytes()).unwrap_err();
    (error.category(), error.route().map(str::to_owned))
}

#[test]
fn valid_github_actions_source_outside_the_subset_is_unsupported_at_its_route() {
    for (source, route) in [
        // Job `if` (B-12) and then `continue-on-error` (B-17) were this example; see
        // conditions.rs and continue_timeout.rs.
        (
            "jobs:\n  b:\n    cancel-timeout-minutes: 5\n    runs-on: linux\n    steps:\n      - run: echo\n"
                .to_owned(),
            "jobs.b.cancel-timeout-minutes",
        ),
        // B-15 contract change: bare events now inspect; their configuration does not.
        // B-22 inspects `push` filters; another event keeps this sample.
        (format!("on: {{release: {{types: [published]}}}}\n{JOB}"), "on.release"),
        (format!("on: [push, push]\n{JOB}"), "on[2]"),
        (format!("on: {{schedule: [{{cron: '0 0 * * *'}}]}}\n{JOB}"), "on.schedule"),
        // B-16 contract change: `concurrency` is now inspected.
        (format!("description: x\n{JOB}"), "description"),
        (
            // B-17 contract change: step `timeout-minutes` is now inspected.
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n      - name: 5\n        run: b\n"
                .to_owned(),
            "jobs.b.steps[2].name",
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
        // B-13: numbers and booleans are converted to strings by the pinned reader.
        (format!("name: 5\n{JOB}"), "name"),
        (
            "jobs:\n  b:\n    runs-on: {group: 1}\n    steps:\n      - run: echo\n".to_owned(),
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

// Definitions with a schema expression context hold valid GHA that WSR does not evaluate: a whole
// step `env` mapping given as an expression. B-24 contract change: job `name`, step `name` and
// `runs-on` expressions are now reported unevaluated (`name_runner_expressions.rs`); B-28 contract
// change: likewise step `run` (`run_expressions.rs`).
#[test]
fn expressions_in_fields_with_a_schema_expression_context_are_unsupported_at_their_route() {
    assert_eq!(
        diagnose(
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        env: ${{ x }}\n"
        ),
        (
            Category::Unsupported,
            Some("jobs.b.steps[1].env".to_owned())
        )
    );
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
        // B-15 contract change: sequence entries are routed to themselves.
        (format!("on: [push, bogus]\n{JOB}"), Some("on[2]")),
        (format!("on: [1]\n{JOB}"), Some("on[1]")),
        (
            "jobs:\n  b:\n    runs-on: {bogus: linux}\n    steps:\n      - run: echo\n".to_owned(),
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
        (
            "jobs:\n  b:\n    runs-on: {group: ~}\n    steps:\n      - run: echo\n".to_owned(),
            Some("jobs.b.runs-on"),
        ),
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
    let error = common::inspect(
        b"jobs:\n  b:\n    runs-on: linux\n    cancel-timeout-minutes: 5\n    steps:\n      - run: echo\n",
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "jobs.b: unsupported field `cancel-timeout-minutes`; this field is not inspected yet at line 4 column 5"
    );
    let duplicate = common::inspect(
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
    let error = common::inspect(
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

// B-20: the message states the category. A key the pinned schema recognises is unsupported, and
// the text says only that WSR does not inspect it; schema membership recognises the key, not the
// value, so the text never claims the source is valid GitHub Actions.
#[test]
fn unsupported_fields_say_so_and_unknown_fields_keep_the_serde_text() {
    let unsupported =
        |field: &str| format!("unsupported field `{field}`; this field is not inspected yet");
    for (source, category, route, message) in [
        (
            format!("description: x\n{JOB}"),
            Category::Unsupported,
            "description",
            unsupported("description"),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    strategy: {matrix: {a: [1]}}\n    steps:\n      - run: echo\n"
                .to_owned(),
            Category::Unsupported,
            "jobs.b.strategy",
            format!("jobs.b: {} at line 4 column 5", unsupported("strategy")),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - parallel: [{run: echo}]\n"
                .to_owned(),
            Category::Unsupported,
            "jobs.b.steps[1].parallel",
            format!("jobs.b.steps[1]: {} at line 5 column 9", unsupported("parallel")),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        bogus: x\n"
                .to_owned(),
            Category::Invalid,
            "jobs.b.steps[1].bogus",
            "jobs.b.steps[1]: unknown field `bogus`, expected one of `id`, `name`, `if`, `run`, `shell`, `working-directory`, `env`, `with`, `uses`, `continue-on-error`, `timeout-minutes` at line 6 column 9"
                .to_owned(),
        ),
        // The pinned reader rejects background, wait, wait-all and cancel steps unless
        // `allowBackgroundSteps` is set, and the default is false.
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        background: true\n"
                .to_owned(),
            Category::Invalid,
            "jobs.b.steps[1].background",
            "jobs.b.steps[1]: unknown field `background`, expected one of `id`, `name`, `if`, `run`, `shell`, `working-directory`, `env`, `with`, `uses`, `continue-on-error`, `timeout-minutes` at line 6 column 9"
                .to_owned(),
        ),
    ] {
        let error = common::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route(), error.to_string()),
            (category, Some(route), message),
            "{source}"
        );
    }
}

#[test]
fn background_step_keys_are_invalid() {
    for key in ["background", "wait", "wait-all", "cancel"] {
        let source = format!(
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {key}: x\n"
        );
        assert_eq!(
            diagnose(&source),
            (Category::Invalid, Some(format!("jobs.b.steps[1].{key}"))),
            "{key}"
        );
    }
}

#[test]
fn dispatch_input_description_still_inspects() {
    common::inspect(
        format!("on:\n  workflow_dispatch:\n    inputs:\n      a:\n        description: d\n{JOB}")
            .as_bytes(),
    )
    .unwrap();
}
