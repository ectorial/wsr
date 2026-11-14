//! Backlog B-23: reusable-workflow call jobs are reported as written; nothing is loaded or passed.

mod common;

use std::collections::BTreeMap;

use wsr_gha::Category;
use wsr_types::inspection::{CallSecrets, SourceText, StepValue, WorkflowCall};

fn call(body: &str) -> WorkflowCall {
    let mut summary = common::inspect(format!("jobs:\n  b:\n{body}").as_bytes())
        .unwrap_or_else(|error| panic!("{body}: {error}"));
    let job = summary.jobs.remove(0);
    assert_eq!(job.runner, None);
    assert!(job.steps.is_empty());
    job.call.expect("a call job")
}

fn failure(body: &str) -> (Category, Option<String>) {
    let error = common::inspect(format!("jobs:\n  b:\n{body}").as_bytes()).unwrap_err();
    (error.category(), error.route().map(str::to_owned))
}

#[test]
fn call_jobs_keep_reference_inputs_and_secrets() {
    assert_eq!(
        call("    uses: ./.github/workflows/missing.yml\n"),
        WorkflowCall {
            uses: "./.github/workflows/missing.yml".into(),
            with: None,
            secrets: None,
        }
    );
    assert_eq!(
        call(
            "    uses: octo/repo/.github/workflows/ci.yml@v1\n    with: {mode: fast, count: 2, ref: '${{ github.ref }}'}\n    secrets: inherit\n"
        ),
        WorkflowCall {
            uses: "octo/repo/.github/workflows/ci.yml@v1".into(),
            with: Some(BTreeMap::from([
                ("count".into(), StepValue::Number("2".into())),
                (
                    "mode".into(),
                    StepValue::Text(SourceText::Literal("fast".into()))
                ),
                (
                    "ref".into(),
                    StepValue::Text(SourceText::Unevaluated("${{ github.ref }}".into()))
                ),
            ])),
            secrets: Some(CallSecrets::Inherit),
        }
    );
    assert_eq!(
        call("    uses: ./w.yml\n    secrets: {token: '${{ secrets.T }}'}\n").secrets,
        Some(CallSecrets::Values(BTreeMap::from([(
            "token".into(),
            StepValue::Text(SourceText::Unevaluated("${{ secrets.T }}".into()))
        )])))
    );
    // Keys shared by both job forms.
    let summary = common::inspect(
        b"jobs:\n  a:\n    runs-on: linux\n    steps:\n      - run: echo\n  b:\n    name: call\n    needs: a\n    if: always()\n    permissions: read-all\n    concurrency: g\n    uses: ./w.yml\n",
    )
    .unwrap();
    assert_eq!(summary.jobs[1].needs, ["a"]);
    assert_eq!(
        summary.jobs[1].name,
        Some(SourceText::Literal("call".into()))
    );
}

#[test]
fn call_job_categories_follow_the_pinned_schema() {
    for (body, category, route) in [
        // Only job-factory keys: `uses` matches neither job form.
        (
            "    uses: ./w.yml\n    runs-on: linux\n",
            Category::Invalid,
            "jobs.b.runs-on",
        ),
        (
            "    uses: ./w.yml\n    steps:\n      - run: echo\n",
            Category::Invalid,
            "jobs.b.steps",
        ),
        (
            "    uses: ./w.yml\n    env: {A: b}\n",
            Category::Invalid,
            "jobs.b.env",
        ),
        (
            "    uses: ./w.yml\n    timeout-minutes: 5\n",
            Category::Invalid,
            "jobs.b.timeout-minutes",
        ),
        (
            "    uses: ./w.yml\n    outputs: {a: b}\n",
            Category::Invalid,
            "jobs.b.outputs",
        ),
        (
            "    uses: ./w.yml\n    environment: prod\n",
            Category::Invalid,
            "jobs.b.environment",
        ),
        (
            "    uses: ./w.yml\n    defaults: {run: {shell: bash}}\n",
            Category::Invalid,
            "jobs.b.defaults",
        ),
        (
            "    uses: ./w.yml\n    continue-on-error: true\n",
            Category::Invalid,
            "jobs.b.continue-on-error",
        ),
        // Rejected by placement before the value is decoded, whatever the key order.
        (
            "    runs-on: {group: fleet}\n    uses: ./w.yml\n",
            Category::Invalid,
            "jobs.b.runs-on",
        ),
        (
            "    uses: ./w.yml\n    timeout-minutes: 0\n",
            Category::Invalid,
            "jobs.b.timeout-minutes",
        ),
        (
            "    steps:\n      - bogus: 1\n    uses: ./w.yml\n",
            Category::Invalid,
            "jobs.b.steps",
        ),
        (
            "    uses: ./w.yml\n    cancel-timeout-minutes: 5\n",
            Category::Invalid,
            "jobs.b.cancel-timeout-minutes",
        ),
        (
            "    container: alpine\n    uses: ./w.yml\n",
            Category::Invalid,
            "jobs.b.container",
        ),
        (
            "    uses: ./w.yml\n    services: {db: {image: pg}}\n",
            Category::Invalid,
            "jobs.b.services",
        ),
        (
            "    uses: ./w.yml\n    snapshot: img\n",
            Category::Invalid,
            "jobs.b.snapshot",
        ),
        (
            "    with: {a: '${{ x }}'}\n    runs-on: linux\n    steps:\n      - run: echo\n",
            Category::Invalid,
            "jobs.b.with",
        ),
        // The scan reads keys as the typed decoder does and survives later failures.
        (
            "    !x uses: ./w.yml\n    with: {mode: fast}\n    container: alpine\n",
            Category::Invalid,
            "jobs.b.container",
        ),
        (
            "    uses: ./w.yml\n    container: alpine\n    name: first\n    name: second\n",
            Category::Invalid,
            "jobs.b.container",
        ),
        (
            "    container: alpine\n    uses: ./w.yml\n    ? [x]\n    : y\n",
            Category::Invalid,
            "jobs.b.container",
        ),
        // Only workflow-job keys.
        (
            "    runs-on: linux\n    with: {a: b}\n    steps:\n      - run: echo\n",
            Category::Invalid,
            "jobs.b.with",
        ),
        (
            "    runs-on: linux\n    secrets: inherit\n    steps:\n      - run: echo\n",
            Category::Invalid,
            "jobs.b.secrets",
        ),
        // `uses` is a non-empty string with no expression context.
        ("    uses: '${{ x }}'\n", Category::Invalid, "jobs.b.uses"),
        ("    uses: ''\n", Category::Invalid, "jobs.b.uses"),
        ("    uses: [a]\n", Category::Invalid, "jobs.b.uses"),
        (
            "    uses: octo/repo/w.yml\n",
            Category::Invalid,
            "jobs.b.uses",
        ),
        ("    uses: octo@v1\n", Category::Invalid, "jobs.b.uses"),
        ("    uses: '5'\n", Category::Invalid, "jobs.b.uses"),
        ("    uses: $/w.yml\n", Category::Unsupported, "jobs.b.uses"),
        // `with` and `secrets` have no context of their own; values are scalars.
        (
            "    uses: ./w.yml\n    with: '${{ x }}'\n",
            Category::Invalid,
            "jobs.b.with",
        ),
        (
            "    uses: ./w.yml\n    with: {'${{ x }}': a}\n",
            Category::Invalid,
            "jobs.b.with.${{ x }}",
        ),
        (
            "    uses: ./w.yml\n    with: {a: [b]}\n",
            Category::Invalid,
            "jobs.b.with.a",
        ),
        (
            "    uses: ./w.yml\n    with: {a: 1, A: 2}\n",
            Category::Invalid,
            "jobs.b.with.A",
        ),
        (
            "    uses: ./w.yml\n    secrets: all\n",
            Category::Invalid,
            "jobs.b.secrets",
        ),
        (
            "    uses: ./w.yml\n    secrets: '${{ x }}'\n",
            Category::Invalid,
            "jobs.b.secrets",
        ),
        (
            "    uses: ./w.yml\n    secrets: {a: {b: c}}\n",
            Category::Invalid,
            "jobs.b.secrets.a",
        ),
        (
            "    uses: ./w.yml\n    strategy: {matrix: {a: [1]}}\n",
            Category::Unsupported,
            "jobs.b.strategy",
        ),
    ] {
        assert_eq!(failure(body), (category, Some(route.to_owned())), "{body}");
    }
}

#[test]
fn missing_runner_and_steps_keep_their_diagnostics() {
    for (body, message) in [
        (
            "    steps:\n      - run: echo\n",
            "jobs.b: missing field `runs-on` at line 3 column 5",
        ),
        (
            "    runs-on: linux\n",
            "jobs.b: missing field `steps` at line 3 column 5",
        ),
    ] {
        let error = common::inspect(format!("jobs:\n  b:\n{body}").as_bytes()).unwrap_err();
        assert_eq!(error.category(), Category::Invalid, "{body}");
        assert_eq!(error.to_string(), message, "{body}");
    }
}

// Aliases can repeat a job body, so the scan stops after one job key per source byte. A call job
// past that point is classified by its keys as they are read: those after `uses` are Invalid.
#[test]
fn call_job_keys_after_uses_stay_invalid_when_the_scan_budget_is_exhausted() {
    let anchor = "  x: &x\n    name: n\n    if: always()\n    runs-on: linux\n    timeout-minutes: 5\n    continue-on-error: false\n    env: {A: b}\n    defaults: {run: {shell: bash}}\n    outputs: {o: v}\n    environment: prod\n    permissions: read-all\n    concurrency: g\n    needs: []\n    steps: [{run: echo}]\n";
    let mut source = format!("jobs:\n{anchor}");
    for index in 0..2500 {
        source.push_str(&format!("  a{index}: *x\n"));
    }
    for (key, route, detail) in [
        ("container: y", "jobs.z.container", "container"),
        ("services: {}", "jobs.z.services", "services"),
        ("snapshot: s", "jobs.z.snapshot", "snapshot"),
        (
            "cancel-timeout-minutes: 5",
            "jobs.z.cancel-timeout-minutes",
            "cancel-timeout-minutes",
        ),
    ] {
        let error = common::inspect(format!("{source}  z: {{uses: ./x.yml, {key}}}\n").as_bytes())
            .unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some(route)),
            "{key}: {error}"
        );
        assert!(
            error
                .to_string()
                .contains(&format!("`{detail}` cannot be combined with `uses`")),
            "{key}: {error}"
        );
    }
}
