use crate::Result;
use crate::cli::{CiArgs, CiJob, TestArgs};
use crate::process;
use crate::tools::{self, Tool};
use crate::workspace::Workspace;

pub(crate) fn check(workspace: &Workspace) -> Result {
    format(workspace)?;
    lint(workspace)
}

fn format(workspace: &Workspace) -> Result {
    cargo(workspace, ["fmt", "--all", "--check"])
}

fn lint(workspace: &Workspace) -> Result {
    cargo(
        workspace,
        [
            "check",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
        ],
    )?;
    cargo(
        workspace,
        [
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )
}

pub(crate) fn test(workspace: &Workspace, args: &TestArgs) -> Result {
    check(workspace)?;
    tests(workspace, args)
}

fn tests(workspace: &Workspace, args: &TestArgs) -> Result {
    if args.nextest {
        let mut nextest = vec!["run", "--workspace", "--locked"];
        if let Some(filter) = &args.filter {
            nextest.push(filter);
        }
        tools::execute(workspace, Tool::Nextest, nextest)?;
        cargo(workspace, ["test", "--workspace", "--doc", "--locked"])
    } else {
        let mut test = vec!["test", "--workspace", "--locked"];
        if let Some(filter) = &args.filter {
            test.push(filter);
        }
        cargo(workspace, test)
    }
}

pub(crate) fn build(workspace: &Workspace) -> Result {
    test(
        workspace,
        &TestArgs {
            filter: None,
            nextest: false,
        },
    )?;
    cargo(workspace, ["build", "--workspace", "--release", "--locked"])
}

pub(crate) fn ci(workspace: &Workspace, args: &CiArgs) -> Result {
    if let Some(job) = args.job {
        return match job {
            CiJob::Format => format(workspace),
            CiJob::Lint => lint(workspace),
            CiJob::Docs => process::rustdoc(workspace.root()),
            CiJob::Test => tests(
                workspace,
                &TestArgs {
                    filter: None,
                    nextest: false,
                },
            ),
        };
    }
    test(
        workspace,
        &TestArgs {
            filter: None,
            nextest: false,
        },
    )?;
    process::rustdoc(workspace.root())?;
    if args.full {
        tools::execute(workspace, Tool::Deny, ["check"])?;
        tools::execute(workspace, Tool::Typos, ["."])?;
    }
    Ok(())
}

pub(crate) fn coverage(workspace: &Workspace) -> Result {
    tools::execute(workspace, Tool::Coverage, ["--workspace", "--html"])
}

fn cargo<I, S>(workspace: &Workspace, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    process::run(workspace.root(), "cargo", args)
}
