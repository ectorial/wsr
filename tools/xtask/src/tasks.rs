use crate::Result;
use crate::cli::TestArgs;
use crate::process;
use crate::tools::{self, Tool};
use crate::workspace::Workspace;

pub(crate) fn check(workspace: &Workspace) -> Result {
    cargo(workspace, ["fmt", "--all", "--check"])?;
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

pub(crate) fn ci(workspace: &Workspace, full: bool) -> Result {
    test(
        workspace,
        &TestArgs {
            filter: None,
            nextest: false,
        },
    )?;
    process::rustdoc(workspace.root())?;
    if full {
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
