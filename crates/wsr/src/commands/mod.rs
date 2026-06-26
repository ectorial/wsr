use wsr_cli::{Cli, Cmd};

pub mod cache;
pub mod daemon;
pub mod hook;
pub mod init;
pub mod inspect;
pub mod list;
pub mod run;
pub mod status;

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Cmd::Init => init::run(),
        Cmd::Run(args) => run::run(args),
        Cmd::Daemon(args) => daemon::run(args),
        Cmd::List => list::run(),
        Cmd::Inspect { file } => inspect::run(&file),
        Cmd::Cache { action } => cache::run(action),
        Cmd::Hook { action } => hook::run(action),
        Cmd::Status => status::run(),
    }
}
