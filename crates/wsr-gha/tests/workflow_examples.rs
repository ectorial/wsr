//! Example input contracts and native shell references, not GHA-to-IR or WSR execution tests.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_yaml::Value;

fn workflow_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows")
}

#[test]
fn dogfood_workflows_only_allow_manual_dispatch() {
    let mut examples = 0;
    for entry in fs::read_dir(workflow_directory()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        if !name.starts_with("example-") || !name.ends_with(".yml") && !name.ends_with(".yaml") {
            continue;
        }
        examples += 1;
        let workflow: Value = serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let triggers = workflow["on"]
            .as_mapping()
            .expect("explicit trigger mapping");
        assert_eq!(triggers.len(), 1, "{name}: only manual dispatch is allowed");
        assert!(
            triggers.contains_key(Value::from("workflow_dispatch")),
            "{name}: workflow_dispatch is required"
        );
        assert!(
            !workflow["jobs"].as_mapping().unwrap().is_empty(),
            "{name}: an example needs runnable jobs"
        );
    }
    assert!(examples > 0, "no dogfood workflows found");
}

#[test]
fn shell_example_reports_success_and_controlled_failure_natively() {
    let workflow: Value = serde_yaml::from_str(
        &fs::read_to_string(workflow_directory().join("example-shell.yml")).unwrap(),
    )
    .unwrap();
    let step = &workflow["jobs"]["shell"]["steps"][0];
    assert_eq!(step["shell"].as_str(), Some("sh"));
    assert_eq!(
        step["env"]["WSR_EXAMPLE_MODE"].as_str(),
        Some("${{ inputs.mode }}")
    );
    let script = step["run"].as_str().unwrap();
    for (mode, exit, marker) in [
        ("pass", 0, "WSR shell example passed"),
        ("fail", 1, "controlled WSR shell failure"),
    ] {
        // Bind the native reference input explicitly; this does not evaluate GHA expressions.
        let output = Command::new("sh")
            .args(["-e", "-c", script])
            .env_clear()
            .env("WSR_EXAMPLE_MODE", mode)
            .output()
            .expect("native reference requires a POSIX shell");
        assert_eq!(output.status.code(), Some(exit), "{mode}: {output:?}");
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(log.contains(marker), "{mode}: {log}");
    }
}
