use wsr_types::inspection::SourceText;

fn run_name(declaration: &str) -> Option<SourceText> {
    wsr_gha::inspect(
        format!("{declaration}jobs:\n  build:\n    runs-on: linux\n    steps:\n      - run: echo hello\n")
            .as_bytes(),
    )
    .unwrap()
    .run_name
}

#[test]
fn run_name_summary_distinguishes_literal_unevaluated_and_omitted_text() {
    assert_eq!(run_name(""), None);
    assert_eq!(
        run_name("run-name: Local reference\n"),
        Some(SourceText::Literal("Local reference".into()))
    );
    assert_eq!(
        run_name("run-name: \"Shell reference — ${{ inputs.mode }}\"\n"),
        Some(SourceText::Unevaluated(
            "Shell reference — ${{ inputs.mode }}".into()
        ))
    );
    assert_eq!(
        run_name("run-name: \"${{ a }} and ${{ b\"\n"),
        Some(SourceText::Unevaluated("${{ a }} and ${{ b".into()))
    );
}
