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
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n      - shell: bash\n        run: b\n"
                .to_owned(),
            "jobs.b.steps[2].shell",
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

#[test]
fn expressions_in_literal_only_fields_are_unsupported_at_their_route() {
    for (source, route) in [
        (format!("name: ${{{{ x }}}}\n{JOB}"), "name"),
        (format!("on: ${{{{ x }}}}\n{JOB}"), "on"),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n        name: ${{ x }}\n"
                .to_owned(),
            "jobs.b.steps[1].name",
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
        "jobs.b: unknown field `if`, expected one of `name`, `runs-on`, `needs`, `permissions`, `steps` at line 4 column 5"
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
