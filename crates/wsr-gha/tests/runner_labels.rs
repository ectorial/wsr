use wsr_types::inspection::RunnerRequirement;

#[test]
fn source_summary_retains_scalar_and_list_forms() {
    for (source, expected) in [
        ("linux", RunnerRequirement::Label("linux".into())),
        ("[linux]", RunnerRequirement::Labels(vec!["linux".into()])),
        (
            "[Linux, x64, Linux]",
            RunnerRequirement::Labels(vec!["Linux".into(), "x64".into(), "Linux".into()]),
        ),
    ] {
        let raw = format!(
            "jobs:\n  build:\n    runs-on: {source}\n    steps:\n      - run: echo hello\n"
        );
        let summary = wsr_gha::inspect(raw.as_bytes()).unwrap();
        assert_eq!(summary.jobs[0].runner, Some(expected));
    }
}
