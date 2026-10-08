use wsr_cli::CacheCmd;

pub fn run(action: CacheCmd) -> anyhow::Result<()> {
    let command = match action {
        CacheCmd::List => "cache list",
        CacheCmd::Verify => "cache verify",
        CacheCmd::Purge => "cache purge",
    };
    super::not_implemented(command)
}
