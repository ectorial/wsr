use wsr_cli::HookCmd;

pub fn run(action: HookCmd) -> anyhow::Result<()> {
    let _ = action;
    anyhow::bail!("wsr hook — not yet implemented")
}
