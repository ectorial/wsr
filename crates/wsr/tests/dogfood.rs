//! Static inspection acceptance on the real dogfood `.github/workflows/example-*.yml` files.
//! Inspection success is not WSR execution support or GitHub Actions compatibility.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HEAD: &str = "Inspection only; execution compatibility is not assessed.
Trigger: workflow_dispatch (not evaluated)
  Input: mode
    Type: choice
    Description: fail intentionally produces a failed workflow run
    Required: true
    Default: \"pass\" (not applied)
    Options: [\"pass\", \"fail\"]
Permissions (requested; not granted):
  contents: read
";

const PYTHON: &str = "Job: python
  Runner: ubuntu-latest
  Timeout minutes: 5 (not applied)
  Step 1: uses actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
    With (not passed to any action):
      persist-credentials: false
  Step 2: uses actions/setup-python@6d4fd22b20465ccc0f7354476588e8df7c9ca0e0
    With (not passed to any action):
      python-version: \"3.13\"
  Step 3: run (Run example tests directly (native reference))
    Working directory: examples/python-basic (not resolved)
    Env (not bound):
      WSR_EXAMPLE_MODE: \"${{ inputs.mode }}\" (unevaluated)
";

const SHELL: &str = "Job: shell
  Runner: ubuntu-latest
  Timeout minutes: 5 (not applied)
  Step 1: run (Controlled shell outcome)
    Shell: sh (not selected)
    Env (not bound):
      WSR_EXAMPLE_MODE: \"${{ inputs.mode }}\" (unevaluated)
";

// Golden table: (file, workflow name, run-name prefix, job report).
const CASES: [(&str, &str, &str, &str); 2] = [
    (
        "example-python.yml",
        "Example Python reference",
        "Python reference",
        PYTHON,
    ),
    (
        "example-shell.yml",
        "Example shell reference",
        "Shell reference",
        SHELL,
    ),
];

fn workflows() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows")
}

fn example(name: &str) -> PathBuf {
    workflows().join(name)
}

fn example_names(directory: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_file())
        // A non-UTF-8 name cannot match `example-*.yml`; skip it rather than panic.
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("example-") && name.ends_with(".yml"))
        .collect();
    names.sort();
    names
}

fn golden_names() -> Vec<String> {
    let mut names: Vec<_> = CASES.iter().map(|case| case.0.to_owned()).collect();
    names.sort();
    names
}

// Backlog B-10: a new dogfood example cannot land without a golden case.
#[test]
fn every_dogfood_example_has_a_golden_case() {
    assert_eq!(example_names(&workflows()), golden_names());
}

// The guard sees an example added beside the real ones (in a temporary copy, not the repo).
#[test]
fn dogfood_guard_detects_an_example_without_a_golden_case() {
    let directory = tempfile::tempdir().unwrap();
    for entry in fs::read_dir(workflows()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, directory.path().join(path.file_name().unwrap())).unwrap();
        }
    }
    assert_eq!(example_names(directory.path()), golden_names());
    fs::write(directory.path().join("example-extra.yml"), "on: push\n").unwrap();
    assert_ne!(example_names(directory.path()), golden_names());
}

fn snapshot(directory: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    entries.sort();
    entries
}

#[test]
fn dogfood_examples_inspect_with_expected_output_and_execute_nothing() {
    for (name, title, run_name, job) in CASES {
        let expected = format!(
            "Workflow: {title}\nRun name: {run_name} — ${{{{ inputs.mode }}}} (unevaluated; expression syntax not validated)\n{HEAD}{job}"
        );
        let source = fs::read(example(name)).unwrap();

        // A copy beside sentinels. On Unix, every PATH command is replaced by one that leaves a
        // marker; the trap evidence is Unix-only, the golden assertions are not.
        let directory = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let marker = directory.path().join("executed");
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
        fs::write(directory.path().join(name), &source).unwrap();
        fs::write(directory.path().join("wsr.toml"), "invalid config sentinel").unwrap();
        let before = snapshot(directory.path());

        for file in [directory.path().join(name), example(name)] {
            let output = Command::new(env!("CARGO_BIN_EXE_wsr"))
                .arg("inspect")
                .arg(&file)
                .current_dir(directory.path())
                .env("PATH", bin.path())
                .env("WSR_EXECUTION_MARKER", &marker)
                .output()
                .expect("run inspection");
            assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
            assert!(output.stderr.is_empty(), "{name}: {output:?}");
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{}",
                file.display()
            );
        }
        assert!(!marker.exists(), "{name}: a command was executed");
        assert_eq!(snapshot(directory.path()), before, "{name}");
        assert_eq!(fs::read(example(name)).unwrap(), source, "{name}");
    }
}
