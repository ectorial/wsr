use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::Result;

pub(crate) fn run<I, S>(directory: &Path, program: impl AsRef<OsStr>, args: I) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_with_path(directory, program, args, None)
}

pub(crate) fn run_with_path<I, S>(
    directory: &Path,
    program: impl AsRef<OsStr>,
    args: I,
    extra_path: Option<&Path>,
) -> Result
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let program = program.as_ref();
    let args: Vec<OsString> = args
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect();
    let rendered = std::iter::once(program.to_string_lossy())
        .chain(args.iter().map(|argument| argument.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");
    println!("$ {rendered}");

    let mut command = Command::new(program);
    command.current_dir(directory).args(&args);
    if let Some(extra_path) = extra_path {
        let mut paths = vec![extra_path.to_path_buf()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let joined = std::env::join_paths(paths)
            .map_err(|error| format!("failed to construct tool PATH: {error}"))?;
        command.env("PATH", joined);
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to run {rendered}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{rendered} exited with status {}",
            status.code().unwrap_or(1)
        ))
    }
}

pub(crate) fn available(program: &OsStr, prefix: &[&str]) -> bool {
    Command::new(program)
        .args(prefix)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub(crate) fn available_version(program: &OsStr, prefix: &[&str], version: &str) -> bool {
    Command::new(program)
        .args(prefix)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && [output.stdout, output.stderr]
                    .concat()
                    .split(u8::is_ascii_whitespace)
                    .any(|word| word == version.as_bytes())
        })
}

/// Build documentation without relaxing an existing Rustdoc flag set.
pub(crate) fn rustdoc(directory: &Path) -> Result {
    let mut flags = std::env::var_os("RUSTDOCFLAGS").unwrap_or_default();
    flags.push(" -D warnings");
    println!("$ cargo doc --workspace --no-deps --locked (Rustdoc warnings denied)");
    let status = Command::new("cargo")
        .current_dir(directory)
        .args(["doc", "--workspace", "--no-deps", "--locked"])
        .env("RUSTDOCFLAGS", flags)
        .status()
        .map_err(|error| format!("failed to run cargo doc: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "cargo doc exited with status {}",
            status.code().unwrap_or(1)
        ))
    }
}
