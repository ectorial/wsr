use clap::Subcommand;

/// Subcommands for `wsr cache`.
#[derive(Subcommand, Debug)]
pub enum CacheCmd {
    /// List cache entries (not implemented yet)
    List,
    /// Verify cache entries (not implemented yet)
    Verify,
    /// Remove cache entries (not implemented yet)
    Purge,
}
