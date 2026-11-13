//! Backlog B-12: job and step `if` kept as written, never parsed or evaluated.

use wsr_gha::Category;

// Contract change: job `if` was reported Unsupported (B-0 acceptance); it is now inspected.
#[test]
fn job_and_step_conditions_are_kept_as_written() {
    let summary = wsr_gha::inspect(
        b"jobs:\n  b:\n    if: success() && github.ref == 'main'\n    runs-on: linux\n    steps:\n      - run: echo\n        if: ${{ always() }}\n      - uses: ./a\n      - run: echo\n        if: \"${{ x }} == 'y'\"\n",
    )
    .unwrap();
    let job = &summary.jobs[0];
    assert_eq!(
        job.condition.as_deref(),
        Some("success() && github.ref == 'main'")
    );
    let conditions: Vec<_> = job.steps.iter().map(|s| s.condition.as_deref()).collect();
    assert_eq!(
        conditions,
        [Some("${{ always() }}"), None, Some("${{ x }} == 'y'")]
    );
}

#[test]
fn conditions_outside_the_subset_have_category_and_route() {
    let job = |field: &str| {
        format!("jobs:\n  b:\n    {field}\n    runs-on: linux\n    steps:\n      - run: echo\n")
    };
    let step = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n        {field}\n")
    };
    for (source, expected) in [
        (job("if: true"), (Category::Unsupported, "jobs.b.if")),
        (step("if: 1"), (Category::Unsupported, "jobs.b.steps[1].if")),
        (job("if: ''"), (Category::Unsupported, "jobs.b.if")),
        (
            step("if: '  '"),
            (Category::Unsupported, "jobs.b.steps[1].if"),
        ),
        (job("if: ~"), (Category::Invalid, "jobs.b.if")),
        (
            step("if: {a: b}"),
            (Category::Invalid, "jobs.b.steps[1].if"),
        ),
    ] {
        let error = wsr_gha::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (expected.0, Some(expected.1)),
            "{source}: {error}"
        );
    }
}
