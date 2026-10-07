//! Rendering for product crates with semantic ownership names.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::workspace::Workspace;

use super::plan::Plan;

pub(crate) fn add(
    workspace: &Workspace,
    semantic_name: &str,
    binary: bool,
    private: bool,
    dry_run: bool,
) -> Result {
    validate_semantic_name(semantic_name)?;
    if semantic_name == "cli" {
        return Err("`cli` is reserved; run `cargo xtask scaffold cli`".into());
    }

    let crate_name = workspace.product_crate(semantic_name);
    let relative = PathBuf::from("crates").join(&crate_name);
    ensure_new_crate_directory(workspace, &relative)?;

    let mut plan = Plan::new(workspace);
    if !binary {
        let mut workspace_manifest = workspace.manifest()?;
        workspace.add_workspace_dependency(
            &mut workspace_manifest,
            &crate_name,
            &relative,
            private,
        )?;
        plan.replace("Cargo.toml", workspace_manifest.to_string());
    }
    plan.create(
        relative.join("Cargo.toml"),
        manifest(&crate_name, semantic_name, binary, private),
    )?;
    plan.create(
        relative.join(if binary { "src/main.rs" } else { "src/lib.rs" }),
        source(&crate_name, semantic_name, binary),
    )?;
    plan.apply(dry_run)
}

fn validate_semantic_name(name: &str) -> Result {
    let mut chars = name.chars();
    let valid = matches!(chars.next(), Some('a'..='z'))
        && chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && !name.ends_with('-')
        && !name.contains("--");
    if !valid {
        return Err(
            "semantic name must start with a lowercase letter and contain lowercase letters, digits, or single hyphens"
                .into(),
        );
    }
    if name == "xtask" {
        return Err("`xtask` is reserved for repository automation".into());
    }
    Ok(())
}

fn ensure_new_crate_directory(workspace: &Workspace, relative: &Path) -> Result {
    let directory = workspace.path(relative);
    if directory.exists() && !directory.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} already exists without a Cargo.toml; refusing to write into it",
            relative.display()
        ));
    }
    Ok(())
}

fn manifest(name: &str, semantic_name: &str, binary: bool, private: bool) -> String {
    let publish = if private { "publish = false\n" } else { "" };
    let target = if binary { "Executable" } else { "Library" };
    format!(
        "[package]\n\
         name = \"{name}\"\n\
         description = \"{target} crate for the {semantic_name} responsibility.\"\n\
         {publish}\
         version.workspace = true\n\
         edition.workspace = true\n\
         rust-version.workspace = true\n\
         authors.workspace = true\n\
         license.workspace = true\n\
         repository.workspace = true\n\
         homepage.workspace = true\n\n\
         [lints]\nworkspace = true\n"
    )
}

fn source(name: &str, semantic_name: &str, binary: bool) -> String {
    if binary {
        format!(
            "//! Executable for the `{semantic_name}` responsibility.\n\n\
             fn main() {{\n    println!(\"Hello from {name}!\");\n}}\n"
        )
    } else {
        format!(
            "//! Product module for the `{semantic_name}` responsibility.\n\n\
             /// Returns a starter message for this module.\n\
             #[must_use]\n\
             pub const fn hello() -> &'static str {{\n\
             \x20   \"Hello from {name}!\"\n\
             }}\n"
        )
    }
}
