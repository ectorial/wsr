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

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/workflows")
        .join(name)
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
    for (name, title, run_name, job) in [
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
    ] {
        let expected = format!(
            "Workflow: {title}\nRun name: {run_name} — ${{{{ inputs.mode }}}} (unevaluated; expression syntax not validated)\n{HEAD}{job}"
        );
        let source = fs::read(example(name)).unwrap();

        // A copy beside sentinels, with every PATH command replaced by one that leaves a marker.
        let directory = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let marker = directory.path().join("executed");
        for command in ["sh", "bash", "python", "python3", "git", "pwsh"] {
            let path = bin.path().join(command);
            fs::write(
                &path,
                format!("#!/bin/sh\n/usr/bin/touch '{}'\n", marker.display()),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            }
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
