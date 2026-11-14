//! B-34: cancellation timeout is source data, never applied.
mod common;

use wsr_gha::Category;
use wsr_types::inspection::SourceText;

fn job(value: &str) -> String {
    format!(
        "jobs:\n  a:\n    runs-on: linux\n    cancel-timeout-minutes: {value}\n    steps: [{{run: echo}}]\n"
    )
}

#[test]
fn retains_literal_and_unevaluated_cancel_timeouts() {
    for (value, expected) in [
        ("5", SourceText::Literal("5".into())),
        ("0.5", SourceText::Literal("0.5".into())),
        (
            "${{ matrix.minutes }}",
            SourceText::Unevaluated("${{ matrix.minutes }}".into()),
        ),
    ] {
        let summary = common::inspect(job(value).as_bytes()).unwrap();
        assert_eq!(summary.jobs[0].cancel_timeout_minutes, Some(expected));
    }
    let summary =
        common::inspect(b"jobs:\n  a:\n    runs-on: linux\n    steps: [{run: echo}]\n").unwrap();
    assert_eq!(summary.jobs[0].cancel_timeout_minutes, None);
}

#[test]
fn cancel_timeout_categories_match_job_timeout_subset() {
    for (value, category) in [
        ("0", Category::Unsupported),
        ("-1", Category::Unsupported),
        ("true", Category::Invalid),
        ("'5'", Category::Invalid),
        ("null", Category::Invalid),
        ("[5]", Category::Invalid),
        (".inf", Category::Invalid),
    ] {
        let error = common::inspect(job(value).as_bytes()).unwrap_err();
        assert_eq!(error.category(), category, "{value}: {error}");
        assert_eq!(error.route(), Some("jobs.a.cancel-timeout-minutes"));
        assert!(
            error.to_string().contains("expected a positive number"),
            "{error}"
        );
    }
}

#[test]
fn cancel_timeout_cannot_be_combined_with_uses_in_either_order() {
    for fields in [
        "    uses: ./called.yml\n    cancel-timeout-minutes: 5\n",
        "    cancel-timeout-minutes: 5\n    uses: ./called.yml\n",
    ] {
        let error = common::inspect(format!("jobs:\n  a:\n{fields}").as_bytes()).unwrap_err();
        assert_eq!(error.category(), Category::Invalid);
        assert!(
            error
                .to_string()
                .contains("`cancel-timeout-minutes` cannot be combined with `uses`"),
            "{error}"
        );
    }
}

// Past the B-23 scan budget, a cancellation timeout read before `uses` is decoded, so the call
// check must reject it instead of leaving it unreported (B-34 review).
#[test]
fn cancel_timeout_before_uses_is_invalid_when_the_scan_budget_is_exhausted() {
    let mut source = "jobs:\n  x: &x\n    name: n\n    if: always()\n    runs-on: linux\n    timeout-minutes: 5\n    continue-on-error: false\n    env: {A: b}\n    defaults: {run: {shell: bash}}\n    outputs: {o: v}\n    environment: prod\n    permissions: read-all\n    concurrency: g\n    needs: []\n    steps: [{run: echo}]\n".to_owned();
    for index in 0..2500 {
        source.push_str(&format!("  a{index}: *x\n"));
    }
    source.push_str("  z: {cancel-timeout-minutes: 5, uses: ./x.yml}\n");
    let Err(error) = common::inspect(source.as_bytes()) else {
        panic!("inspection succeeded");
    };
    assert_eq!(
        (error.category(), error.route()),
        (Category::Invalid, Some("jobs.z.cancel-timeout-minutes")),
        "{error}"
    );
    assert!(
        error
            .to_string()
            .contains("`cancel-timeout-minutes` cannot be combined with `uses`"),
        "{error}"
    );
}
