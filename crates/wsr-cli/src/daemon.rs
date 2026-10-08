use clap::Args;

/// Arguments for `wsr daemon`.
#[derive(Args, Debug)]
pub struct DaemonArgs {
    /// Request service registration (not implemented yet)
    #[arg(long)]
    pub install: bool,
}
