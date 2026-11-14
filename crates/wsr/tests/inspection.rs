use std::fs;
use std::path::Path;
use std::process::{Command, Output};

/// A block-mapping fixture whose root lacks `on`, which the pinned schema requires (backlog B-25),
/// gets `on: workflow_dispatch` appended after its content, so diagnostic locations are unchanged.
/// A flow-style root (`{…}` or `[…]`) is left as written.
fn with_on(source: impl AsRef<[u8]>) -> Vec<u8> {
    let mut source = source.as_ref().to_vec();
    let text = String::from_utf8_lossy(&source);
    let flow = text.trim_start().starts_with(['{', '[']);
    let has_on = text.lines().any(|line| {
        ["on:", "on ", "\"on\"", "'on'"]
            .iter()
            .any(|start| line.starts_with(start))
    });
    if !flow && !has_on {
        if !source.is_empty() && !source.ends_with(b"\n") {
            source.push(b'\n');
        }
        source.extend_from_slice(b"on: workflow_dispatch\n");
    }
    source
}

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
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    fs::write(directory.path().join("wsr.toml"), "invalid config sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Local CI\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: test (Tests)\n  Runner: ubuntu-latest\n  Step 1: run (Dangerous command)\n  Step 2: uses ./action\n"
    );
    assert_eq!(
        fs::read(directory.path().join("ci.yml")).unwrap(),
        with_on(workflow)
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "invalid config sentinel"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
}

// B-24 contract change: `${{` text in job and step `name` and in `runs-on` was rejected as
// unsupported; it is now reported as written, never evaluated.
#[test]
fn inspect_reports_name_and_runner_expressions_unevaluated() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  a:\n    name: \"${{ matrix.os }}$(touch executed)\\e[31m\\u202E\"\n    runs-on: ${{ matrix.os }}\n    steps:\n      - name: ${{ inputs.step }}\n        run: touch executed\n  b:\n    runs-on: [linux, \"${{ vars.POOL }}\\e\"]\n    steps:\n      - run: echo hello\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a (${{ matrix.os }}$(touch executed)\\u{1b}[31m\\u{202e}; unevaluated; expression syntax not validated)\n  Runner: ${{ matrix.os }} (unevaluated; expression syntax not validated)\n  Step 1: run (${{ inputs.step }}; unevaluated; expression syntax not validated)\nJob: b\n  Runner labels: [\"linux\", \"${{ vars.POOL }}\\u{1b}\"] (some unevaluated; expression syntax not validated)\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
}

// Backlog B-28: `${{` text in a step `run` is valid GHA that is reported, never evaluated; the
// script itself is never printed.
#[test]
fn inspect_reports_run_expressions_without_printing_the_script() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  a:\n    runs-on: linux\n    steps:\n      - name: Hostile\n        run: \"echo ${{ inputs.mode }}$(touch executed)\\e[31m\\u202E\"\n      - run: echo ${{ github.ref }}\n        shell: bash\n      - run: touch executed\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect_with_execution_traps(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        report,
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner: linux\n  Step 1: run (script has unevaluated expressions; not evaluated) (Hostile)\n  Step 2: run (script has unevaluated expressions; not evaluated)\n    Shell: bash (not selected)\n  Step 3: run\n"
    );
    for script in ["echo", "touch", "inputs.mode", "github.ref"] {
        assert!(!report.contains(script), "{script}: {report}");
    }
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

// Backlog B-29: a job `container` is reported as written; nothing is pulled, started or logged in
// to, and a password is never printed.
#[test]
fn inspect_reports_job_containers_without_pulling_or_starting_them() {
    let directory = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();
    let marker = directory.path().join("executed");
    #[cfg(unix)]
    for command in ["docker", "podman", "sh"] {
        use std::os::unix::fs::PermissionsExt;
        let path = bin.path().join(command);
        fs::write(
            &path,
            "#!/bin/sh\n/usr/bin/touch \"$WSR_EXECUTION_MARKER\"\n",
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let workflow = "jobs:\n  a:\n    runs-on: linux\n    container: \"alpine$(touch executed)\\e[31m\\u202E\"\n    steps:\n      - run: echo\n  b:\n    runs-on: linux\n    container:\n      image: ${{ matrix.image }}\n      credentials:\n        username: bot\n        password: hunter2-secret\n      env: {A: \"$(touch executed)\", N: 1}\n      ports: [80, '${{ inputs.port }}']\n      volumes: [/data]\n      options: --cpus 1 $(touch executed)\n    steps:\n      - run: echo\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", "ci.yml"])
        .current_dir(directory.path())
        .env("PATH", bin.path())
        .env("WSR_EXECUTION_MARKER", &marker)
        .output()
        .expect("run inspection");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        report,
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner: linux\n  Container (not pulled; not started):\n    Image: alpine$(touch executed)\\u{1b}[31m\\u{202e}\n  Step 1: run\nJob: b\n  Runner: linux\n  Container (not pulled; not started):\n    Image: ${{ matrix.image }} (unevaluated; expression syntax not validated)\n    Credentials (not used):\n      Username: bot\n      Password: set (not shown)\n    Env (not bound):\n      A: \"$(touch executed)\"\n      N: 1\n    Ports (not exposed): [80, \"${{ inputs.port }}\"] (some unevaluated; expression syntax not validated)\n    Volumes (not mounted): [\"/data\"]\n    Options: \"--cpus 1 $(touch executed)\" (not passed)\n  Step 1: run\n"
    );
    assert!(!report.contains("hunter2"), "{report}");
    assert!(!marker.exists(), "a command was executed");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

// A mismatched tag on a container password must not echo it through the decoder's error.
#[test]
fn inspect_does_not_quote_container_credentials_in_decoder_errors() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  a:\n    runs-on: linux\n    container: {image: i, credentials: {password: !!int hunter2-secret}}\n    steps:\n      - run: echo\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("detail withheld"), "{stderr}");
    assert!(!stderr.contains("hunter2"), "{stderr}");
}

// A matrix amplified through YAML aliases is rejected, not rendered (B-30 review P2).
#[test]
fn inspect_rejects_alias_amplified_matrices_without_output() {
    let directory = tempfile::tempdir().unwrap();
    let scalar = "x".repeat(10_000);
    let refs = |name: &str| vec![format!("*{name}"); 10].join(", ");
    let workflow = format!(
        "jobs:\n  a:\n    runs-on: linux\n    strategy:\n      matrix: {{a: &a ['{scalar}'], b: &b [{}], c: &c [{}]}}\n    steps:\n      - run: echo\n",
        refs("a"),
        refs("b")
    );
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("jobs.a.strategy.matrix: matrix expands beyond twice the workflow source size through YAML aliases; not inspected yet"),
        "{stderr}"
    );
    assert!(stderr.len() < 1_000, "{stderr}");
}

#[test]
fn inspect_diagnoses_invalid_or_unsupported_source_without_partial_output() {
    let directory = tempfile::tempdir().unwrap();
    for (source, diagnostic) in [
        (
            "jobs:\n  test:\n    runs-on: linux\n    steps:\n      - run: null\n        uses: ./action",
            "non-string value is not inspected yet",
        ),
        ("jobs: {}", "jobs: expected at least one job"),
        (
            "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps: []",
            "jobs.test.steps: expected at least one step",
        ),
        (
            "jobs:\n  test:\n    runs-on: 42\n    steps:\n      - run: echo hello",
            "non-string value is not inspected yet",
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
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on("on: workflow_dispatch\njobs:\n  zebra:\n    runs-on: linux\n    steps:\n      - uses: owner/action@v1\n      - run: echo last\n  alpha:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo first\n")).unwrap();
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
        with_on(include_str!(
            "../../../tests/fixtures/inspection/static-needs.yml"
        )),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Dependency example\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: build\n  Runner: ubuntu-latest\n  Step 1: run\nJob: package\n  Runner: ubuntu-latest\n  Needs: build, test\n  Step 1: run\nJob: test\n  Runner: ubuntu-latest\n  Needs: build\n  Step 1: run\n"
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
            with_on(dependency_workflow(&[("build", ""), ("test", needs)])),
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
        with_on(dependency_workflow(&[("build", "    needs: build\n")])),
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
        with_on(dependency_workflow(&[
            ("downstream", "    needs: one\n"),
            ("one", "    needs: two\n"),
            ("two", "    needs: [three]\n"),
            ("three", "    needs: one\n"),
        ])),
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
        with_on(dependency_workflow(&[
            ("package", "    needs: [test, build, test]\n"),
            ("test", "    needs: [build]\n"),
            ("build", "    needs: last\n"),
            ("last", "    needs: []\n"),
            ("disconnected", ""),
        ])),
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
        with_on(dependency_workflow(&[
            ("cycle", "    needs: cycle\n"),
            ("test", "    needs: [missing]\n"),
        ])),
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
            with_on(dependency_workflow(&[
                ("build", ""),
                ("test", &format!("    needs: {value}\n")),
            ])),
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
        fs::write(root.join("ci.yml"), with_on(&source)).unwrap();
        let output = inspect(root, "ci.yml");
        assert_eq!(output.status.code(), Some(status), "{output:?}");
        assert_eq!(fs::read(root.join("ci.yml")).unwrap(), with_on(&source));
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Step identity example\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: build\n  Runner: ubuntu-latest\n  Step 1 [compile]: run (Compile sources)\n  Step 2 [upload]: uses ./unopened-action\n  Step 3: run\n"
    );
    assert_eq!(
        fs::read(directory.path().join("ci.yml")).unwrap(),
        with_on(source)
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
        ("true".to_owned(), "non-string value is not inspected yet"),
        ("[]".to_owned(), "expected a string"),
        ("{}".to_owned(), "expected a string"),
    ] {
        let source = dependency_workflow(&[("build", "")]).replace(
            "      - run:",
            &format!("      - id: {value}\n        run:"),
        );
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
    fs::write(directory.path().join("wsr.toml"), "sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: build\n  Runner labels: [\"self-hosted\", \"linux\", \"x64\"]\n  Step 1 [compile]: run\n  Step 2: uses ./unopened-action\nJob: test\n  Runner: ubuntu-latest\n  Needs: build\n  Step 1: run\n"
    );
    assert_eq!(
        fs::read(directory.path().join("ci.yml")).unwrap(),
        with_on(source)
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("wsr.toml")).unwrap(),
        "sentinel"
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_runner_labels_reject_empty_and_invalid_values() {
    let directory = tempfile::tempdir().unwrap();
    for (value, diagnostic) in [
        ("[]", "runs-on: expected at least one label"),
        ("[linux, ' ']", "runs-on[2]: expected a nonempty string"),
        // B-24: `${{` labels now inspect (`inspect_reports_name_and_runner_expressions_unevaluated`).
        ("null", "runs-on: expected a string or list of strings"),
        ("42", "runs-on: non-string value is not inspected yet"),
        ("true", "runs-on: non-string value is not inspected yet"),
        (
            "[linux, 42]",
            "runs-on[2]: non-string value is not inspected yet",
        ),
        (
            "[linux, true]",
            "runs-on[2]: non-string value is not inspected yet",
        ),
        ("[linux, null]", "runs-on[2]: expected a string label"),
        ("[linux, [x64]]", "runs-on[2]: expected a string label"),
        ("[linux, {}]", "runs-on[2]: expected a string label"),
        // B-35 contract change: selectors inspect (`inspect_reports_runner_selectors_without_selecting_a_runner`).
        (
            "{}",
            "runs-on: empty runner-selector mapping; not inspected yet",
        ),
        (
            "{group: team/fleet}",
            "runs-on.group: invalid runner group name",
        ),
    ] {
        let source = format!(
            "jobs:\n  build:\n    runs-on: {value}\n    steps:\n      - run: touch executed\n"
        );
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{value}: {output:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("ci.yml:") && error.contains("jobs.build") && error.contains(diagnostic),
            "{value}: {error}"
        );
        assert_eq!(
            fs::read(directory.path().join("ci.yml")).unwrap(),
            with_on(&source)
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}

#[test]
fn inspect_runner_labels_quote_boundaries_preserve_case_order_and_repetitions() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("ci.yml"), with_on("jobs:\n  build:\n    runs-on: ['Linux', 'a,b', 'Linux', \"line\\nbreak\", 'quote\"label']\n    steps:\n      - run: echo hello\n  single:\n    runs-on: [linux]\n    steps:\n      - run: echo hello\n")).unwrap();
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
    fs::write(directory.path().join("wsr.toml"), "sentinel").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: Manual input example\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n  Input: mode\n    Type: choice\n    Description: Choose the reference outcome\n    Required: true\n    Default: \"pass\" (not applied)\n    Options: [\"pass\", \"fail\"]\nJob: build\n  Runner: ubuntu-latest\n  Step 1 [compile]: run\n  Step 2: uses ./unopened-action\n"
    );
    assert_eq!(
        fs::read(directory.path().join("ci.yml")).unwrap(),
        with_on(source)
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
        fs::write(
            directory.path().join("ci.yml"),
            with_on(dispatch_workflow(fields)),
        )
        .unwrap();
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
        (
            "        description: 42",
            ".description: non-string value is not inspected yet",
        ),
        (
            "        description: null",
            ".description: non-string value is not inspected yet",
        ),
        ("        required: 'true'", ".required: expected a boolean"),
        ("        required: null", ".required: expected a boolean"),
        ("        type: object", ".type: unsupported input type"),
        ("        type: []", ".type: expected a string"),
        (
            "        default: null",
            ".default: non-string value is not inspected yet",
        ),
        (
            "        type: boolean\n        default: null",
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
            ".options[2]: non-string value is not inspected yet",
        ),
        (
            "        type: choice\n        options: [a, null]",
            ".options[2]: non-string value is not inspected yet",
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
            "on.workflow_dispatch.inputs.value: unknown field `unexpected`",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(dispatch_workflow(fields)),
        )
        .unwrap();
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
        // B-15 contract change: sequences and bare non-dispatch events now inspect.
        ("null", "on: expected an event name, sequence or mapping"),
        ("[]", "on: an empty event list is not inspected yet"),
        ("{}", "on: an empty event list is not inspected yet"),
        (
            // B-22 inspects `push` filters; another event keeps this sample.
            "{release: {types: [published]}}",
            "on.release: event configuration is not inspected yet",
        ),
        (
            "{workflow_dispatch: {unexpected: true}}",
            "on.workflow_dispatch: unknown field `unexpected`",
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
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
            with_on(format!(
                "on: {trigger}\n{}",
                dependency_workflow(&[("build", "")])
            )),
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
        with_on(format!("{source}{}", dependency_workflow(&[("build", "")]))),
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
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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

const RUN_NAME_HEADER: &str = "Workflow: Example shell reference\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n";

fn run_name_workflow(run_name: &str) -> String {
    format!(
        "name: Example shell reference\n{run_name}on:\n  workflow_dispatch:\n    inputs:\n      mode:\n        type: string\n        default: quick\njobs:\n  shell:\n    runs-on: ubuntu-latest\n    steps:\n      - run: touch executed\n      - uses: ./action\n"
    )
}

#[test]
fn inspect_run_name_reports_unevaluated_source_without_executing_or_binding_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = run_name_workflow("run-name: \"Shell reference — ${{ inputs.mode }}\"\n");
    fs::write(directory.path().join("ci.yml"), with_on(&workflow)).unwrap();
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
        fs::read(directory.path().join("ci.yml")).unwrap(),
        with_on(&workflow)
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
            with_on(run_name_workflow(declaration)),
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
        ("run-name: null\n", "non-string value is not inspected yet"),
        ("run-name: ~\n", "non-string value is not inspected yet"),
        ("run-name: true\n", "non-string value is not inspected yet"),
        ("run-name: 42\n", "non-string value is not inspected yet"),
        ("run-name: [a]\n", "expected a string"),
        ("run-name: {a: b}\n", "expected a string"),
        ("run-name: ''\n", "run-name: expected a nonempty string"),
        ("run-name: '  '\n", "run-name: expected a nonempty string"),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(run_name_workflow(declaration)),
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
fn inspect_reports_unsupported_fields_without_claiming_them_valid() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("ci.yml"), with_on("jobs:\n  b:\n    runs-on: linux\n    services: {}\n    steps:\n      - run: touch executed\n"),
    )
    .unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: wsr inspect: ci.yml: jobs.b: unsupported field `services`; this field is not inspected yet at line 4 column 5\n"
    );
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_diagnostics_use_one_based_field_routes_and_validate_job_ids_first() {
    let directory = tempfile::tempdir().unwrap();
    let job = "    runs-on: linux\n    steps:\n      - run: echo\n";
    for (source, diagnostic) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        bogus: {}\n"
                .to_owned(),
            "jobs.b.steps[1]: unknown field `bogus`, expected one of `id`, `name`, `if`, `run`, `shell`, `working-directory`, `env`, `with`, `uses`, `continue-on-error`, `timeout-minutes` at line 6 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - id: ~\n        run: echo\n"
                .to_owned(),
            "jobs.b.steps[1].id: expected a string at line 5 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n      - run: echo\n        name: 5\n"
                .to_owned(),
            "jobs.b.steps[2].name: non-string value is not inspected yet at line 6 column 9",
        ),
        (
            "jobs:\n  b:\n    runs-on: [linux, ~]\n    steps:\n      - run: echo\n".to_owned(),
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
            format!("jobs:\n  __build:\n{job}"),
            "jobs.__build: invalid job identifier '__build'",
        ),
        (
            format!("jobs:\n  {}:\n{job}", "j".repeat(100)),
            &format!(
                "jobs.{0}: invalid job identifier '{0}'",
                "j".repeat(100)
            ),
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
        (
            format!("jobs:\n  build:\n{job}  Build:\n{job}"),
            "jobs.Build: duplicate job identifier 'Build'; already defined as 'build' at line 2 column 3",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
            "jobs.b.steps[1]: unknown field `k\\u{202e}q\\u{200b}`, expected one of `id`, `name`, `if`, `run`, `shell`, `working-directory`, `env`, `with`, `uses`, `continue-on-error`, `timeout-minutes` at line 6 column 9",
        ),
        (
            format!("jobs:\n  \"b\\u202E\\u200B\\u2066\\u2069\":\n{job}"),
            "jobs.b\\u{202e}\\u{200b}\\u{2066}\\u{2069}: invalid job identifier 'b\\u{202e}\\u{200b}\\u{2066}\\u{2069}'",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
    let mut source = String::from("on: workflow_dispatch\njobs:\n");
    for index in 0..5000 {
        source.push_str(&format!(
            "  job{index:05}:\n    name: A reasonably long display name for job {index}\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n"
        ));
    }
    let size = source.len();
    fs::write(directory.path().join("big.yml"), with_on(source)).unwrap();
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
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n";
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
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\n";
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
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
    let head = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n";
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
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
Trigger: workflow_dispatch (not evaluated)
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
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
            // B-13: the runner rejects a converted `3` shell, unlike a converted directory.
            format!("defaults: {{run: {{shell: 3}}}}\n{job}"),
            "defaults.run.shell: expected a string".to_owned(),
        ),
        (
            format!("defaults: {{run: {{working-directory: 3}}}}\n{job}"),
            "defaults.run.working-directory: non-string value is not inspected yet".to_owned(),
        ),
        (
            format!("defaults: {{run: {{working-directory: [a]}}}}\n{job}"),
            "defaults.run.working-directory: expected a string".to_owned(),
        ),
        (
            in_job("defaults: {run: {'${{ matrix.key }}': bash}}"),
            format!("jobs.b.defaults.run.${{{{ matrix.key }}}}: {dynamic}"),
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
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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

// Backlog B-12: job and step `if` are reported as written, never evaluated.
#[test]
fn inspect_reports_job_and_step_conditions_without_evaluating_them() {
    let directory = tempfile::tempdir().unwrap();
    let source = r#"jobs:
  b:
    if: github.event_name == 'push'
    runs-on: linux
    steps:
      - if: ${{ failure() }}
        run: touch executed
      - uses: ./action
        if: "$(touch executed)\x1b[31m\u202e"
  c:
    runs-on: linux
    if: ${{ inputs.mode == 'fail' }} && always()
    steps: [{run: echo}]
"#;
    let report = r#"Workflow: (unnamed)
Inspection only; execution compatibility is not assessed.
Trigger: workflow_dispatch (not evaluated)
Job: b
  Runner: linux
  Condition: github.event_name == \'push\' (not evaluated; expression syntax not validated)
  Step 1: run
    Condition: ${{ failure() }} (not evaluated; expression syntax not validated)
  Step 2: uses ./action
    Condition: $(touch executed)\u{1b}[31m\u{202e} (not evaluated; expression syntax not validated)
Job: c
  Runner: linux
  Condition: ${{ inputs.mode == \'fail\' }} && always() (not evaluated; expression syntax not validated)
  Step 1: run
"#;
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), report);
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_conditions_outside_the_inspected_subset() {
    let directory = tempfile::tempdir().unwrap();
    let in_job = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
    };
    let in_step = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
    };
    for (source, diagnostic) in [
        (
            in_job("if: true"),
            "jobs.b.if: non-string condition is not inspected yet at line 3 column 5",
        ),
        (
            in_step("if: 0"),
            "jobs.b.steps[1].if: non-string condition is not inspected yet at line 5 column 9",
        ),
        (
            in_job("if: ''"),
            "jobs.b.if: expected a nonempty condition at line 3 column 5",
        ),
        (
            in_step("if: ' '"),
            "jobs.b.steps[1].if: expected a nonempty condition at line 5 column 9",
        ),
        (
            in_job("if: ~"),
            "jobs.b.if: expected a nonempty condition at line 3 column 5",
        ),
        (
            in_step("if: [a]"),
            "jobs.b.steps[1].if: expected a string at line 5 column 9",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
            "jobs.b.steps[1].env.A: non-string value is not inspected yet".to_owned(),
        ),
        (
            uses("with: {a: ~}"),
            "jobs.b.steps[1].with.a: non-string value is not inspected yet".to_owned(),
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
            "jobs.b.steps[1].working-directory: non-string value is not inspected yet at line 5 column 9".to_owned(),
        ),
        (
            run("working-directory: ' '"),
            "jobs.b.steps[1].working-directory: expected a nonempty string".to_owned(),
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
    let workflow =
        "on: workflow_dispatch\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n# ";
    let report = "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n  Step 1: run\n";
    for size in [511_999, 512_000, 512_001] {
        let source = format!("{workflow}{}\n", "x".repeat(size - workflow.len() - 1));
        assert_eq!(source.len(), size);
        fs::write(directory.path().join("ci.yml"), with_on(&source)).unwrap();
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
    let workflow =
        "on: workflow_dispatch\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n# ";
    let source = format!("{workflow}{}\n", "x".repeat(512_001 - workflow.len() - 1));
    fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
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
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n  Step 1: run\n"
    );
}

// Backlog B-14: a shell or action reference the runner rejects fails before stdout, with the
// hostile text escaped once and nothing run or fetched.
#[test]
fn inspect_rejects_unknown_shells_and_malformed_action_references() {
    let directory = tempfile::tempdir().unwrap();
    for (source, diagnostic) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        shell: \"touch executed\\e[31m\\u202E\"\n",
            "jobs.b.steps[1].shell: unknown shell 'touch executed\\u{1b}[31m\\u{202e}'; expected bash, pwsh, python, sh, cmd, powershell or a command containing {0}",
        ),
        (
            "defaults:\n  run:\n    shell: perl\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n",
            "defaults.run.shell: unknown shell 'perl'; expected bash, pwsh, python, sh, cmd, powershell or a command containing {0}",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: \"actions/checkout\\e\\u202E\"\n",
            "jobs.b.steps[1].uses: invalid action reference 'actions/checkout\\u{1b}\\u{202e}'; expected ./path, docker://image or owner/repo[/path]@ref",
        ),
    ] {
        fs::write(directory.path().join("ci.yml"), with_on(source)).unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty(), "{source}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{source}"
        );
        assert!(!directory.path().join("executed").exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}

// Backlog B-15: bare `on:` events are reported in source order without being evaluated.
#[test]
fn inspect_reports_bare_trigger_events_in_source_order() {
    let directory = tempfile::tempdir().unwrap();
    let job = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: touch executed\n";
    for (on, triggers) in [
        ("workflow_call", "workflow_call"),
        (
            "[push, workflow_dispatch, pull_request]",
            "push workflow_dispatch pull_request",
        ),
        (
            "\n  workflow_call:\n  push: {}\n  workflow_dispatch:\n    inputs:\n      mode: {type: boolean}\n  create:",
            "workflow_call push workflow_dispatch create",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(format!("on: {on}\n{job}")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{on}: {output:?}");
        assert!(output.stderr.is_empty(), "{on}: {output:?}");
        let lines: String = triggers
            .split(' ')
            .map(|event| {
                let inputs = if event == "workflow_dispatch" && on.contains("inputs") {
                    "  Input: mode\n    Type: boolean\n"
                } else {
                    ""
                };
                format!("Trigger: {event} (not evaluated)\n{inputs}")
            })
            .collect();
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\n{lines}Job: b\n  Runner: linux\n  Step 1: run\n"
            ),
            "{on}"
        );
        assert!(!directory.path().join("executed").exists());
    }
}

#[test]
fn inspect_rejects_unknown_and_configured_trigger_events() {
    let directory = tempfile::tempdir().unwrap();
    let job = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    for (on, diagnostic) in [
        (
            "\"push\\e[31m\\u202E\"",
            "on: unknown event 'push\\u{1b}[31m\\u{202e}'",
        ),
        ("[push, \"x\\e\"]", "on[2]: unknown event 'x\\u{1b}'"),
        ("[push, push]", "on[2]: repeated event is not inspected yet"),
        (
            "{workflow_call: {inputs: {}}}",
            "on.workflow_call: event configuration is not inspected yet",
        ),
        ("{create: {}}", "on.create: expected null"),
        ("{schedule: null}", "on.schedule: expected a sequence"),
        ("{push: main}", "on.push: expected null or a mapping"),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(format!("on: {on}\n{job}")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{on}: {output:?}");
        assert!(output.stdout.is_empty(), "{on}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{on}"
        );
    }
}

// The repository's workflows, inspected as temporary copies only with every PATH command trapped
// (audit F33). `ci.yml` and `release.yml` pin their first diagnostic, updated as slices land.
#[test]
fn inspect_repository_workflows_from_temporary_copies() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows");
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("executed");
    let call = "\nTrigger: workflow_call (not evaluated)\n";
    let cases: [(&str, Result<&str, &str>); 6] = [
        ("check-docs.yml", Ok(call)),
        ("check-fmt.yml", Ok(call)),
        ("check-lint.yml", Ok(call)),
        ("test.yml", Ok(call)),
        (
            "ci.yml",
            Ok(
                "\nJob: check-fmt (format)\n  Calls workflow: ./.github/workflows/check-fmt.yml (not loaded; not resolved)\n",
            ),
        ),
        (
            "release.yml",
            Ok(
                "  Strategy (not expanded):\n    Fail-fast: false\n    Matrix (normalized YAML): ${{ fromJson(needs.plan.outputs.val).ci.github.artifacts_matrix }} (unevaluated; expression syntax not validated)\n",
            ),
        ),
    ];
    // Audit F40: every repository workflow except the dogfood examples is pinned here.
    let mut names: Vec<String> = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".yml") && !name.starts_with("example-"))
        .collect();
    names.sort();
    let mut pinned: Vec<&str> = cases.iter().map(|(name, _)| *name).collect();
    pinned.sort();
    assert_eq!(names, pinned);
    for (name, expected) in cases {
        let source = fs::read_to_string(root.join(name)).unwrap();
        fs::write(directory.path().join(name), &source).unwrap();
        let output = inspect_with_execution_traps(directory.path(), name);
        match expected {
            Ok(fragment) => {
                assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
                assert!(output.stderr.is_empty(), "{name}: {output:?}");
                let report = String::from_utf8(output.stdout).unwrap();
                assert!(report.contains(fragment), "{name}: {report}");
            }
            Err(diagnostic) => {
                assert_eq!(output.status.code(), Some(1), "{name}: {output:?}");
                assert!(output.stdout.is_empty(), "{name}: {output:?}");
                assert_eq!(
                    String::from_utf8(output.stderr).unwrap(),
                    format!("error: wsr inspect: {name}: {diagnostic}\n"),
                );
            }
        }
        assert_eq!(
            fs::read_to_string(directory.path().join(name)).unwrap(),
            source
        );
    }
    assert!(!marker.exists(), "a command was executed");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), cases.len());
}

// Backlog B-23: a reusable-workflow call is reported as written; the target is never opened.
#[test]
fn inspect_reports_workflow_calls_without_loading_them() {
    let directory = tempfile::tempdir().unwrap();
    // A target beside the caller that would leave a marker if anything ran it.
    fs::create_dir_all(directory.path().join(".github/workflows")).unwrap();
    let target = "on: workflow_call\njobs:\n  t:\n    runs-on: linux\n    steps:\n      - run: touch executed\n";
    fs::write(
        directory.path().join(".github/workflows/target.yml"),
        target,
    )
    .unwrap();
    let workflow = "jobs:\n  a:\n    uses: ./.github/workflows/target.yml\n    with:\n      mode: \"x$(touch executed)\\e[31m\\u202E\"\n      ref: ${{ github.ref }}\n    secrets: inherit\n  b:\n    uses: ./.github/workflows/missing.yml\n    secrets:\n      token: ${{ secrets.T }}\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Calls workflow: ./.github/workflows/target.yml (not loaded; not resolved)\n  With (not passed):\n    mode: \"x$(touch executed)\\u{1b}[31m\\u{202e}\"\n    ref: \"${{ github.ref }}\" (unevaluated)\n  Secrets: inherit (not passed)\nJob: b\n  Calls workflow: ./.github/workflows/missing.yml (not loaded; not resolved)\n  Secrets (not passed):\n    token: \"${{ secrets.T }}\" (unevaluated)\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert!(
        !directory
            .path()
            .join(".github/workflows/missing.yml")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(directory.path().join(".github/workflows/target.yml")).unwrap(),
        target
    );
}

#[test]
fn inspect_rejects_invalid_workflow_calls() {
    let directory = tempfile::tempdir().unwrap();
    for (job, diagnostic) in [
        (
            "    uses: ./w.yml\n    runs-on: linux\n",
            "jobs.b.runs-on: `runs-on` cannot be combined with `uses` at line 4 column 5",
        ),
        (
            "    uses: ./w.yml\n    steps:\n      - run: echo\n",
            "jobs.b.steps: `steps` cannot be combined with `uses` at line 4 column 5",
        ),
        (
            "    runs-on: linux\n    with: {a: b}\n    steps:\n      - run: echo\n",
            "jobs.b.with: `with` applies only to a reusable workflow call at line 4 column 5",
        ),
        (
            "    uses: '${{ x }}'\n",
            "jobs.b.uses: expression is unevaluated; dynamic inspection is not supported yet",
        ),
        (
            "    uses: octo/repo/w.yml\n",
            "jobs.b.uses: invalid workflow reference 'octo/repo/w.yml'; expected ./path or owner/repo/path@ref",
        ),
        (
            "    uses: ./w.yml\n    secrets: all\n",
            "jobs.b.secrets: expected `inherit` or a mapping",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(format!("jobs:\n  b:\n{job}")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{job}: {output:?}");
        assert!(output.stdout.is_empty(), "{job}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{job}"
        );
    }
}

// Backlog B-22: `push` and pull request filters are reported as written, never matched.
#[test]
fn inspect_reports_event_filters_without_matching_them() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "on:\n  push:\n    branches: [main, \"x$(touch executed)\\e[31m\\u202E\"]\n    tags-ignore: v*\n  pull_request:\n    paths: docs/**\n  pull_request_target:\n    branches-ignore: []\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: touch executed\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: push (not evaluated)\n  Filters (not matched):\n    branches: [\"main\", \"x$(touch executed)\\u{1b}[31m\\u{202e}\"]\n    tags-ignore: [\"v*\"]\nTrigger: pull_request (not evaluated)\n  Filters (not matched):\n    paths: [\"docs/**\"]\nTrigger: pull_request_target (not evaluated)\n  Filters (not matched):\n    branches-ignore: []\nJob: b\n  Runner: linux\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
}

#[test]
fn inspect_rejects_invalid_event_filters() {
    let directory = tempfile::tempdir().unwrap();
    let job = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    for (on, diagnostic) in [
        (
            "{pull_request: {tags: v1}}",
            "on.pull_request: unknown field `tags`",
        ),
        (
            "{push: {branches: main, branches-ignore: dev}}",
            "on.push.branches-ignore: cannot be combined with `branches`",
        ),
        (
            "{push: {branches: '${{ github.ref }}'}}",
            "on.push.branches: expression is unevaluated; dynamic inspection is not supported yet",
        ),
        (
            "{push: {paths: [a, '']}}",
            "on.push.paths[2]: expected a non-empty string",
        ),
        (
            "{push: {tags: 5}}",
            "on.push.tags: non-string value is not inspected yet",
        ),
        (
            "{pull_request: {types: [opened]}}",
            "on.pull_request.types: unsupported field `types`; this field is not inspected yet",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(format!("on: {on}\n{job}")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{on}: {output:?}");
        assert!(output.stdout.is_empty(), "{on}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{on}"
        );
    }
}

// Backlog B-16: workflow and job `concurrency` are reported as written, never applied.
#[test]
fn inspect_reports_concurrency_without_applying_it() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "concurrency: \"ci-$(touch executed)\\e[31m\\u202E\"\njobs:\n  b:\n    runs-on: linux\n    concurrency:\n      group: ${{ github.ref }}\n      cancel-in-progress: true\n      queue: max\n    steps:\n      - run: touch executed\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nConcurrency (not applied):\n  Group: ci-$(touch executed)\\u{1b}[31m\\u{202e}\nJob: b\n  Runner: linux\n  Concurrency (not applied):\n    Group: ${{ github.ref }} (unevaluated; expression syntax not validated)\n    Cancel in progress: true\n    Queue: max\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_concurrency_outside_the_inspected_subset() {
    let directory = tempfile::tempdir().unwrap();
    for (value, diagnostic) in [
        (
            "{group: ''}",
            "jobs.b.concurrency.group: expected a nonempty string",
        ),
        (
            "{group: ci, cancel-in-progress: 'true'}",
            "jobs.b.concurrency.cancel-in-progress: expected a boolean",
        ),
        (
            "{group: ci, queue: many}",
            "jobs.b.concurrency.queue: expected single or max",
        ),
        ("{queue: max}", "jobs.b.concurrency: missing field `group`"),
        (
            "{group: ci, \"b\\e\": 1}",
            "jobs.b.concurrency: unknown field `b\\u{1b}`, expected `group`, `cancel-in-progress` or `queue`",
        ),
        (
            "{group: 5}",
            "jobs.b.concurrency.group: non-string value is not inspected yet",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"), with_on(format!("jobs:\n  b:\n    runs-on: linux\n    concurrency: {value}\n    steps:\n      - run: echo\n")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{value}: {output:?}");
        assert!(output.stdout.is_empty(), "{value}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{value}"
        );
    }
}

// Backlog B-17: job and step `continue-on-error` and step `timeout-minutes`, never applied.
#[test]
fn inspect_reports_continue_on_error_and_step_timeouts_without_applying_them() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  b:\n    runs-on: linux\n    timeout-minutes: 5\n    continue-on-error: \"${{ $(touch executed)\\e\\u202E }}\"\n    steps:\n      - run: touch executed\n        continue-on-error: true\n        timeout-minutes: 1.5\n      - uses: ./action\n        timeout-minutes: ${{ inputs.minutes }}\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n  Timeout minutes: 5 (not applied)\n  Continue on error: ${{ $(touch executed)\\u{1b}\\u{202e} }} (unevaluated; expression syntax not validated; not applied)\n  Step 1: run\n    Continue on error: true (not applied)\n    Timeout minutes: 1.5 (not applied)\n  Step 2: uses ./action\n    Timeout minutes: ${{ inputs.minutes }} (unevaluated; expression syntax not validated; not applied)\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_non_boolean_continue_on_error_and_non_number_step_timeouts() {
    let directory = tempfile::tempdir().unwrap();
    for (field, diagnostic) in [
        (
            "    continue-on-error: 5\n    steps:\n      - run: echo\n",
            "jobs.b.continue-on-error: expected a boolean at line 3 column 5",
        ),
        (
            "    steps:\n      - run: echo\n        continue-on-error: 'true'\n",
            "jobs.b.steps[1].continue-on-error: expected a boolean at line 5 column 9",
        ),
        (
            "    steps:\n      - run: echo\n        timeout-minutes: true\n",
            "jobs.b.steps[1].timeout-minutes: expected a positive number at line 5 column 9",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"),
            with_on(format!("jobs:\n  b:\n    runs-on: linux\n{field}")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{field}: {output:?}");
        assert!(output.stdout.is_empty(), "{field}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{field}"
        );
    }
}

// Backlog B-18: job `outputs` are reported by name as source text, never resolved.
#[test]
fn inspect_reports_job_outputs_without_resolving_them() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  b:\n    runs-on: linux\n    outputs:\n      result: ${{ steps.s.outputs.r }}\n      \"n\\u202E\": \"$(touch executed)\\e[31m\"\n    steps:\n      - id: s\n        run: touch executed\n  c:\n    runs-on: linux\n    outputs: {}\n    steps:\n      - run: echo\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n  Outputs (not resolved):\n    n\\u{202e}: \"$(touch executed)\\u{1b}[31m\"\n    result: \"${{ steps.s.outputs.r }}\" (unevaluated)\n  Step 1 [s]: run\nJob: c\n  Runner: linux\n  Outputs (not resolved): {}\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_job_outputs_outside_the_inspected_subset() {
    let directory = tempfile::tempdir().unwrap();
    for (outputs, diagnostic) in [
        (
            "'${{ fromJSON(x) }}'",
            "jobs.b.outputs: expression is unevaluated and not allowed in job outputs",
        ),
        (
            "{a: x, A: y}",
            "jobs.b.outputs.A: duplicate key 'A'; already defined as 'a'",
        ),
        ("{a: [x]}", "jobs.b.outputs.a: expected a string"),
        (
            "{a: 5}",
            "jobs.b.outputs.a: non-string value is not inspected yet",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"), with_on(format!("jobs:\n  b:\n    runs-on: linux\n    outputs: {outputs}\n    steps:\n      - run: echo\n")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{outputs}: {output:?}");
        assert!(output.stdout.is_empty(), "{outputs}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{outputs}"
        );
    }
}

// Backlog B-19: job `environment` is reported as written, never selected or protected.
#[test]
fn inspect_reports_job_environments_without_selecting_them() {
    let directory = tempfile::tempdir().unwrap();
    let workflow = "jobs:\n  b:\n    runs-on: linux\n    environment: \"prod$(touch executed)\\e\\u202E\"\n    steps:\n      - run: touch executed\n  c:\n    runs-on: linux\n    environment:\n      name: ${{ inputs.target }}\n      url: https://example.test\n      deployment: false\n    outputs:\n      u: ${{ steps.d.outputs.url }}\n    steps:\n      - run: echo\n";
    fs::write(directory.path().join("ci.yml"), with_on(workflow)).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: b\n  Runner: linux\n  Environment (not selected; no protection rules evaluated):\n    Name: prod$(touch executed)\\u{1b}\\u{202e}\n  Step 1: run\nJob: c\n  Runner: linux\n  Environment (not selected; no protection rules evaluated):\n    Name: ${{ inputs.target }} (unevaluated; expression syntax not validated)\n    URL: https://example.test\n    Deployment: false\n  Outputs (not resolved):\n    u: \"${{ steps.d.outputs.url }}\" (unevaluated)\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_job_environments_outside_the_inspected_subset() {
    let directory = tempfile::tempdir().unwrap();
    for (environment, diagnostic) in [
        ("{url: x}", "jobs.b.environment: missing field `name`"),
        (
            "{name: a, \"b\\e\": 1}",
            "jobs.b.environment: unknown field `b\\u{1b}`, expected `name`, `url` or `deployment`",
        ),
        (
            "{name: a, deployment: 'false'}",
            "jobs.b.environment.deployment: expected a boolean",
        ),
        (
            "{name: ''}",
            "jobs.b.environment.name: expected a nonempty string",
        ),
        (
            "5",
            "jobs.b.environment: non-string value is not inspected yet",
        ),
    ] {
        fs::write(
            directory.path().join("ci.yml"), with_on(format!("jobs:\n  b:\n    runs-on: linux\n    environment: {environment}\n    steps:\n      - run: echo\n")),
        )
        .unwrap();
        let output = inspect(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(1), "{environment}: {output:?}");
        assert!(output.stdout.is_empty(), "{environment}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("error: wsr inspect: ci.yml: {diagnostic}\n"),
            "{environment}"
        );
    }
}

// Backlog B-25: the pinned root schema requires `on`.
#[test]
fn inspect_rejects_a_workflow_without_on() {
    let directory = tempfile::tempdir().unwrap();
    let source = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: touch executed\n";
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: wsr inspect: ci.yml: missing field `on`\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
}

// B-33: workflow `description` is retained as written and escaped.
#[test]
fn inspect_reports_workflow_descriptions_as_written() {
    let directory = tempfile::tempdir().unwrap();
    let source = "name: ci\ndescription: \"$(touch executed)\\e[31m\\u202E\"\non: workflow_dispatch\njobs:\n  a:\n    runs-on: linux\n    steps: [{run: touch executed}]\n";
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect_with_execution_traps(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: ci\nDescription: $(touch executed)\\u{1b}[31m\\u{202e}\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner: linux\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    fs::write(directory.path().join("ci.yml"), "description: '${{ x }}'\non: workflow_dispatch\njobs:\n  a:\n    runs-on: linux\n    steps: [{run: echo}]\n").unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: wsr inspect: ci.yml: description: expression is unevaluated; dynamic inspection is not supported yet\n"
    );
}

// B-35: runner selectors are reported as written; no runner is selected or contacted.
#[test]
fn inspect_reports_runner_selectors_without_selecting_a_runner() {
    let directory = tempfile::tempdir().unwrap();
    let source = "on: workflow_dispatch\njobs:\n  a:\n    runs-on: {group: \"$(touch executed)\\e[31m\\u202E\", labels: [\"${{ matrix.os }}\", linux]}\n    steps: [{run: touch executed}]\n  b:\n    runs-on: {group: '${{ vars.group }}', labels: gpu}\n    steps: [{run: touch executed}]\n  c:\n    runs-on: {labels: []}\n    steps: [{run: touch executed}]\n";
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect_with_execution_traps(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner group: $(touch executed)\\u{1b}[31m\\u{202e} (not selected)\n  Runner labels: [\"${{ matrix.os }}\", \"linux\"] (some unevaluated; expression syntax not validated)\n  Step 1: run\nJob: b\n  Runner group: ${{ vars.group }} (unevaluated; expression syntax not validated; not selected)\n  Runner: gpu\n  Step 1: run\nJob: c\n  Runner labels: []\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_reports_job_strategies_without_expanding_or_executing() {
    let directory = tempfile::tempdir().unwrap();
    let source = "on: workflow_dispatch\njobs:\n  a:\n    runs-on: linux\n    strategy:\n      fail-fast: false\n      max-parallel: 2\n      matrix:\n        os: [\"$(touch executed)\\e[31m\\u202E\"]\n    steps: [{run: touch executed}]\n";
    fs::write(directory.path().join("ci.yml"), source).unwrap();
    let output = inspect_with_execution_traps(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner: linux\n  Strategy (not expanded):\n    Fail-fast: false\n    Max-parallel: 2\n    Matrix (normalized YAML): os:\\n- \\\"$(touch executed)\\\\e[31m\\u{202e}\\\"\\n\n  Step 1: run\n"
    );
    assert!(!directory.path().join("executed").exists());
    assert_eq!(
        fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
        source
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn inspect_rejects_matrix_keys_duplicated_ignoring_case() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("ci.yml"), with_on("jobs:\n  a:\n    runs-on: linux\n    strategy: {matrix: {a: [1], A: [2]}}\n    steps: [{run: echo}]\n")).unwrap();
    let output = inspect(directory.path(), "ci.yml");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: wsr inspect: ci.yml: jobs.a.strategy.matrix.A: duplicate key 'A'; already defined as 'a'\n"
    );
}

// Literal matrices inspect at any size, with only the D-032 warning (B-32).
#[test]
fn inspect_reports_large_literal_matrices_without_expanding_them() {
    let directory = tempfile::tempdir().unwrap();
    let zeros = vec!["0"; 64_000].join(",");
    let large = "x".repeat(512_001);
    for (matrix, warned) in [
        (format!("{{a: [[{zeros}]]}}"), false),
        (format!("{{a: ['{large}']}}"), true),
    ] {
        let source = format!(
            "on: workflow_dispatch\njobs:\n  a:\n    runs-on: linux\n    strategy:\n      matrix: {matrix}\n    steps: [{{run: touch executed}}]\n"
        );
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect_with_execution_traps(directory.path(), "ci.yml");
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let warning = if warned {
            format!(
                "warning: wsr inspect: ci.yml: {} bytes exceeds GitHub's documented 500 KB (512000-byte) workflow file limit; GitHub would not run this workflow\n",
                source.len()
            )
        } else {
            String::new()
        };
        assert_eq!(String::from_utf8(output.stderr).unwrap(), warning);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.matches("Job: ").count(), 1);
        assert_eq!(stdout.matches("Matrix (normalized YAML): ").count(), 1);
        assert!(
            stdout.ends_with("\n  Step 1: run\n"),
            "{}",
            &stdout[stdout.len() - 200..]
        );
        assert!(!directory.path().join("executed").exists());
    }
}

fn inspect_with_execution_traps(directory: &Path, file: &str) -> Output {
    let bin = tempfile::tempdir().unwrap();
    let marker = directory.join("executed");
    #[cfg(unix)]
    for command in ["sh", "bash", "python", "python3", "git", "pwsh"] {
        use std::os::unix::fs::PermissionsExt;
        let path = bin.path().join(command);
        fs::write(
            &path,
            "#!/bin/sh\n/usr/bin/touch \"$WSR_EXECUTION_MARKER\"\n",
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        // Positive control: each trap leaves the marker when it is run.
        let status = Command::new(&path)
            .env("WSR_EXECUTION_MARKER", &marker)
            .status()
            .unwrap();
        assert!(status.success() && marker.exists(), "{command} trap");
        fs::remove_file(&marker).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(["inspect", file])
        .current_dir(directory)
        .env("PATH", bin.path())
        .env("WSR_EXECUTION_MARKER", &marker)
        .output()
        .expect("run inspection");
    assert!(!marker.exists(), "a command was executed");
    output
}

#[test]
fn inspect_reports_cancel_timeouts_without_applying_or_executing() {
    for (value, line) in [
        ("5", "5 (not applied)"),
        (
            "\"${{ $(touch executed)\\e[31m\\u202E }}\"",
            "${{ $(touch executed)\\u{1b}[31m\\u{202e} }} (unevaluated; expression syntax not validated; not applied)",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let source = format!(
            "on: workflow_dispatch\njobs:\n  a:\n    runs-on: linux\n    timeout-minutes: 10\n    cancel-timeout-minutes: {value}\n    steps: [{{run: touch executed}}]\n"
        );
        fs::write(directory.path().join("ci.yml"), &source).unwrap();
        let output = inspect_with_execution_traps(directory.path(), "ci.yml");
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "Workflow: (unnamed)\nInspection only; execution compatibility is not assessed.\nTrigger: workflow_dispatch (not evaluated)\nJob: a\n  Runner: linux\n  Timeout minutes: 10 (not applied)\n  Cancel timeout minutes: {line}\n  Step 1: run\n"
            )
        );
        assert!(!directory.path().join("executed").exists());
        assert_eq!(
            fs::read_to_string(directory.path().join("ci.yml")).unwrap(),
            source
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
