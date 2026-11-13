use wsr_gha::Category;
use wsr_types::inspection::SourceText;

#[test]
fn timeout_and_shell_are_kept_as_source_data() {
    let summary = wsr_gha::inspect(
        b"jobs:\n  a:\n    runs-on: linux\n    timeout-minutes: 5\n    steps:\n      - run: echo\n        shell: bash\n      - run: echo\n  b:\n    runs-on: linux\n    timeout-minutes: ${{ inputs.t }}\n    steps:\n      - run: echo\n  c:\n    runs-on: linux\n    steps:\n      - run: echo\n",
    )
    .unwrap();
    let timeouts: Vec<_> = summary
        .jobs
        .iter()
        .map(|job| job.timeout_minutes.as_ref())
        .collect();
    assert_eq!(
        timeouts,
        [
            Some(&SourceText::Literal("5".into())),
            Some(&SourceText::Unevaluated("${{ inputs.t }}".into())),
            None,
        ]
    );
    let shells: Vec<_> = summary.jobs[0]
        .steps
        .iter()
        .map(|step| step.shell.as_deref())
        .collect();
    assert_eq!(shells, [Some("bash"), None]);
}

#[test]
fn invalid_timeouts_and_shells_are_invalid_at_their_route() {
    for (source, route) in [
        (
            "jobs:\n  b:\n    runs-on: linux\n    timeout-minutes: -5\n    steps:\n      - run: echo\n",
            "jobs.b.timeout-minutes",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        shell: ${{ x }}\n",
            "jobs.b.steps[1].shell",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - uses: ./a\n        shell: sh\n",
            "jobs.b.steps[1].shell",
        ),
    ] {
        let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some(route)),
            "{source}"
        );
    }
}
