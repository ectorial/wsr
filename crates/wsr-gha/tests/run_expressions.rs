//! Backlog B-28: `${{` text in a step `run` is reported unevaluated; the script is not retained.

mod common;

use wsr_gha::Category;
use wsr_types::inspection::StepSummaryKind;

#[test]
fn run_scripts_record_unevaluated_expressions_without_retaining_text() {
    let summary = common::inspect(
        b"jobs:\n  a:\n    runs-on: linux\n    steps:\n      - run: echo ${{ inputs.mode }}\n      - run: echo plain $HOME\n      - run: |\n          echo one\n          echo '${{ github.ref }}'\n",
    )
    .unwrap();
    let kinds: Vec<_> = summary.jobs[0]
        .steps
        .iter()
        .map(|step| match step.kind {
            StepSummaryKind::Run { unevaluated } => unevaluated,
            StepSummaryKind::Uses(_) => panic!("expected run steps"),
        })
        .collect();
    assert_eq!(kinds, [true, false, true]);
}

#[test]
fn other_run_rules_are_unchanged() {
    for (source, category, route) in [
        // string-steps-context allows an empty script; WSR does not inspect one.
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: ''\n",
            Category::Unsupported,
            "jobs.b.steps[1].run",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: 5\n",
            Category::Unsupported,
            "jobs.b.steps[1].run",
        ),
        // `shell` has no expression context, also beside an unevaluated script.
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo ${{ x }}\n        shell: ${{ y }}\n",
            Category::Invalid,
            "jobs.b.steps[1].shell",
        ),
        (
            "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo ${{ x }}\n        uses: ./a\n",
            Category::Invalid,
            "jobs.b.steps[1]",
        ),
    ] {
        let error = common::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (category, Some(route)),
            "{source}: {error}"
        );
    }
}
