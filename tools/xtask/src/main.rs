//! Development and scaffolding automation for this workspace.

mod cli;
mod process;
mod release;
mod scaffold;
mod tasks;
mod tooling;
mod tools;
mod workspace;

use clap::Parser;
use cli::{Cli, Command};
use std::process::ExitCode;

type Result<T = ()> = std::result::Result<T, String>;

fn run(cli: Cli) -> Result {
    let workspace = workspace::Workspace::discover()?;
    match cli.command {
        Command::Check => tasks::check(&workspace),
        Command::Test(args) => tasks::test(&workspace, &args),
        Command::Build => tasks::build(&workspace),
        Command::Ci(args) => tasks::ci(&workspace, args.full),
        Command::Coverage => tasks::coverage(&workspace),
        Command::Scaffold { command, dry_run } => scaffold::run(&workspace, command, dry_run),
        Command::Doctor => {
            tools::doctor(&workspace);
            Ok(())
        }
        Command::Tools { command } => tools::run(&workspace, &command),
        Command::Release { command } => release::run(&workspace, command),
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
