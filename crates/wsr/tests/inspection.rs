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
