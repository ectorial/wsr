use clap::Parser;
use wsr_cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    wsr::commands::run(cli)
}
