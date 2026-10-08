use clap::Subcommand;

/// Subcommands for `wsr hook`.
#[derive(Subcommand, Debug)]
pub enum HookCmd {
    /// Install a git hook (not implemented yet)
    Install {
        /// Git hook name (e.g. pre-push, pre-commit)
        hook: String,
    },
    /// Remove a git hook (not implemented yet)
    Remove {
        /// Git hook name
        hook: String,
    },
}
