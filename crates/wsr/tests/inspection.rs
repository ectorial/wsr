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
            "unknown field `needs`",
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
