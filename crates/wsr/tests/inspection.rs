use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn inspect(directory: &Path, file: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", file])
        .current_dir(directory)
        .output()
        .expect("run inspection")
}

#[test]
fn inspect_reports_static_jobs_and_steps_without_running_them() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "name: Local CI\njobs:\n  test:\n    name: Tests\n    runs-on: ubuntu-latest\n    steps:\n      - name: Dangerous command\n        run: touch executed\n      - uses: ./action\n";
    fs::write(directory.path().join("ci.yml"), workflow).unwrap();
    fs::write(directory.path().join("wsr.toml"), "invalid config sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Local CI\nInspection only; execution compatibility is not assessed.\nJob: test (Tests)\n  Runner: ubuntu-latest\n  Step 1: run (Dangerous command)\n  Step 2: uses ./action\n"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        workflow
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "invalid config sentinel"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn inspect_rejects_unevaluated_expressions_in_static_fields() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        "jobs:\n  test:\n    runs-on: ${{ matrix.os }}\n    steps:\n      - run: echo hello\n",
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("ci.yml") && error.contains("jobs.test.runs-on"),
        "{error}"
    );
    assert!(error.contains("expression is unevaluated"), "{error}");
}

#[test]
fn inspect_diagnoses_invalid_or_unsupported_source_without_partial_output() {
    let directory = tempfile::tempdir().unwrap();
    for (source, diagnostic) in [
        (
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - run: null\n        uses: ./action",
            "expected a string",
        ),
        ("jobs: {}", "jobs: expected at least one job"),
        (
            "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps: []",
            "jobs.test.steps: expected at least one step",
        ),
        (
            "jobs:\n  test:\n    runs-on: 42\n    steps:\n      - run: echo hello",
            "expected a string",
        ),
        (
            "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n        uses: ./action",
            "expected exactly one of run or uses",
        ),
        (
            "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n    needs: build",
            "jobs.test.needs: unknown job 'build'",
        ),
        ("jobs: [", "line 1"),
        (
            "jobs:\n  test:\n    runs-on: ''\n    steps:\n      - run: echo hello",
            "jobs.test.runs-on: expected a nonempty string",
        ),
        (
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - run: ''",
            "jobs.test.steps[1].run: expected a nonempty string",
        ),
        (
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - uses: ''",
            "jobs.test.steps[1].uses: expected a nonempty string",
        ),
        (
            "jobs:\n  'bad id':\n    runs-on: linux\n    steps:\n      - run: echo hello",
            "invalid job identifier",
        ),
        (
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - run: echo one\n  test:\n    runs-on: linux\n    steps:\n      - run: echo two",
            "duplicate job identifier",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("wsr inspect: ci.yml:") && error.contains(diagnostic),
            "{source}: {error}"
        );
    }
    let missing = inspect(directory.path(), "missing.yml");
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("wsr inspect: missing.yml:")
    );
}

#[test]
fn inspect_manual_workflow_is_deterministic_and_preserves_step_order() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("ci.yml"), "on: workflow_dispatch\njobs:\n  zebra:\n    runs-on: linux\n    steps:\n      - uses: owner/action@v1\n      - run: echo last\n  alpha:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo first\n").unwrap();
    let first = inspect(directory.path(), "ci.yml");
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let second = inspect(directory.path(), "ci.yml");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        String::from_utf8(first.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: alpha\n  Runner: ubuntu-latest\n  Step 1: run\nJob: zebra\n  Runner: linux\n  Step 1: uses owner/action@v1\n  Step 2: run\n"
    );
}

#[test]
fn inspect_static_dependencies() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        include_str!("../../../tests/fixtures/inspection/static-needs.yml"),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Dependency example\nInspection only; execution compatibility is not assessed.\nJob: build\n  Runner: ubuntu-latest\n  Step 1: run\nJob: package\n  Runner: ubuntu-latest\n  Needs: build, test\n  Step 1: run\nJob: test\n  Runner: ubuntu-latest\n  Needs: build\n  Step 1: run\n"
    );
}

fn dependency_workflow(jobs: &[(&str, &str)]) -> String {
    let mut source = String::from("jobs:\n");
    for (id, needs) in jobs {
        source.push_str(&format!(
            "  {id}:\n    runs-on: linux\n{needs}    steps:\n      - run: touch executed\n"
        ));
    }
    source
}

#[test]
fn inspect_unknown_dependencies_fail_before_rendering() {
    let directory = tempfile::tempdir().unwrap();
    for (needs, route) in [
        ("    needs: missing\n", "jobs.test.needs"),
        ("    needs: [build, missing]\n", "jobs.test.needs[2]"),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            dependency_workflow(&[("build", ""), ("test", needs)]),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:")
                && error.contains(route)
                && error.contains("unknown job 'missing'"),
            "{error}"
        );
    }
}

#[test]
fn inspect_rejects_self_dependencies() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        dependency_workflow(&[("build", "    needs: build\n")]),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("jobs.build.needs: job 'build' depends on itself")
    );
}

#[test]
fn inspect_cycle_diagnostic_excludes_blocked_downstream_jobs() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        dependency_workflow(&[
            ("downstream", "    needs: one\n"),
            ("one", "    needs: two\n"),
            ("two", "    needs: [three]\n"),
            ("three", "    needs: one\n"),
        ]),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("jobs.three.needs: dependency cycle: one -> two -> three -> one"),
        "{error}"
    );
    assert!(!error.contains("downstream"), "{error}");
}

#[test]
fn inspect_dependencies_allow_diamonds_repeated_edges_forward_references_and_empty_lists() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        dependency_workflow(&[
            ("package", "    needs: [test, build, test]\n"),
            ("test", "    needs: [build]\n"),
            ("build", "    needs: last\n"),
            ("last", "    needs: []\n"),
            ("disconnected", ""),
        ]),
    )
    .unwrap();
    let first = inspect(directory.path(), "ci.yml");
    let second = inspect(directory.path(), "ci.yml");
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    let summary = String::from_utf8(first.stdout).unwrap();
    assert!(summary.contains("Job: package\n  Runner: linux\n  Needs: test, build, test\n"));
    assert!(summary.contains("Job: last\n  Runner: linux\n  Step 1: run\n"));
    assert!(summary.contains("Job: disconnected\n  Runner: linux\n  Step 1: run\n"));
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_unknown_dependencies_take_precedence_over_cycles() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"),
        dependency_workflow(&[
            ("cycle", "    needs: cycle\n"),
            ("test", "    needs: [missing]\n"),
        ]),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("jobs.test.needs[1]: unknown job 'missing'"),
        "{error}"
    );
    assert!(
        !error.contains("depends on itself") && !error.contains("dependency cycle:"),
        "{error}"
    );
}

#[test]
fn inspect_rejects_malformed_and_dynamic_dependencies() {
    let directory = tempfile::tempdir().unwrap();
    for (value, diagnostic) in [
        ("null", "expected a string or a sequence of strings"),
        ("42", "expected a string or a sequence of strings"),
        ("false", "expected a string or a sequence of strings"),
        ("{job: build}", "expected a string or a sequence of strings"),
        ("[build, 42]", "needs[2]: expected a string"),
        ("[null]", "needs[1]: expected a string"),
        ("''", "jobs.test.needs: expected a nonempty string"),
        (
            "[build, '  ']",
            "jobs.test.needs[2]: expected a nonempty string",
        ),
        (
            "'${{ inputs.job }}'",
            "jobs.test.needs: expression is unevaluated",
        ),
        (
            "[build, '${{ inputs.job }}']",
            "jobs.test.needs[2]: expression is unevaluated",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            dependency_workflow(&[("build", ""), ("test", &format!("    needs: {value}\n"))]),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{value}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            // YAML decoding errors locate the enclosing job and name the needs field in
            // the custom decoder message; semantic diagnostics use the full field route.
            error.contains("ci.yml:")
                && error.contains("jobs.test")
                && error.contains("needs")
                && error.contains(diagnostic),
            "{value}: {error}"
        );
    }
}

#[test]
fn inspect_dependency_validation_does_not_execute_or_modify_repository_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let sentinels = [
        ("wsr.toml", "invalid configuration sentinel"),
        (".git/hooks/pre-push", "#!/bin/sh\ntouch hook-executed\n"),
        (".wsr/cache/entry", "cache sentinel"),
    ];
    for (file, contents) in sentinels {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    for (needs, status) in [("    needs: build\n", 0), ("    needs: test\n", 1)] {
        let source = dependency_workflow(&[("build", ""), ("test", needs)]).replace(
            "      - run: touch executed",
            "      - run: touch executed\n      - uses: ./unopened-action",
        );
        fs::write(root.join("ci.yml"), &source).unwrap();
        let output = inspect(root, "ci.yml");
        assert_eq!(output.status.code(), Some(status), "{output:?}");
        assert_eq!(fs::read_to_string(root.join("ci.yml")).unwrap(), source);
        for (file, contents) in sentinels {
            assert_eq!(fs::read_to_string(root.join(file)).unwrap(), contents);
        }
        assert_eq!(fs::read_dir(root).unwrap().count(), 4);
        assert_eq!(fs::read_dir(root.join(".git/hooks")).unwrap().count(), 1);
        assert_eq!(fs::read_dir(root.join(".wsr/cache")).unwrap().count(), 1);
        assert!(!root.join("executed").exists());
        assert!(!root.join("hook-executed").exists());
    }
}

#[test]
fn inspect_deep_dependency_chain_uses_bounded_graph_traversal() {
    let directory = tempfile::tempdir().unwrap();
    let count = 4096;
    let mut source = String::from("jobs:\n");
    for index in 0..count {
        let needs = if index + 1 < count {
            format!("    needs: job_{}\n", index + 1)
        } else {
            String::new()
        };
        source.push_str(&format!(
            "  job_{index}:\n    runs-on: linux\n{needs}    steps:\n      - run: echo hello\n"
        ));
    }
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    let summary = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        summary
            .lines()
            .filter(|line| line.starts_with("Job: "))
            .count(),
        count
    );
    assert_eq!(
        summary
            .lines()
            .filter(|line| line.starts_with("  Needs: "))
            .count(),
        count - 1
    );
}

#[test]
fn inspect_explicit_step_identities_preserve_names_kinds_and_order() {
    let directory = tempfile::tempdir().unwrap();
    let source = include_str!("../../../tests/fixtures/inspection/step-identities.yml");
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Step identity example\nInspection only; execution compatibility is not assessed.\nJob: build\n  Runner: ubuntu-latest\n  Step 1 [compile]: run (Compile sources)\n  Step 2 [upload]: uses ./unopened-action\n  Step 3: run\n"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_rejects_invalid_step_identities_before_rendering() {
    let directory = tempfile::tempdir().unwrap();
    for (value, diagnostic) in [
        ("''".to_owned(), "expected a nonempty string"),
        ("'bad id'".to_owned(), "invalid step identifier"),
        ("'1build'".to_owned(), "invalid step identifier"),
        ("'__internal'".to_owned(), "invalid step identifier"),
        ("'compile.out'".to_owned(), "invalid step identifier"),
        ("éclair".to_owned(), "invalid step identifier"),
        ("x".repeat(100), "invalid step identifier"),
        ("'${{ inputs.id }}'".to_owned(), "expression is unevaluated"),
        ("null".to_owned(), "expected a string"),
        ("42".to_owned(), "expected a string"),
        ("true".to_owned(), "expected a string"),
        ("[]".to_owned(), "expected a string"),
        ("{}".to_owned(), "expected a string"),
    ] {
        let source = dependency_workflow(&[("build", "")]).replace(
            "      - run:",
            &format!("      - id: {value}\n        run:"),
        );
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{value}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:")
                && error.contains("jobs.build.steps")
                && error.contains(diagnostic),
            "{value}: {error}"
        );
    }
}

#[test]
fn inspect_rejects_duplicate_explicit_step_ids_with_both_locations() {
    let directory = tempfile::tempdir().unwrap();
    let source = include_str!("../../../tests/fixtures/inspection/step-identities.yml")
        .replace("id: upload", "id: compile");
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    for expected in [
        "ci.yml:",
        "jobs.build.steps[2].id",
        "duplicate step identifier 'compile'",
        "first declared at jobs.build.steps[1].id",
    ] {
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn inspect_step_ids_preserve_case_allow_cross_job_reuse_and_accept_length_boundary() {
    let directory = tempfile::tempdir().unwrap();
    let longest = "x".repeat(99);
    let source = format!(
        "jobs:\n  first:\n    runs-on: linux\n    steps:\n      - id: Build\n        run: echo first\n      - id: build\n        run: echo second\n      - id: {longest}\n        run: echo longest\n      - id: _valid-1\n        run: echo valid\n  second:\n    needs: first\n    runs-on: linux\n    steps:\n      - id: Build\n        run: echo reused\n"
    );
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    let summary = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "  Step 1 [Build]: run",
        "  Step 2 [build]: run",
        &format!("  Step 3 [{longest}]: run"),
        "  Step 4 [_valid-1]: run",
        "Job: second\n  Runner: linux\n  Needs: first\n  Step 1 [Build]: run",
    ] {
        assert!(summary.contains(expected), "{summary}");
    }
}

#[test]
fn inspect_runner_label_lists_preserve_steps_dependencies_and_sentinels() {
    let directory = tempfile::tempdir().unwrap();
    let source = include_str!("../../../tests/fixtures/inspection/runner-labels.yml");
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    fs::write(directory.path().join("wsr.toml"), "sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nJob: build\n  Runner labels: [\"self-hosted\", \"linux\", \"x64\"]\n  Step 1 [compile]: run\n  Step 2: uses ./unopened-action\nJob: test\n  Runner: ubuntu-latest\n  Needs: build\n  Step 1: run\n"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "sentinel"
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_runner_labels_reject_empty_invalid_and_dynamic_values() {
    let directory = tempfile::tempdir().unwrap();
    for (value, diagnostic) in [
        ("[]", "runs-on: expected at least one label"),
        ("[linux, ' ']", "runs-on[2]: expected a nonempty string"),
        (
            "[linux, '${{ matrix.os }}']",
            "runs-on[2]: expression is unevaluated",
        ),
        ("'${{ matrix.os }}'", "runs-on: expression is unevaluated"),
        ("null", "runs-on: expected a string or list of strings"),
        ("42", "runs-on: expected a string or list of strings"),
        ("true", "runs-on: expected a string or list of strings"),
        ("[linux, 42]", "runs-on[2]: expected a string label"),
        ("[linux, true]", "runs-on[2]: expected a string label"),
        ("[linux, null]", "runs-on[2]: expected a string label"),
        ("[linux, [x64]]", "runs-on[2]: expected a string label"),
        ("[linux, {}]", "runs-on[2]: expected a string label"),
        ("{group: fleet}", "unsupported runner-selector mapping"),
    ] {
        let source = format!(
            "jobs:\n  build:\n    runs-on: {value}\n    steps:\n      - run: touch executed\n"
        );
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{value}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:") && error.contains("jobs.build") && error.contains(diagnostic),
            "{value}: {error}"
        );
        assert_eq!(
            fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
            source
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}

#[test]
fn inspect_runner_labels_quote_boundaries_preserve_case_order_and_repetitions() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("ci.yml"), "jobs:\n  build:\n    runs-on: ['Linux', 'a,b', 'Linux', \"line\\nbreak\", 'quote\"label']\n    steps:\n      - run: echo hello\n  single:\n    runs-on: [linux]\n    steps:\n      - run: echo hello\n").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    let summary = String::from_utf8(output.stdout).unwrap();
    assert!(
        summary.contains(
            "Runner labels: [\"Linux\", \"a,b\", \"Linux\", \"line\\nbreak\", \"quote\\\"label\"]"
        ),
        "{summary}"
    );
    assert!(summary.contains("Runner labels: [\"linux\"]"), "{summary}");
}

#[test]
fn inspect_manual_input_declarations_without_applying_or_executing_them() {
    let directory = tempfile::tempdir().unwrap();
    let source = include_str!("../../../tests/fixtures/inspection/dispatch-inputs.yml");
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    fs::write(directory.path().join("wsr.toml"), "sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Manual input example\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n  Input: mode\n    Type: choice\n    Description: Choose the reference outcome\n    Required: true\n    Default: \"pass\" (not applied)\n    Options: [\"pass\", \"fail\"]\nJob: build\n  Runner: ubuntu-latest\n  Step 1 [compile]: run\n  Step 2: uses ./unopened-action\n"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "sentinel"
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    assert!(!directory.path().join("executed").exists());
}

fn dispatch_workflow(declaration: &str) -> String {
    format!(
        "on:\n  workflow_dispatch:\n    inputs:\n      value:\n{declaration}\njobs:\n  build:\n    runs-on: linux\n    steps:\n      - run: touch executed\n"
    )
}

#[test]
fn inspect_dispatch_enforces_choice_options_and_default_types() {
    let directory = tempfile::tempdir().unwrap();
    for (fields, diagnostic) in [
        (
            "        type: choice",
            "options: expected at least one option for choice input",
        ),
        (
            "        type: choice\n        options: []",
            "options: expected at least one option for choice input",
        ),
        (
            "        options: [a]",
            "options: only supported for choice input",
        ),
        (
            "        type: boolean\n        default: 'false'",
            "default: expected a boolean for boolean input",
        ),
        (
            "        type: string\n        default: true",
            "default: expected a string for string, choice or environment input",
        ),
        (
            "        type: environment\n        default: 42",
            "default: expected a string for string, choice or environment input",
        ),
        (
            "        default: 42",
            "default: expected a string for string, choice or environment input",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), dispatch_workflow(fields)).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{fields}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:")
                && error.contains("on.workflow_dispatch.inputs.value")
                && error.contains(diagnostic),
            "{error}"
        );
    }
}

#[test]
fn inspect_dispatch_rejects_malformed_unsupported_and_dynamic_declarations() {
    let directory = tempfile::tempdir().unwrap();
    for (fields, diagnostic) in [
        ("        description: 42", ".description: expected a string"),
        (
            "        description: null",
            ".description: expected a string",
        ),
        ("        required: 'true'", ".required: expected a boolean"),
        ("        required: null", ".required: expected a boolean"),
        ("        type: object", ".type: unsupported input type"),
        ("        type: []", ".type: expected a string"),
        (
            "        default: null",
            ".default: expected a string, boolean or number",
        ),
        (
            "        default: []",
            ".default: expected a string, boolean or number",
        ),
        (
            "        default: {}",
            ".default: expected a string, boolean or number",
        ),
        (
            "        type: choice\n        options: a",
            ".options: expected a string list",
        ),
        (
            "        type: choice\n        options: [a, false]",
            ".options[2]: expected a string",
        ),
        (
            "        type: choice\n        options: [a, null]",
            ".options[2]: expected a string",
        ),
        (
            "        description: '${{ inputs.mode }}'",
            ".description: expression is unevaluated",
        ),
        (
            "        default: '${{ secrets.VALUE }}'",
            ".default: expression is unevaluated",
        ),
        (
            "        type: '${{ inputs.type }}'",
            ".type: expression is unevaluated",
        ),
        (
            "        type: choice\n        options: [a, '${{ inputs.mode }}']",
            ".options[2]: expression is unevaluated",
        ),
        (
            "        unexpected: true",
            ".unexpected: unsupported field or event",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), dispatch_workflow(fields)).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{fields}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:")
                && error.contains("on.workflow_dispatch.inputs.value")
                && error.contains(diagnostic),
            "{error}"
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
    for (trigger, diagnostic) in [
        ("null", "on: expected a mapping"),
        ("[]", "on: expected a mapping"),
        ("{}", "on: expected workflow_dispatch entry"),
        ("{push: null}", "on.push: unsupported field or event"),
        (
            "{workflow_dispatch: {unexpected: true}}",
            "on.workflow_dispatch.unexpected",
        ),
        (
            "{workflow_dispatch: []}",
            "on.workflow_dispatch: expected a mapping",
        ),
        (
            "{workflow_dispatch: {inputs: null}}",
            "on.workflow_dispatch.inputs: expected a mapping",
        ),
        (
            "{workflow_dispatch: {inputs: {value: null}}}",
            "on.workflow_dispatch.inputs.value: expected a mapping",
        ),
        (
            "{workflow_dispatch: {inputs: {42: {}}}}",
            "on.workflow_dispatch.inputs: expected a string",
        ),
        (
            "{workflow_dispatch: {inputs: {'': {}}}}",
            "on.workflow_dispatch.inputs.: expected a nonempty string",
        ),
        (
            "{workflow_dispatch: {inputs: {value: {}, value: {}}}}",
            "duplicate",
        ),
        (
            "{workflow_dispatch: {inputs: {value: {type: string, type: boolean}}}}",
            "duplicate",
        ),
        (
            "{workflow_dispatch: null, workflow_dispatch: {}}",
            "duplicate",
        ),
    ] {
        let source = format!("on: {trigger}\n{}", dependency_workflow(&[("build", "")]));
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{trigger}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:") && error.contains(diagnostic),
            "{trigger}: {error}"
        );
    }
}

#[test]
fn inspect_dispatch_preserves_empty_declarations_types_and_deterministic_order() {
    let directory = tempfile::tempdir().unwrap();
    for trigger in [
        "workflow_dispatch",
        "{workflow_dispatch: null}",
        "{workflow_dispatch: {}}",
        "{workflow_dispatch: {inputs: {}}}",
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            format!("on: {trigger}\n{}", dependency_workflow(&[("build", "")])),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{trigger}: {output:?}");
        assert!(output.stderr.is_empty());
        let summary = String::from_utf8(output.stdout).unwrap();
        assert!(
            summary.contains("Trigger: workflow_dispatch (not evaluated)\nJob: build"),
            "{summary}"
        );
    }
    let source = "on:\n  workflow_dispatch:\n    inputs:\n      z_string:\n        type: string\n        default: '42'\n        required: false\n      d_environment:\n        type: environment\n        default: production\n      c_number:\n        type: number\n        default: 42\n      b_boolean:\n        type: boolean\n        default: false\n      a_omitted:\n        description: \"line\\nbreak\"\n      e_choice:\n        type: choice\n        options: ['a,b', a, a]\n        default: other\n";
    fs::write(
        directory.path().join("ci.yml"),
        format!("{source}{}", dependency_workflow(&[("build", "")])),
    )
    .unwrap();
    let first = inspect(directory.path(), "ci.yml");
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(first.stderr.is_empty());
    let second = inspect(directory.path(), "ci.yml");
    assert_eq!(first.stdout, second.stdout);
    let summary = String::from_utf8(first.stdout).unwrap();
    for expected in [
        "Input: a_omitted\n    Description: line\\nbreak\n  Input: b_boolean",
        "Type: boolean\n    Default: false (not applied)",
        "Type: number\n    Default: 42 (not applied)",
        "Type: environment\n    Default: \"production\" (not applied)",
        "Options: [\"a,b\", \"a\", \"a\"]",
        "Type: string\n    Required: false\n    Default: \"42\" (not applied)",
    ] {
        assert!(summary.contains(expected), "{summary}");
    }
}

#[test]
fn inspect_dispatch_identifiers_and_dynamic_required_are_diagnosed() {
    let directory = tempfile::tempdir().unwrap();
    for (key, fields, diagnostic) in [
        (
            "bad id",
            "{}",
            "on.workflow_dispatch.inputs.bad id: invalid input identifier 'bad id'",
        ),
        (
            "1input",
            "{}",
            "on.workflow_dispatch.inputs.1input: invalid input identifier '1input'",
        ),
        (
            "valid",
            "{required: '${{ inputs.flag }}'}",
            "required: expression is unevaluated",
        ),
    ] {
        let source = format!(
            "on: {{workflow_dispatch: {{inputs: {{'{key}': {fields}}}}}}}\n{}",
            dependency_workflow(&[("build", "")])
        );
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:")
                && error.contains("on.workflow_dispatch.inputs")
                && error.contains(diagnostic),
            "{error}"
        );
    }
}

const RUN_NAME_HEADER: &str = "Workflow: Example shell reference\nInspection only; execution compatibility is not assessed.\n";

fn run_name_workflow(run_name: &str) -> String {
    format!(
        "name: Example shell reference\n{run_name}on:\n  workflow_dispatch:\n    inputs:\n      mode:\n        type: string\n        default: quick\njobs:\n  shell:\n    runs-on: ubuntu-latest\n    steps:\n      - run: touch executed\n      - uses: ./action\n"
    )
}

#[test]
fn inspect_run_name_reports_unevaluated_source_without_executing_or_binding_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = run_name_workflow("run-name: \"Shell reference — ${{ inputs.mode }}\"\n");
    fs::write(directory.path().join("ci.yml"), &workflow).unwrap();
    fs::write(directory.path().join("wsr.toml"), "invalid config sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "Workflow: Example shell reference\nRun name: Shell reference — ${{ inputs.mode }} (unevaluated; expression syntax not validated)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n  Input: mode\n    Type: string\n    Default: \"quick\" (not applied)\nJob: shell\n  Runner: ubuntu-latest\n  Step 1: run\n  Step 2: uses ./action\n"
    );
    assert!(!stdout.contains("Shell reference — quick"), "{stdout}");
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        workflow
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "invalid config sentinel"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[test]
fn inspect_run_name_literal_omitted_and_unvalidated_fences() {
    let directory = tempfile::tempdir().unwrap();
    for (declaration, line) in [
        (
            "run-name: Local reference\n",
            Some("Run name: Local reference (literal)"),
        ),
        ("", None),
        (
            "run-name: \"${{ inputs.a }}-${{ inputs.b }}\"\n",
            Some(
                "Run name: ${{ inputs.a }}-${{ inputs.b }} (unevaluated; expression syntax not validated)",
            ),
        ),
        (
            "run-name: \"open ${{ inputs.mode\"\n",
            Some("Run name: open ${{ inputs.mode (unevaluated; expression syntax not validated)"),
        ),
        (
            "run-name: \"a ${{\\n inputs.mode }}\\u001b\"\n",
            Some(
                "Run name: a ${{\\n inputs.mode }}\\u{1b} (unevaluated; expression syntax not validated)",
            ),
        ),
        (
            "run-name: \"tab\\there\"\n",
            Some("Run name: tab\\there (literal)"),
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            run_name_workflow(declaration),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{declaration}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let expected = match line {
            Some(line) => RUN_NAME_HEADER.replacen("\n", &format!("\n{line}\n"), 1),
            None => RUN_NAME_HEADER.to_string(),
        };
        assert!(stdout.starts_with(&expected), "{declaration}: {stdout}");
        assert_eq!(stdout.contains("Run name:"), line.is_some(), "{stdout}");
    }
}

#[test]
fn inspect_run_name_rejects_wrong_shapes_and_blank_text_before_rendering() {
    let directory = tempfile::tempdir().unwrap();
    for (declaration, diagnostic) in [
        ("run-name: null\n", "expected a string"),
        ("run-name: ~\n", "expected a string"),
        ("run-name: true\n", "expected a string"),
        ("run-name: 42\n", "expected a string"),
        ("run-name: [a]\n", "expected a string"),
        ("run-name: {a: b}\n", "expected a string"),
        ("run-name: ''\n", "run-name: expected a nonempty string"),
        ("run-name: '  '\n", "run-name: expected a nonempty string"),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            run_name_workflow(declaration),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{declaration}: {output:?}");
        assert!(output.stdout.is_empty(), "{declaration}: {output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("wsr inspect: ci.yml:") && error.contains(diagnostic),
            "{declaration}: {error}"
        );
    }
}

#[test]
fn inspect_diagnostics_use_one_based_field_routes_and_validate_job_ids_first() {
    let directory = tempfile::tempdir().unwrap();
    let job = "    runs-on: linux\n    steps:\n      - run: echo\n";
    for (source, diagnostic) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        bogus: {}\n"
                .to_owned(),
            "jobs.b.steps[1]: unknown field `bogus`, expected one of `id`, `name`, `run`, `shell`, `working-directory`, `env`, `with`, `uses` at line 6 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: ~\n        run: echo\n"
                .to_owned(),
            "jobs.b.steps[1].id: expected a string at line 5 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n      - run: echo\n        name: 5\n"
                .to_owned(),
            "jobs.b.steps[2].name: expected a string at line 6 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: [linux, 42]\n    steps:\n      - run: echo\n".to_owned(),
            "jobs.b.runs-on[2]: expected a string label at line 3 column 5",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    needs: [b, 1]\n    steps:\n      - run: echo\n"
                .to_owned(),
            "jobs.b.needs[2]: expected a string at line 3 column 5",
        ),
        (
            format!("jobs:\n  'b c':\n    needs: missing\n{job}"),
            "jobs.b c: invalid job identifier 'b c'",
        ),
        // The identifier is validated before its body, so it cannot forge `jobs.a.steps[9].bogus`.
        (
            format!("jobs:\n  a.steps[9]:\n    bogus: 1\n{job}"),
            "jobs.a.steps[9]: invalid job identifier 'a.steps[9]'",
        ),
        (
            format!("jobs:\n  '${{{{ x }}}}':\n{job}"),
            "jobs.${{ x }}: expression is unevaluated; dynamic inspection is not supported yet",
        ),
        (
            format!("jobs:\n  \"b\\x01\":\n{job}"),
            "jobs.b\\u{1}: invalid job identifier 'b\\u{1}'",
        ),
        (
            format!("jobs:\n  b:\n{job}  b:\n{job}"),
            "jobs.b: duplicate job identifier 'b' at line 2 column 3",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
    }
}

#[test]
fn inspect_errors_never_write_raw_control_bytes_to_stderr() {
    let directory = tempfile::tempdir().unwrap();
    let steps = "    runs-on: linux\n    steps:\n      - run: echo hello\n";
    for (location, source) in [
        (
            "top-level key",
            format!("\"\\e[31mbad\": 1\njobs:\n  test:\n{steps}"),
        ),
        (
            "job key",
            format!("jobs:\n  test:\n    \"\\e[31mbad\": 1\n{steps}"),
        ),
        (
            "step key",
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - run: echo hello\n        \"\\e[31mbad\": 1\n".to_string(),
        ),
        (
            "on key",
            format!("on:\n  \"\\e[31mbad\": null\njobs:\n  test:\n{steps}"),
        ),
        (
            "job identifier",
            format!("jobs:\n  \"\\e[31mbad\":\n{steps}"),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{location}: {output:?}");
        assert!(output.stdout.is_empty(), "{location}: {output:?}");
        assert!(
            !output.stderr.contains(&0x1b),
            "{location}: raw ESC on stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("\\u{1b}"), "{location}: {error}");
    }
}

// Backlog B-9: bidi overrides, isolates and zero-width characters (Unicode `Cf`) can make a
// diagnostic read differently from its bytes, so stderr escapes them exactly as stdout does.
#[test]
fn inspect_errors_escape_unicode_format_characters_on_stderr() {
    let directory = tempfile::tempdir().unwrap();
    let job = "    runs-on: linux\n    steps:\n      - run: echo\n";
    for (source, diagnostic) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        \"k\\u202Eq\\u200B\": 1\n".to_owned(),
            "jobs.b.steps[1]: unknown field `k\\u{202e}q\\u{200b}`, expected one of `id`, `name`, `run`, `shell`, `working-directory`, `env`, `with`, `uses` at line 6 column 9",
        ),
        (
            format!("jobs:\n  \"b\\u202E\\u200B\\u2066\\u2069\":\n{job}"),
            "jobs.b\\u{202e}\\u{200b}\\u{2066}\\u{2069}: invalid job identifier 'b\\u{202e}\\u{200b}\\u{2066}\\u{2069}'",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            !error.contains(['\u{202e}', '\u{200b}', '\u{2066}', '\u{2069}']),
            "raw format character on stderr: {error:?}"
        );
        assert_eq!(error, format!("error: wsr inspect: ci.yml: {diagnostic}\n"), "{source}");
    }
}

#[test]
fn inspect_does_not_panic_when_stdout_is_closed_early() {
    use std::process::Stdio;

    let directory = tempfile::tempdir().unwrap();
    let mut source = String::from("jobs:\n");
    for index in 0..5000 {
        source.push_str(&format!(
            "  job{index:05}:\n    name: A reasonably long display name for job {index}\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n"
        ));
    }
    let size = source.len();
    fs::write(directory.path().join("big.yml"), source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", "big.yml"])
        .current_dir(directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn inspection");
    // The report exceeds any pipe buffer, so closing the read end before it is
    // consumed guarantees the child's write fails with a broken pipe.
    drop(child.stdout.take());
    let output = child.wait_with_output().expect("wait for inspection");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    // Only the D-032 size warning, which is written before stdout.
    assert_eq!(
        stderr,
        format!(
            "warning: wsr inspect: big.yml: {size} bytes exceeds GitHub's documented 500 KB (512000-byte) workflow file limit; GitHub would not run this workflow\n"
        )
    );
}

#[cfg(unix)]
fn inspect_non_regular(directory: &Path, file: &str) -> Output {
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let mut child = Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", file])
        .current_dir(directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn inspection");
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().expect("poll inspection").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("kill hung inspection");
            let output = child.wait_with_output().expect("reap inspection");
            panic!("inspect {file} did not terminate within 2s: {output:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().expect("collect inspection")
}

#[cfg(unix)]
#[test]
fn inspect_rejects_non_regular_files_without_reading_them() {
    let directory = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink("/dev/zero", directory.path().join("zero.yml")).unwrap();
    let fifo = directory.path().join("pipe.yml");
    let made_fifo = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .is_ok_and(|status| status.success());
    let mut files = vec!["zero.yml", "."];
    if made_fifo {
        files.push("pipe.yml");
    }
    for file in files {
        let output = inspect_non_regular(directory.path(), file);
        assert_eq!(output.status.code(), Some(1), "{file}: {output:?}");
        assert!(output.stdout.is_empty(), "{file}: {output:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(&format!("wsr inspect: {file}: not a regular file")),
            "{file}: {error}"
        );
    }
}

#[test]
fn inspect_reports_requested_permissions_without_granting_them() {
    let directory = tempfile::tempdir().unwrap();
    let job = "    runs-on: linux\n    steps:\n      - run: touch executed\n";
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\n";
    for (source, report) in [
        (
            format!("permissions: read-all\njobs:\n  b:\n{job}"),
            format!(
                "{head}Permissions: read-all (requested; not granted)\nJob: b\n  Runner: linux\n  Step 1: run\n"
            ),
        ),
        (
            format!("permissions: {{}}\njobs:\n  b:\n{job}"),
            format!(
                "{head}Permissions: {{}} (requested; not granted)\nJob: b\n  Runner: linux\n  Step 1: run\n"
            ),
        ),
        (
            format!(
                "permissions:\n  pull-requests: none\n  contents: read\n  id-token: write\njobs:\n  a:\n{job}  b:\n    permissions: write-all\n    needs: a\n{job}"
            ),
            format!(
                "{head}Permissions (requested; not granted):\n  contents: read\n  id-token: write\n  pull-requests: none\nJob: a\n  Runner: linux\n  Step 1: run\nJob: b\n  Runner: linux\n  Needs: a\n  Permissions: write-all (requested; not granted)\n  Step 1: run\n"
            ),
        ),
        (
            format!(
                "jobs:\n  b:\n    permissions:\n      models: read\n      actions: write\n{job}"
            ),
            format!(
                "{head}Job: b\n  Runner: linux\n  Permissions (requested; not granted):\n    actions: write\n    models: read\n  Step 1: run\n"
            ),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{source}: {output:?}");
        assert!(output.stderr.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            report,
            "{source}"
        );
        assert!(!directory.path().join("executed").exists());
    }
}

#[test]
fn inspect_rejects_permissions_outside_the_pinned_schema() {
    let directory = tempfile::tempdir().unwrap();
    let job = "    runs-on: linux\n    steps:\n      - run: echo\n";
    let levels = "expected read-all, write-all or a mapping of permission scopes";
    for (source, diagnostic) in [
        (
            format!("permissions: read\njobs:\n  b:\n{job}"),
            format!("permissions: {levels}"),
        ),
        (
            format!("permissions: ~\njobs:\n  b:\n{job}"),
            format!("permissions: {levels}"),
        ),
        (
            format!("permissions: [contents]\njobs:\n  b:\n{job}"),
            format!("permissions: {levels}"),
        ),
        (
            format!("permissions:\n  secrets: read\njobs:\n  b:\n{job}"),
            "permissions.secrets: unknown permission scope 'secrets'".to_owned(),
        ),
        (
            format!("permissions:\n  1: read\njobs:\n  b:\n{job}"),
            "permissions: expected a string permission scope".to_owned(),
        ),
        (
            format!("jobs:\n  b:\n    permissions:\n      contents: admin\n{job}"),
            "jobs.b.permissions.contents: expected read, write or none at line 3 column 5"
                .to_owned(),
        ),
        (
            format!("jobs:\n  b:\n    permissions:\n      models: write\n{job}"),
            "jobs.b.permissions.models: expected read or none at line 3 column 5".to_owned(),
        ),
        (
            format!("jobs:\n  b:\n    permissions:\n      id-token: read\n{job}"),
            "jobs.b.permissions.id-token: expected write or none at line 3 column 5".to_owned(),
        ),
        (
            format!("permissions: ${{{{ inputs.level }}}}\njobs:\n  b:\n{job}"),
            "permissions: expression is unevaluated and not allowed in permissions".to_owned(),
        ),
        (
            format!("permissions:\n  contents: ${{{{ inputs.level }}}}\njobs:\n  b:\n{job}"),
            "permissions.contents: expression is unevaluated and not allowed in permissions"
                .to_owned(),
        ),
        (
            format!("permissions:\n  \"x\\x1b[31m\": read\njobs:\n  b:\n{job}"),
            "permissions.x\\u{1b}[31m: unknown permission scope 'x\\u{1b}[31m'".to_owned(),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
    }
}

#[test]
fn inspect_reports_job_timeouts_and_step_shells_without_applying_them() {
    let directory = tempfile::tempdir().unwrap();
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\n";
    for (source, report) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    timeout-minutes: 5\n    steps:\n      - run: touch executed\n        shell: bash\n      - uses: ./action\n",
            format!(
                "{head}Job: b\n  Runner: linux\n  Timeout minutes: 5 (not applied)\n  Step 1: run\n    Shell: bash (not selected)\n  Step 2: uses ./action\n"
            ),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    timeout-minutes: 0.5\n    steps:\n      - run: touch executed\n",
            format!(
                "{head}Job: b\n  Runner: linux\n  Timeout minutes: 0.5 (not applied)\n  Step 1: run\n"
            ),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    timeout-minutes: ${{ inputs.minutes }}\n    steps:\n      - run: touch executed\n        shell: \"pwsh -command \\\"& '{0}'\\\"\\x1b\"\n",
            format!(
                "{head}Job: b\n  Runner: linux\n  Timeout minutes: ${{{{ inputs.minutes }}}} (unevaluated; expression syntax not validated; not applied)\n  Step 1: run\n    Shell: pwsh -command \\\"& \\'{{0}}\\'\\\"\\u{{1b}} (not selected)\n"
            ),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{source}: {output:?}");
        assert!(output.stderr.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            report,
            "{source}"
        );
        assert!(!directory.path().join("executed").exists());
    }
}

#[test]
fn inspect_rejects_invalid_job_timeouts_and_step_shells() {
    let directory = tempfile::tempdir().unwrap();
    let job = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
    };
    let step = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
    };
    let timeout = "jobs.b.timeout-minutes: expected a positive number at line 3 column 5";
    for (source, diagnostic) in [
        (job("timeout-minutes: true"), timeout),
        (job("timeout-minutes: -1"), timeout),
        (job("timeout-minutes: 0"), timeout),
        (job("timeout-minutes: .inf"), timeout),
        (job("timeout-minutes: '5'"), timeout),
        (job("timeout-minutes: ~"), timeout),
        (job("timeout-minutes: [5]"), timeout),
        (step("shell: ~"), "jobs.b.steps[1].shell: expected a string at line 5 column 9"),
        (step("shell: 5"), "jobs.b.steps[1].shell: expected a string at line 5 column 9"),
        (step("shell: ' '"), "jobs.b.steps[1].shell: expected a nonempty string"),
        (
            step("shell: ${{ matrix.shell }}"),
            "jobs.b.steps[1].shell: expression is unevaluated and not allowed in shell",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./action\n        shell: bash\n"
                .to_owned(),
            "jobs.b.steps[1].shell: shell applies only to run steps",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        uses: ./action\n        shell: ''\n"
                .to_owned(),
            "jobs.b.steps[1]: expected exactly one of run or uses",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - shell: bash\n".to_owned(),
            "jobs.b.steps[1]: expected exactly one of run or uses",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
    }
}

#[test]
fn inspect_reports_step_inputs_without_binding_or_passing_them() {
    let directory = tempfile::tempdir().unwrap();
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nJob: b\n  Runner: linux\n";
    for (source, report) in [
        (
            r#"jobs:
  b:
    runs-on: linux
    steps:
      - uses: ./action
        with:
          persist-credentials: false
          python-version: "3.13"
          retries: 3
          token: ${{ secrets.TOKEN }}
      - run: touch executed
        working-directory: examples/python-basic
        env:
          ZED: "$(touch executed)\x1b[31m"
          MODE: ${{ inputs.mode }}
      - run: echo
        working-directory: "a\x07\"b\\"
        env:
          "K\x1b\"": "q\"\\\x07"
"#
            .to_owned(),
            format!(
                "{head}  Step 1: uses ./action\n    With (not passed to any action):\n      persist-credentials: false\n      python-version: \"3.13\"\n      retries: 3\n      token: \"${{{{ secrets.TOKEN }}}}\" (unevaluated)\n  Step 2: run\n    Working directory: examples/python-basic (not resolved)\n    Env (not bound):\n      MODE: \"${{{{ inputs.mode }}}}\" (unevaluated)\n      ZED: \"$(touch executed)\\u{{1b}}[31m\"\n{}",
                r#"  Step 3: run
    Working directory: a\u{7}\"b\\ (not resolved)
    Env (not bound):
      K\u{1b}\": "q\"\\\u{7}"
"#
            ),
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: touch executed\n        working-directory: ${{ github.workspace }}/x\n        env: {}\n      - uses: ./action\n        with: {}\n".to_owned(),
            format!(
                "{head}  Step 1: run\n    Working directory: ${{{{ github.workspace }}}}/x (unevaluated; expression syntax not validated; not resolved)\n    Env (not bound): {{}}\n  Step 2: uses ./action\n    With (not passed to any action): {{}}\n"
            ),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{source}: {output:?}");
        assert!(output.stderr.is_empty(), "{source}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), report, "{source}");
        assert!(!directory.path().join("executed").exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}

// Backlog B-11: workflow and job `env` and `defaults.run` are source data, never bound or applied.
#[test]
fn inspect_reports_workflow_and_job_env_and_run_defaults_without_applying_them() {
    let directory = tempfile::tempdir().unwrap();
    let source = r#"env:
  ZED: "$(touch executed)\x1b[31m"
  MODE: ${{ inputs.mode }}
  N: 3
defaults:
  run:
    shell: bash
    working-directory: src
jobs:
  b:
    runs-on: linux
    env: {}
    defaults:
      run:
        shell: ${{ matrix.shell }}
        working-directory: ${{ github.workspace }}
    steps:
      - run: touch executed
  c:
    runs-on: linux
    defaults: {run: {}}
    env: {"A\u202e": "\u200b"}
    steps: [{run: echo}]
"#;
    let report = r#"Workflow: (unnamed)
Inspection only; execution compatibility is not assessed.
Env (not bound):
  MODE: "${{ inputs.mode }}" (unevaluated)
  N: 3
  ZED: "$(touch executed)\u{1b}[31m"
Run defaults (not applied):
  Shell: bash
  Working directory: src
Job: b
  Runner: linux
  Env (not bound): {}
  Run defaults (not applied):
    Shell: ${{ matrix.shell }} (unevaluated; expression syntax not validated)
    Working directory: ${{ github.workspace }} (unevaluated; expression syntax not validated)
  Step 1: run
Job: c
  Runner: linux
  Env (not bound):
    A\u{202e}: "\u{200b}"
  Run defaults (not applied): {}
  Step 1: run
"#;
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), report);
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_invalid_workflow_and_job_env_and_run_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let job = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    let in_job = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
    };
    let dynamic = "expression is unevaluated; dynamic inspection is not supported yet";
    for (source, diagnostic) in [
        (
            format!("env: ~\n{job}"),
            "env: expected a mapping".to_owned(),
        ),
        (
            format!("env: ${{{{ x }}}}\n{job}"),
            format!("env: {dynamic}"),
        ),
        (
            format!("env: {{A: [1]}}\n{job}"),
            "env.A: expected a string, number or boolean".to_owned(),
        ),
        (
            in_job("env: {a: x, A: y}"),
            "jobs.b.env.A: duplicate key 'A'; already defined as 'a'".to_owned(),
        ),
        (
            format!("defaults: ~\n{job}"),
            "defaults: expected a mapping".to_owned(),
        ),
        (
            format!("defaults: ${{{{ x }}}}\n{job}"),
            "defaults: expression is unevaluated and not allowed in defaults".to_owned(),
        ),
        (
            format!("defaults: {{bogus: 1}}\n{job}"),
            "defaults: unknown field `bogus`, expected `run`".to_owned(),
        ),
        (
            format!("defaults: {{run: ~}}\n{job}"),
            "defaults.run: expected a mapping".to_owned(),
        ),
        (
            format!("defaults: {{run: {{bogus: 1}}}}\n{job}"),
            "defaults.run: unknown field `bogus`, expected `shell` or `working-directory`"
                .to_owned(),
        ),
        (
            format!("defaults: {{run: '${{{{ x }}}}'}}\n{job}"),
            "defaults.run: expression is unevaluated and not allowed in workflow defaults"
                .to_owned(),
        ),
        (
            format!("defaults: {{run: {{shell: '${{{{ x }}}}'}}}}\n{job}"),
            "defaults.run.shell: expression is unevaluated and not allowed in workflow defaults"
                .to_owned(),
        ),
        (
            format!("defaults: {{run: {{shell: ''}}}}\n{job}"),
            "defaults.run.shell: expected a nonempty string".to_owned(),
        ),
        (
            format!("defaults: {{run: {{working-directory: 3}}}}\n{job}"),
            "defaults.run.working-directory: expected a string".to_owned(),
        ),
        (
            in_job("defaults: {run: '${{ x }}'}"),
            format!("jobs.b.defaults.run: {dynamic}"),
        ),
        (
            in_job("defaults: {run: {working-directory: ''}}"),
            "jobs.b.defaults.run.working-directory: expected a nonempty string".to_owned(),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
    }
}

#[test]
fn inspect_rejects_invalid_step_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let run = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
    };
    let uses = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./a\n        {field}\n")
    };
    let dynamic = "expression is unevaluated; dynamic inspection is not supported yet";
    for (source, diagnostic) in [
        (
            run("env: ~"),
            "jobs.b.steps[1].env: expected a mapping".to_owned(),
        ),
        (
            run("env: [A]"),
            "jobs.b.steps[1].env: expected a mapping".to_owned(),
        ),
        (
            run("env: ${{ fromJSON(x) }}"),
            format!("jobs.b.steps[1].env: {dynamic}"),
        ),
        (
            run("env: {A: ~}"),
            "jobs.b.steps[1].env.A: expected a string, number or boolean".to_owned(),
        ),
        (
            run("env: {A: [1]}"),
            "jobs.b.steps[1].env.A: expected a string, number or boolean".to_owned(),
        ),
        (
            run("env: {'': x}"),
            "jobs.b.steps[1].env: expected a nonempty string key".to_owned(),
        ),
        (
            run("env: {1: x}"),
            "jobs.b.steps[1].env: non-string key is not inspected yet".to_owned(),
        ),
        (
            run("env: {'${{ x }}': y}"),
            format!("jobs.b.steps[1].env.${{{{ x }}}}: {dynamic}"),
        ),
        (
            run("env: {Mode: a, MODE: b}"),
            "jobs.b.steps[1].env.MODE: duplicate key 'MODE'; already defined as 'Mode'".to_owned(),
        ),
        (
            run(r#"env: {"K\x1b\"": [1]}"#),
            "jobs.b.steps[1].env.K\\u{1b}\": expected a string, number or boolean".to_owned(),
        ),
        (
            uses("with: {a: {b: c}}"),
            "jobs.b.steps[1].with.a: expected a string, number or boolean".to_owned(),
        ),
        (
            uses("with: {token: a, Token: b}"),
            "jobs.b.steps[1].with.Token: duplicate key 'Token'; already defined as 'token'"
                .to_owned(),
        ),
        (
            run("with: {a: b}"),
            "jobs.b.steps[1].with: with applies only to uses steps".to_owned(),
        ),
        (
            run("with: ${{ inputs.args }}"),
            "jobs.b.steps[1].with: with applies only to uses steps".to_owned(),
        ),
        (
            run("uses: ./a\n        env: {A: [1]}"),
            "jobs.b.steps[1]: expected exactly one of run or uses".to_owned(),
        ),
        (
            run("shell: ' '\n        env: {A: [1]}"),
            "jobs.b.steps[1].shell: expected a nonempty string".to_owned(),
        ),
        (
            uses("working-directory: x"),
            "jobs.b.steps[1].working-directory: working-directory applies only to run steps"
                .to_owned(),
        ),
        (
            run("working-directory: 5"),
            "jobs.b.steps[1].working-directory: expected a string at line 5 column 9".to_owned(),
        ),
        (
            run("working-directory: ' '"),
            "jobs.b.steps[1].working-directory: expected a nonempty string".to_owned(),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
    }
}

#[test]
fn inspect_warns_above_the_github_workflow_size_limit_and_still_reports() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n# ";
    let report = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nJob: b\n  Runner: linux\n  Step 1: run\n";
    for size in [511_999, 512_000, 512_001] {
        let source = format!("{workflow}{}\n", "x".repeat(size - workflow.len() - 1));
        assert_eq!(source.len(), size);
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{size}: {output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), report, "{size}");
        let warning = if size > 512_000 {
            format!(
                "warning: wsr inspect: ci.yml: {size} bytes exceeds GitHub's documented 500 KB (512000-byte) workflow file limit; GitHub would not run this workflow\n"
            )
        } else {
            String::new()
        };
        assert_eq!(String::from_utf8(output.stderr).unwrap(), warning, "{size}");
    }
}

#[test]
fn inspect_size_warning_does_not_panic_when_stderr_is_closed() {
    use std::io::Read as _;
    use std::process::Stdio;

    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n# ";
    let source = format!("{workflow}{}\n", "x".repeat(512_001 - workflow.len() - 1));
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", "ci.yml"])
        .current_dir(directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn inspection");
    drop(child.stderr.take());
    let mut stdout = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(0));
    assert_eq!(
        stdout,
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nJob: b\n  Runner: linux\n  Step 1: run\n"
    );
}
