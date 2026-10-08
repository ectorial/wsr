use wsr_cli::HookCmd;

pub fn run(action: HookCmd) -> anyhow::Result<()> {
    let command = match action {
        HookCmd::Install { .. } => "hook install",
        HookCmd::Remove { .. } => "hook remove",
    };
    super::not_implemented(command)
}
