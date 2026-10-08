use std::path::PathBuf;

use clap::{CommandFactory, Parser, error::ErrorKind};
use wsr_cli::{CacheCmd, Cli, Cmd, HookCmd, OutputFormat};

#[test]
fn command_graph_is_valid() {
    Cli::command().debug_assert();
}

#[test]
fn parses_existing_command_tree() {
    assert!(matches!(
        Cli::parse_from(["wsr", "init"]).command,
        Cmd::Init
    ));
    assert!(matches!(
        Cli::parse_from(["wsr", "list"]).command,
        Cmd::List
    ));
    assert!(matches!(
        Cli::parse_from(["wsr", "status"]).command,
        Cmd::Status
    ));

    let Cmd::Inspect { file } = Cli::parse_from(["wsr", "inspect", "workflow.yml"]).command else {
        panic!("expected inspect");
    };
    assert_eq!(file, PathBuf::from("workflow.yml"));

    let Cmd::Daemon(args) = Cli::parse_from(["wsr", "daemon", "--install"]).command else {
        panic!("expected daemon");
    };
    assert!(args.install);
    let Cmd::Daemon(args) = Cli::parse_from(["wsr", "daemon"]).command else {
        panic!("expected daemon");
    };
    assert!(!args.install);

    for (name, expected) in [
        ("list", CacheCmd::List),
        ("verify", CacheCmd::Verify),
        ("purge", CacheCmd::Purge),
    ] {
        let Cmd::Cache { action } = Cli::parse_from(["wsr", "cache", name]).command else {
            panic!("expected cache");
        };
        assert_eq!(
            std::mem::discriminant(&action),
            std::mem::discriminant(&expected)
        );
    }

    let Cmd::Hook {
        action: HookCmd::Install { hook },
    } = Cli::parse_from(["wsr", "hook", "install", "pre-push"]).command
    else {
        panic!("expected hook install");
    };
    assert_eq!(hook, "pre-push");
    let Cmd::Hook {
        action: HookCmd::Remove { hook },
    } = Cli::parse_from(["wsr", "hook", "remove", "custom-hook"]).command
    else {
        panic!("expected hook remove");
    };
    assert_eq!(hook, "custom-hook");
}

#[test]
fn run_options_are_typed_and_preserved() {
    let Cmd::Run(args) = Cli::parse_from([
        "wsr",
        "run",
        "workflow.yml",
        "--event",
        "custom_event",
        "--dry-run",
        "--verbose",
        "--yes",
        "--format",
        "gha",
    ])
    .command
    else {
        panic!("expected run");
    };
    assert_eq!(args.file, Some(PathBuf::from("workflow.yml")));
    assert_eq!(args.event.as_deref(), Some("custom_event"));
    assert!(args.dry_run && args.verbose && args.yes);
    assert_eq!(args.format, OutputFormat::Gha);
}

#[test]
fn run_defaults_and_format_values() {
    let Cmd::Run(args) = Cli::parse_from(["wsr", "run"]).command else {
        panic!("expected run");
    };
    assert!(args.file.is_none() && args.event.is_none());
    assert!(!args.dry_run && !args.verbose && !args.yes);
    assert_eq!(args.format, OutputFormat::Human);

    for (value, expected) in [("human", OutputFormat::Human), ("gha", OutputFormat::Gha)] {
        let Cmd::Run(args) = Cli::parse_from(["wsr", "run", "--format", value]).command else {
            panic!("expected run");
        };
        assert_eq!(args.format, expected);
    }
    assert_eq!(
        Cli::try_parse_from(["wsr", "run", "--format", "json"])
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidValue
    );
}

#[test]
fn rejects_missing_inputs_and_run_options_on_other_commands() {
    for args in [
        vec!["wsr", "inspect"],
        vec!["wsr", "hook", "install"],
        vec!["wsr", "hook", "remove"],
        vec!["wsr", "run", "--event"],
        vec!["wsr", "run", "--format"],
        vec!["wsr", "list", "--yes"],
        vec!["wsr", "status", "--verbose"],
    ] {
        assert!(Cli::try_parse_from(args.clone()).is_err(), "{args:?}");
    }
}

#[cfg(unix)]
#[test]
fn workflow_paths_preserve_non_utf8_os_strings() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let path = OsString::from_vec(b"workflow-\xff.yml".to_vec());
    let Cmd::Inspect { file } = Cli::parse_from([
        OsString::from("wsr"),
        OsString::from("inspect"),
        path.clone(),
    ])
    .command
    else {
        panic!("expected inspect");
    };
    assert_eq!(file.into_os_string(), path);
}
