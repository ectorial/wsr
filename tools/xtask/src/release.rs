use crate::Result;
use crate::cli::ReleaseCommand;
use crate::scaffold;
use crate::tools::{self, Tool};
use crate::workspace::Workspace;

pub(crate) fn run(workspace: &Workspace, command: ReleaseCommand) -> Result {
    match command {
        ReleaseCommand::Init => {
            let has_binary = workspace.has_publishable_binary();
            if has_binary {
                tools::ensure(workspace, Tool::Dist)?;
            }
            scaffold::release_files(workspace)?;
            if has_binary {
                tools::execute(workspace, Tool::Dist, ["init", "--yes"])
            } else {
                println!(
                    "cargo-dist skipped: no publishable binary; scaffold one and rerun release init"
                );
                Ok(())
            }
        }
        ReleaseCommand::Plan { tag } => {
            if !workspace.has_publishable_binary() {
                return Err(
                    "cargo-dist needs a publishable binary; run `cargo xtask scaffold cli --entrypoint primary` or scaffold a publishable binary crate first"
                        .into(),
                );
            }
            let mut args = vec!["plan"];
            let tag_argument;
            if let Some(tag) = tag {
                tag_argument = format!("--tag={tag}");
                args.push(&tag_argument);
            }
            tools::execute(workspace, Tool::Dist, args)
        }
        ReleaseCommand::Prepare { level, execute } => {
            tools::ensure(workspace, Tool::Release)?;
            tools::ensure(workspace, Tool::Cliff)?;
            let mut args = vec![level.as_str(), "--workspace", "--no-publish", "--no-push"];
            if execute {
                args.push("--execute");
            }
            tools::execute(workspace, Tool::Release, args)
        }
        ReleaseCommand::Publish { execute } => {
            tools::ensure(workspace, Tool::Release)?;
            let mut args = vec![
                "publish",
                "--workspace",
                "--exclude",
                "xtask",
                "--exclude",
                "wsr-bench",
            ];
            if execute {
                args.push("--execute");
            }
            tools::execute(workspace, Tool::Release, args)
        }
        ReleaseCommand::Changelog { tag } => {
            if std::env::var("DRY_RUN").as_deref() == Ok("true") {
                println!("changelog generation skipped during cargo-release dry run");
                Ok(())
            } else {
                tools::ensure(workspace, Tool::Cliff)?;
                tools::execute(
                    workspace,
                    Tool::Cliff,
                    ["-o", "CHANGELOG.md", "--tag", &tag],
                )
            }
        }
    }
}
