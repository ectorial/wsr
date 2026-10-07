//! CLI parser and entrypoint scaffolding.

use std::path::PathBuf;

use toml_edit::{ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value, value};

use crate::Result;
use crate::cli::CliEntrypoint;
use crate::workspace::{Workspace, read_document};

use super::plan::Plan;

pub(crate) fn add(workspace: &Workspace, entrypoint: CliEntrypoint, dry_run: bool) -> Result {
    let primary = workspace.primary_crate();
    let cli_name = workspace.product_crate("cli");
    let cli_relative = PathBuf::from("crates").join(&cli_name);
    ensure_new_crate_directory(workspace, &cli_relative)?;

    let mut workspace_manifest = workspace.manifest()?;
    if matches!(entrypoint, CliEntrypoint::Primary) {
        workspace.add_workspace_dependency(
            &mut workspace_manifest,
            &cli_name,
            &cli_relative,
            false,
        )?;
    }
    workspace.align_workspace_dependency_version(&mut workspace_manifest, primary)?;
    Workspace::add_workspace_external_dependency(
        &mut workspace_manifest,
        "clap",
        version_with_features("4", &["derive"]),
    )?;

    let mut plan = Plan::new(workspace);
    plan.replace("Cargo.toml", workspace_manifest.to_string());
    plan.create(
        cli_relative.join("Cargo.toml"),
        cli_manifest(primary, &cli_name, entrypoint),
    )?;
    plan.create(cli_relative.join("src/lib.rs"), cli_source(primary))?;

    match entrypoint {
        CliEntrypoint::Companion => {
            plan.create(
                cli_relative.join("src/main.rs"),
                companion_binary_source(primary, &cli_name),
            )?;
        }
        CliEntrypoint::Primary => {
            let primary_manifest_path = workspace.primary_manifest_path()?;
            let mut primary_manifest = read_document(&workspace.path(&primary_manifest_path))?;
            primary_manifest["package"]["default-run"] = value(primary);
            disable_binary_harness(&mut primary_manifest, primary)?;
            insert_inline_dependency(
                &mut primary_manifest["dependencies"],
                &cli_name,
                workspace_reference(),
            )?;
            insert_inline_dependency(
                &mut primary_manifest["dependencies"],
                "clap",
                workspace_reference(),
            )?;
            plan.replace(primary_manifest_path.clone(), primary_manifest.to_string());
            let primary_directory = primary_manifest_path
                .parent()
                .ok_or("primary crate manifest has no parent directory")?;
            plan.create(
                primary_directory
                    .join("src/bin")
                    .join(format!("{primary}.rs")),
                primary_binary_source(primary, &cli_name),
            )?;
        }
    }
    plan.apply(dry_run)
}

fn ensure_new_crate_directory(workspace: &Workspace, relative: &std::path::Path) -> Result {
    let directory = workspace.path(relative);
    if directory.exists() && !directory.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} already exists without a Cargo.toml; refusing to write into it",
            relative.display()
        ));
    }
    Ok(())
}

fn disable_binary_harness(document: &mut DocumentMut, project: &str) -> Result {
    let mut target = Table::new();
    target["name"] = value(project);
    target["path"] = value(format!("src/bin/{project}.rs"));
    target["test"] = value(false);
    target["harness"] = value(false);

    if let Some(item) = document.get_mut("bin") {
        let binaries = item
            .as_array_of_tables_mut()
            .ok_or("the primary crate's bin entry is not an array of tables")?;
        if let Some(existing) = binaries
            .iter_mut()
            .find(|binary| binary.get("name").and_then(Item::as_str) == Some(project))
        {
            existing["test"] = value(false);
            existing["harness"] = value(false);
        } else {
            binaries.push(target);
        }
    } else {
        let mut binaries = ArrayOfTables::new();
        binaries.push(target);
        document["bin"] = Item::ArrayOfTables(binaries);
    }
    Ok(())
}

fn insert_inline_dependency(table: &mut Item, name: &str, dependency: InlineTable) -> Result {
    if let Some(existing) = table.get(name) {
        if existing
            .as_inline_table()
            .is_some_and(|existing| inline_tables_match(existing, &dependency))
        {
            return Ok(());
        }
        return Err(format!(
            "workspace dependency {name:?} already has another value"
        ));
    }
    table[name] = Item::Value(Value::InlineTable(dependency));
    Ok(())
}

fn inline_tables_match(left: &InlineTable, right: &InlineTable) -> bool {
    left.len() == right.len()
        && right.iter().all(|(key, value)| {
            left.get(key)
                .is_some_and(|existing| values_match(existing, value))
        })
}

fn values_match(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::String(left), Value::String(right)) => left.value() == right.value(),
        (Value::Boolean(left), Value::Boolean(right)) => left.value() == right.value(),
        (Value::Integer(left), Value::Integer(right)) => left.value() == right.value(),
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| values_match(left, right))
        }
        _ => false,
    }
}

fn workspace_reference() -> InlineTable {
    let mut dependency = InlineTable::new();
    dependency.insert("workspace", Value::from(true));
    dependency
}

fn version_with_features(version: &str, features: &[&str]) -> InlineTable {
    let mut dependency = InlineTable::new();
    dependency.insert("version", Value::from(version));
    let mut values = toml_edit::Array::new();
    values.extend(features.iter().copied());
    dependency.insert("features", Value::Array(values));
    dependency
}

fn cli_manifest(project: &str, cli_name: &str, entrypoint: CliEntrypoint) -> String {
    let companion = matches!(entrypoint, CliEntrypoint::Companion);
    let publishing = if companion { "publish = false\n" } else { "" };
    let project_dependency = companion.then(|| format!("{project} = {{ workspace = true }}\n"));
    let binary = companion.then(|| {
        format!(
            "\n[[bin]]\nname = \"{project}\"\npath = \"src/main.rs\"\ntest = false\nharness = false\n"
        )
    });
    format!(
        "[package]\n\
         name = \"{cli_name}\"\n\
         description = \"Command-line model for {project}.\"\n\
         {publishing}\
         version.workspace = true\n\
         edition.workspace = true\n\
         rust-version.workspace = true\n\
         authors.workspace = true\n\
         license.workspace = true\n\
         repository.workspace = true\n\
         homepage.workspace = true\n\n\
         [lints]\nworkspace = true\n\n\
         [dependencies]\nclap = {{ workspace = true }}\n{project_dependency}{binary}",
        project_dependency = project_dependency.unwrap_or_default(),
        binary = binary.unwrap_or_default(),
    )
}

fn cli_source(project: &str) -> String {
    format!(
        "//! Clap command model for `{project}`.\n\n\
         use clap::{{Parser, Subcommand}};\n\n\
         /// `{project}` command-line interface.\n\
         #[derive(Debug, Parser)]\n\
         #[command(name = \"{project}\", version, about)]\n\
         pub struct Cli {{\n\
         \x20   /// Command to run.\n\
         \x20   #[command(subcommand)]\n\
         \x20   pub command: Command,\n\
         }}\n\n\
         /// Available commands.\n\
         #[derive(Debug, Subcommand)]\n\
         pub enum Command {{\n\
         \x20   /// Print a greeting.\n\
         \x20   Hello {{\n\
         \x20       /// Name to greet.\n\
         \x20       #[arg(default_value = \"world\")]\n\
         \x20       name: String,\n\
         \x20   }},\n\
         }}\n"
    )
}

fn companion_binary_source(project: &str, cli_name: &str) -> String {
    let cli_crate = cli_name.replace('-', "_");
    let primary_crate = project.replace('-', "_");
    format!(
        "//! Executable entrypoint for `{project}`.\n\n\
         use clap::Parser;\n\
         use {cli_crate}::{{Cli, Command}};\n\n\
         fn main() {{\n\
         \x20   let output = match Cli::parse().command {{\n\
         \x20       Command::Hello {{ name }} => format!(\"{{}}, {{name}}!\", {primary_crate}::hello()),\n\
         \x20   }};\n\
         \x20   println!(\"{{output}}\");\n\
         }}\n"
    )
}

fn primary_binary_source(project: &str, cli_name: &str) -> String {
    let cli_crate = cli_name.replace('-', "_");
    let primary_crate = project.replace('-', "_");
    format!(
        "//! Executable entrypoint for `{project}`.\n\n\
         use clap::Parser;\n\
         use {cli_crate}::{{Cli, Command}};\n\n\
         fn main() {{\n\
         \x20   let output = match Cli::parse().command {{\n\
         \x20       Command::Hello {{ name }} => format!(\"{{}}, {{name}}!\", {primary_crate}::hello()),\n\
         \x20   }};\n\
         \x20   println!(\"{{output}}\");\n\
         }}\n"
    )
}
