//! Runnable test examples with explicit implementation obligations.

pub(super) fn unit(body: &str) -> String {
    format!(
        "\n#[cfg(test)]\nmod tests {{\n    // Unit tests can access private items through `super` or `crate`.\n    #[test]\n    fn unit_behavior() {{\n{body}\n        todo!(\"TEST_OBLIGATION: replace this unit example with domain assertions\");\n    }}\n}}\n"
    )
}

pub(super) fn documentation(body: &str) -> String {
    let example = format!("//! {}\n", body.replace('\n', "\n//! "));
    format!(
        "//!\n//! # Examples\n//!\n//! Documentation tests use the public API, like downstream callers.\n//! ```\n{example}//! todo!(\"TEST_OBLIGATION: replace this documentation example with domain assertions\");\n//! ```\n"
    )
}

pub(super) fn integration(body: &str) -> String {
    format!(
        "//! Public API integration test; private crate internals are unavailable here.\n\n#[test]\nfn integration_behavior() {{\n{body}\n    todo!(\"TEST_OBLIGATION: replace this integration example with domain assertions\");\n}}\n"
    )
}

pub(super) fn e2e(binary: &str, arguments: &str) -> String {
    format!(
        "//! End-to-end integration test for the compiled executable.\n\n#[test]\nfn executable_behavior() {{\n    let executable = env!(\"CARGO_BIN_EXE_{binary}\");\n    let output = std::process::Command::new(executable)\n        .args({arguments})\n        .output()\n        .expect(\"executable should start\");\n    assert!(output.status.success());\n    let stdout = String::from_utf8(output.stdout).expect(\"stdout should be UTF-8\");\n    assert!(stdout.starts_with(\"Hello from \"));\n    todo!(\"TEST_OBLIGATION: assert the executable's domain output\");\n}}\n"
    )
}
