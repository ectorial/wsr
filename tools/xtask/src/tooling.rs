//! Shared, bundled versions for workflow rendering and local tool installs.

use toml_edit::DocumentMut;

fn registry() -> DocumentMut {
    include_str!("../assets/tooling.toml")
        .parse()
        .expect("the bundled tooling registry must be valid TOML")
}

pub(crate) fn version(package: &str) -> String {
    registry()["tools"][package]["version"]
        .as_str()
        .expect("every registered tool must have a version")
        .to_owned()
}

pub(crate) fn workflow(template: &str) -> String {
    let registry = registry();
    let mut rendered = template.to_owned();
    for (name, action) in registry["actions"].as_table().expect("actions table") {
        rendered = rendered.replace(
            &format!("@{name}@"),
            action.as_str().expect("action reference"),
        );
    }
    for (name, tool) in registry["tools"].as_table().expect("tools table") {
        rendered = rendered.replace(
            &format!("@{name}@"),
            tool["version"].as_str().expect("tool version"),
        );
    }
    rendered
}
