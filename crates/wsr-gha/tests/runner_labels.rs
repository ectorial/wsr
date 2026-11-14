mod common;
use wsr_types::inspection::{RunnerRequirement, SourceText};

fn literal(text: &str) -> SourceText {
    SourceText::Literal(text.into())
}

#[test]
fn source_summary_retains_scalar_and_list_forms() {
    for (source, expected) in [
        ("linux", RunnerRequirement::Label(literal("linux"))),
        ("[linux]", RunnerRequirement::Labels(vec![literal("linux")])),
        (
            "[Linux, x64, Linux]",
            RunnerRequirement::Labels(vec![literal("Linux"), literal("x64"), literal("Linux")]),
        ),
    ] {
        let raw = format!(
            "jobs:\n  build:\n    runs-on: {source}\n    steps:\n      - run: echo hello\n"
        );
        let summary = common::inspect(raw.as_bytes()).unwrap();
        assert_eq!(summary.jobs[0].runner, Some(expected));
    }
}
