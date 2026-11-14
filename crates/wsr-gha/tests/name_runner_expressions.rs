//! Backlog B-24: `${{` text in job `name`, step `name` and job `runs-on` is reported unevaluated.

mod common;

use wsr_gha::Category;
use wsr_types::inspection::{RunnerRequirement, SourceText};

fn unevaluated(text: &str) -> SourceText {
    SourceText::Unevaluated(text.into())
}

#[test]
fn names_and_runner_labels_keep_unevaluated_text() {
    let summary = common::inspect(
        b"jobs:\n  a:\n    name: build ${{ matrix.os }}\n    runs-on: ${{ matrix.os }}\n    steps:\n      - name: ${{ inputs.step }}\n        run: echo\n  b:\n    runs-on: [self-hosted, '${{ vars.POOL }}']\n    steps:\n      - run: echo\n  c:\n    name: ${{ inputs.call }}\n    uses: ./w.yml\n",
    )
    .unwrap();
    let [a, b, c] = &summary.jobs[..] else {
        panic!("expected three jobs");
    };
    assert_eq!(a.name, Some(unevaluated("build ${{ matrix.os }}")));
    assert_eq!(
        a.runner,
        Some(RunnerRequirement::Label(unevaluated("${{ matrix.os }}")))
    );
    assert_eq!(a.steps[0].name, Some(unevaluated("${{ inputs.step }}")));
    assert_eq!(
        b.runner,
        Some(RunnerRequirement::Labels(vec![
            SourceText::Literal("self-hosted".into()),
            unevaluated("${{ vars.POOL }}"),
        ]))
    );
    assert_eq!(c.name, Some(unevaluated("${{ inputs.call }}")));
}

#[test]
fn other_name_and_runner_rules_are_unchanged() {
    for (source, category, route) in [
        // The workflow `name` has no expression context.
        (
            "name: ${{ x }}\njobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: a\n",
            Category::Invalid,
            "name",
        ),
        (
            "jobs:\n  b:\n    runs-on: ' '\n    steps:\n      - run: a\n",
            Category::Invalid,
            "jobs.b.runs-on",
        ),
        (
            "jobs:\n  b:\n    runs-on: [linux, '']\n    steps:\n      - run: a\n",
            Category::Invalid,
            "jobs.b.runs-on[2]",
        ),
        (
            "jobs:\n  b:\n    runs-on: {group: '${{ x }}'}\n    steps:\n      - run: a\n",
            Category::Unsupported,
            "jobs.b.runs-on",
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
