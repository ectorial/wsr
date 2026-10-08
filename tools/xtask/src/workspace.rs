use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata::{Metadata, MetadataCommand, Package, TargetKind};
use toml_edit::{DocumentMut, InlineTable, Item, Value};

use crate::Result;

pub(crate) struct Workspace {
    root: PathBuf,
    crate_prefix: String,
    primary_crate: String,
    version: String,
    metadata: Metadata,
}

impl Workspace {
    pub(crate) fn discover() -> Result<Self> {
        let start = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?;
        let metadata = MetadataCommand::new()
            .current_dir(&start)
            .no_deps()
            .exec()
            .map_err(|error| format!("failed to read Cargo workspace metadata: {error}"))?;
        let root = metadata.workspace_root.as_std_path().to_path_buf();
        let manifest = read_document(&root.join("Cargo.toml"))?;
        let crate_prefix = manifest["workspace"]["metadata"]["xtask"]["crate-prefix"]
            .as_str()
            .ok_or("workspace.metadata.xtask.crate-prefix must be set")?
            .to_owned();
        let primary_crate = manifest["workspace"]["metadata"]["xtask"]["primary-crate"]
            .as_str()
            .ok_or("workspace.metadata.xtask.primary-crate must be set")?
            .to_owned();
        let version = manifest["workspace"]["package"]["version"]
            .as_str()
            .ok_or("workspace.package.version must be set")?
            .to_owned();

        Ok(Self {
            root,
            crate_prefix,
            primary_crate,
            version,
            metadata,
        })
    }

    pub(crate) fn path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root.join(path)
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn primary_crate(&self) -> &str {
        &self.primary_crate
    }

    pub(crate) fn product_crate(&self, semantic_name: &str) -> String {
        format!("{}-{semantic_name}", self.crate_prefix)
    }

    pub(crate) fn manifest(&self) -> Result<DocumentMut> {
        read_document(&self.path("Cargo.toml"))
    }

    pub(crate) fn primary_manifest_path(&self) -> Result<PathBuf> {
        let package = self.package(&self.primary_crate)?;
        package
            .manifest_path
            .as_std_path()
            .strip_prefix(&self.root)
            .map(Path::to_path_buf)
            .map_err(|error| format!("failed to locate primary crate manifest: {error}"))
    }

    pub(crate) fn product_packages(&self) -> impl Iterator<Item = &Package> {
        self.metadata
            .workspace_packages()
            .into_iter()
            .filter(|package| {
                package
                    .manifest_path
                    .as_std_path()
                    .starts_with(self.root.join("crates"))
            })
    }

    pub(crate) fn add_workspace_dependency(
        &self,
        document: &mut DocumentMut,
        crate_name: &str,
        relative: &Path,
        private: bool,
    ) -> Result {
        let path = relative
            .to_str()
            .ok_or_else(|| format!("non-UTF-8 crate path: {}", relative.display()))?
            .replace('\\', "/");
        let mut dependency = InlineTable::new();
        dependency.insert("path", Value::from(path));
        if !private {
            dependency.insert("version", Value::from(self.version.as_str()));
        }
        insert_inline_dependency(
            &mut document["workspace"]["dependencies"],
            crate_name,
            dependency,
        )
    }

    pub(crate) fn align_workspace_dependency_version(
        &self,
        document: &mut DocumentMut,
        crate_name: &str,
    ) -> Result {
        let dependency = document["workspace"]["dependencies"]
            .get_mut(crate_name)
            .ok_or_else(|| format!("workspace dependency {crate_name:?} does not exist"))?
            .as_inline_table_mut()
            .ok_or_else(|| format!("workspace dependency {crate_name:?} is not an inline table"))?;
        if dependency.get("path").is_none() {
            return Err(format!(
                "workspace dependency {crate_name:?} must have a path before its version can be aligned"
            ));
        }
        dependency.insert("version", Value::from(self.version.as_str()));
        Ok(())
    }

    pub(crate) fn has_publishable_binary(&self) -> bool {
        self.metadata.workspace_packages().iter().any(|package| {
            !package.publish.as_ref().is_some_and(Vec::is_empty)
                && package
                    .targets
                    .iter()
                    .any(|target| target.kind.contains(&TargetKind::Bin))
        })
    }

    fn package(&self, name: &str) -> Result<&Package> {
        self.metadata
            .workspace_packages()
            .into_iter()
            .find(|package| package.name.as_str() == name)
            .ok_or_else(|| format!("workspace package {name:?} does not exist"))
    }
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

pub(crate) fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

pub(crate) fn read_document(path: &Path) -> Result<DocumentMut> {
    read(path)?
        .parse()
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}
