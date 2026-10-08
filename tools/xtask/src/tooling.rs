//! Shared action references for workflow rendering.

use toml_edit::DocumentMut;

fn registry() -> DocumentMut {
    include_str!("../assets/tooling.toml")
        .parse()
        .expect("the bundled tooling registry must be valid TOML")
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
    rendered
}
