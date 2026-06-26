use wsr_cli::CacheCmd;

pub fn run(action: CacheCmd) -> anyhow::Result<()> {
    let _ = action;
    anyhow::bail!("wsr cache — not yet implemented")
}
