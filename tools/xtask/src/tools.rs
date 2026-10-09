use std::ffi::OsString;
use std::path::PathBuf;

use crate::Result;
use crate::cli::{ToolGroup, ToolsCommand};
use crate::process;
use crate::workspace::Workspace;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Tool {
    Nextest,
    Coverage,
    Deny,
    Typos,
    Dist,
    Release,
    Cliff,
}

const ALL: &[Tool] = &[
    Tool::Nextest,
    Tool::Coverage,
    Tool::Deny,
    Tool::Typos,
    Tool::Dist,
    Tool::Release,
    Tool::Cliff,
];
const CI: &[Tool] = &[Tool::Deny, Tool::Typos];
const COVERAGE: &[Tool] = &[Tool::Coverage];
const RELEASE: &[Tool] = &[Tool::Dist, Tool::Release, Tool::Cliff];
const TEST: &[Tool] = &[Tool::Nextest];

impl Tool {
    const fn package(self) -> &'static str {
        match self {
            Self::Nextest => "cargo-nextest",
            Self::Coverage => "cargo-llvm-cov",
            Self::Deny => "cargo-deny",
            Self::Typos => "typos-cli",
            Self::Dist => "cargo-dist",
            Self::Release => "cargo-release",
            Self::Cliff => "git-cliff",
        }
    }

    const fn executable(self) -> &'static str {
        match self {
            Self::Nextest => "cargo-nextest",
            Self::Coverage => "cargo-llvm-cov",
            Self::Deny => "cargo-deny",
            Self::Typos => "typos",
            Self::Dist => "dist",
            Self::Release => "cargo-release",
            Self::Cliff => "git-cliff",
        }
    }

    const fn prefix(self) -> &'static [&'static str] {
        match self {
            Self::Nextest => &["nextest"],
            Self::Coverage => &["llvm-cov"],
            Self::Release => &["release"],
            _ => &[],
        }
    }
}

pub(crate) fn run(workspace: &Workspace, command: &ToolsCommand) -> Result {
    match command {
        ToolsCommand::Sync { group } => sync(workspace, *group),
    }
}

pub(crate) fn doctor(workspace: &Workspace) {
    println!("built-in:");
    for (name, args) in [("rustfmt", &["fmt"][..]), ("clippy", &["clippy"][..])] {
        let available = process::available("cargo".as_ref(), args);
        println!("  {name:<18} {}", state(available));
    }
    println!("optional:");
    for tool in ALL {
        let location = resolve_optional(workspace, *tool);
        let detail = location
            .as_ref()
            .map_or_else(|| "missing".to_owned(), |path| path.display().to_string());
        println!(
            "  {:<18} {:<8} {}",
            tool.package(),
            state(location.is_some()),
            detail
        );
    }
    println!("\nInstall optional tools with `cargo xtask tools sync <group>`.");
}

pub(crate) fn execute<I, S>(workspace: &Workspace, tool: Tool, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let program = resolve_optional(workspace, tool).ok_or_else(|| {
        format!(
            "{} is missing or incompatible; run `cargo xtask tools sync {}`",
            tool.package(),
            group_for(tool)
        )
    })?;
    let args: Vec<OsString> = tool
        .prefix()
        .iter()
        .map(OsString::from)
        .chain(
            args.into_iter()
                .map(|argument| argument.as_ref().to_owned()),
        )
        .collect();
    process::run_with_path(workspace.root(), program, args, Some(&local_bin(workspace)))
}

pub(crate) fn ensure(workspace: &Workspace, tool: Tool) -> Result {
    resolve_optional(workspace, tool).map_or_else(
        || {
            Err(format!(
                "{} is missing or incompatible; run `cargo xtask tools sync {}`",
                tool.package(),
                group_for(tool)
            ))
        },
        |_| Ok(()),
    )
}

fn sync(workspace: &Workspace, group: ToolGroup) -> Result {
    process::run(workspace.root(), "rustup", ["update", "stable"])?;
    process::run(
        workspace.root(),
        "rustup",
        [
            "component",
            "add",
            "--toolchain",
            "stable",
            "clippy",
            "rustfmt",
        ],
    )?;
    std::fs::create_dir_all(local_root(workspace))
        .map_err(|error| format!("failed to create project tool directory: {error}"))?;
    for tool in tools(group) {
        if matches!(tool, Tool::Coverage) {
            process::run(
                workspace.root(),
                "rustup",
                [
                    "component",
                    "add",
                    "--toolchain",
                    "stable",
                    "llvm-tools-preview",
                ],
            )?;
        }
        println!("installing {} under .xtask/tools", tool.package());
        let install_root = local_root(workspace);
        let mut args = vec![
            OsString::from("+stable"),
            OsString::from("install"),
            OsString::from("--root"),
            install_root.into_os_string(),
            OsString::from("--locked"),
        ];
        // WSR's generated release workflow requires its configured dist version.
        let version = if matches!(tool, Tool::Dist) {
            dist_version(workspace)?
        } else {
            None
        };
        if let Some(version) = version {
            args.extend([
                OsString::from("--git"),
                OsString::from("https://github.com/axodotdev/cargo-dist"),
                OsString::from("--tag"),
                OsString::from(format!("v{version}")),
            ]);
            println!("using configured cargo-dist {version}");
        }
        args.push(OsString::from(tool.package()));
        process::run(workspace.root(), "cargo", args)?;
    }
    Ok(())
}

fn resolve_optional(workspace: &Workspace, tool: Tool) -> Option<PathBuf> {
    let version = if matches!(tool, Tool::Dist) {
        dist_version(workspace).ok()?
    } else {
        None
    };
    let available = |program: &std::ffi::OsStr| {
        version.as_ref().map_or_else(
            || process::available(program, tool.prefix()),
            |version| process::available_version(program, tool.prefix(), version),
        )
    };
    let local = local_bin(workspace).join(executable_name(tool.executable()));
    if local.is_file() && available(local.as_os_str()) {
        return Some(local);
    }
    let current = std::env::current_dir().ok()?;
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| {
            current
                .join(directory)
                .join(executable_name(tool.executable()))
        })
        .find(|path| path.is_file() && available(path.as_os_str()))
}

fn dist_version(workspace: &Workspace) -> Result<Option<String>> {
    let path = workspace.path("dist-workspace.toml");
    if !path.is_file() {
        return Ok(None);
    }
    let config = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read dist configuration: {error}"))?
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| format!("invalid dist configuration: {error}"))?;
    config
        .get("dist")
        .and_then(|dist| dist.get("cargo-dist-version"))
        .and_then(toml_edit::Item::as_str)
        .map(|version| Some(version.to_owned()))
        .ok_or_else(|| "dist.cargo-dist-version must be set".to_owned())
}

fn tools(group: ToolGroup) -> &'static [Tool] {
    match group {
        ToolGroup::Ci => CI,
        ToolGroup::Coverage => COVERAGE,
        ToolGroup::Release => RELEASE,
        ToolGroup::Test => TEST,
        ToolGroup::All => ALL,
    }
}

fn local_root(workspace: &Workspace) -> PathBuf {
    workspace.path(".xtask/tools")
}

fn local_bin(workspace: &Workspace) -> PathBuf {
    local_root(workspace).join("bin")
}

fn executable_name(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

const fn group_for(tool: Tool) -> &'static str {
    match tool {
        Tool::Nextest => "test",
        Tool::Coverage => "coverage",
        Tool::Deny | Tool::Typos => "ci",
        Tool::Dist | Tool::Release | Tool::Cliff => "release",
    }
}

const fn state(available: bool) -> &'static str {
    if available { "ready" } else { "missing" }
}
