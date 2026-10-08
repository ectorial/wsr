use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const LEAVES: &[(&[&str], &str)] = &[
    (&["init"], "init"),
    (&["run"], "run"),
    (&["daemon"], "daemon"),
    (&["list"], "list"),
    (&["inspect", "workflow.yml"], "inspect"),
    (&["cache", "list"], "cache list"),
    (&["cache", "verify"], "cache verify"),
    (&["cache", "purge"], "cache purge"),
    (&["hook", "install", "pre-push"], "hook install"),
    (&["hook", "remove", "pre-push"], "hook remove"),
    (&["status"], "status"),
];

fn invoke(args: &[&str], directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wsr"))
        .args(args)
        .current_dir(directory)
        .output()
        .expect("run wsr")
}

fn assert_placeholder(output: &Output, command: &str) {
    assert_eq!(output.status.code(), Some(1), "{command}: {output:?}");
    assert!(output.stdout.is_empty(), "{command}: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("error: wsr {command}: not implemented yet\n")
    );
}

#[test]
fn smoke_every_operational_subcommand_has_the_same_placeholder_contract() {
    let directory = tempfile::tempdir().unwrap();
    for (args, command) in LEAVES {
        assert_placeholder(&invoke(args, directory.path()), command);
    }
}

#[test]
fn valid_options_and_nonexistent_paths_still_reach_placeholders() {
    let directory = tempfile::tempdir().unwrap();
    for (args, command) in [
        (&["run", "missing.yml"][..], "run"),
        (&["inspect", "missing.yml"][..], "inspect"),
        (&["run", "--dry-run", "--yes", "--format", "gha"][..], "run"),
        (
            &[
                "run",
                "workflow.yml",
                "--event",
                "push",
                "--verbose",
                "--format",
                "human",
            ][..],
            "run",
        ),
        (&["daemon", "--install"][..], "daemon"),
    ] {
        assert_placeholder(&invoke(args, directory.path()), command);
    }
}

#[test]
fn help_and_version_succeed_without_operational_output() {
    let directory = tempfile::tempdir().unwrap();
    let root = invoke(&["--help"], directory.path());
    assert_eq!(root.status.code(), Some(0));
    assert!(root.stderr.is_empty());
    let help = String::from_utf8(root.stdout).unwrap();
    assert!(help.contains("A pre-alpha local CI CLI scaffold"));
    for name in [
        "init", "run", "daemon", "list", "inspect", "cache", "hook", "status",
    ] {
        assert!(help.contains(name), "missing {name} in root help");
    }

    let mut paths: Vec<Vec<&str>> = LEAVES
        .iter()
        .map(|(_, path)| path.split(' ').collect())
        .collect();
    paths.extend([vec!["cache"], vec!["hook"]]);
    for mut args in paths {
        args.push("--help");
        let output = invoke(&args, directory.path());
        assert_eq!(output.status.code(), Some(0), "{args:?}: {output:?}");
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("Usage:"), "{args:?}");
        assert!(help.contains("not implemented yet"), "{args:?}");
        if args == ["run", "--help"] {
            for flag in ["--event", "--dry-run", "--verbose", "--yes", "--format"] {
                assert!(help.contains(flag), "missing {flag} in run help");
            }
            assert!(help.contains("human") && help.contains("gha"));
        }
    }

    let version = invoke(&["--version"], directory.path());
    assert_eq!(version.status.code(), Some(0));
    assert!(version.stderr.is_empty());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!("wsr {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_arguments_and_incomplete_namespaces_exit_two() {
    let directory = tempfile::tempdir().unwrap();
    for args in [
        &[][..],
        &["cache"][..],
        &["hook"][..],
        &["unknown"][..],
        &["run", "--unknown"][..],
        &["inspect"][..],
        &["hook", "install"][..],
        &["hook", "remove"][..],
        &["cache", "unknown"][..],
        &["run", "--format", "json"][..],
        &["run", "--event"][..],
        &["run", "--format"][..],
        &["list", "--yes"][..],
        &["status", "--verbose"][..],
    ] {
        let output = invoke(args, directory.path());
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        assert!(!output.stderr.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).starts_with("error: wsr "));
    }
}

// Include directories as well as file bytes so additions, deletions, and content changes fail.
fn snapshot(root: &Path, directory: &Path, entries: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if path.is_dir() {
            entries.insert(relative, None);
            snapshot(root, &path, entries);
        } else {
            entries.insert(relative, Some(fs::read(path).unwrap()));
        }
    }
}

#[test]
fn operational_commands_leave_repository_sentinels_untouched() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for (file, contents) in [
        ("workflow.yml", "invalid workflow sentinel\n"),
        (".github/workflows/ci.yml", "workflow sentinel\n"),
        (".git/hooks/pre-push", "hook sentinel\n"),
        (".wsr/cache/entry", "cache sentinel\n"),
        ("wsr.json", "invalid configuration sentinel\n"),
    ] {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    let mut before = BTreeMap::new();
    snapshot(root, root, &mut before);
    let mut cases = LEAVES.to_vec();
    cases.extend([
        (
            &[
                "run",
                "workflow.yml",
                "--dry-run",
                "--yes",
                "--format",
                "gha",
            ][..],
            "run",
        ),
        (&["daemon", "--install"][..], "daemon"),
    ]);
    for (args, command) in cases {
        assert_placeholder(&invoke(args, root), command);
        let mut after = BTreeMap::new();
        snapshot(root, root, &mut after);
        assert_eq!(before, after, "files changed by {args:?}");
    }
}
