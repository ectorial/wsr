//! Backlog B-11: workflow and job `env` and `defaults.run` as source data, never bound or applied.

use std::collections::BTreeMap;

use wsr_gha::Category;
use wsr_types::inspection::{RunDefaults, SourceText, StepValue};

#[test]
fn workflow_and_job_env_and_run_defaults_keep_source_values() {
    let summary = wsr_gha::inspect(
        b"env: {N: 3, M: '${{ inputs.m }}'}\ndefaults: {run: {shell: bash}}\njobs:\n  b:\n    runs-on: linux\n    env: {}\n    defaults:\n      run: {shell: '${{ matrix.s }}', working-directory: src}\n    steps:\n      - run: echo\n  c:\n    runs-on: linux\n    defaults: {}\n    steps:\n      - run: echo\n",
    )
    .unwrap();
    assert_eq!(
        summary.env,
        Some(BTreeMap::from([
            (
                "M".to_owned(),
                StepValue::Text(SourceText::Unevaluated("${{ inputs.m }}".into()))
            ),
            ("N".to_owned(), StepValue::Number("3".into())),
        ]))
    );
    assert_eq!(
        summary.defaults,
        Some(RunDefaults {
            shell: Some(SourceText::Literal("bash".into())),
            working_directory: None,
        })
    );
    let (b, c) = (&summary.jobs[0], &summary.jobs[1]);
    assert_eq!(b.env, Some(BTreeMap::new()));
    assert_eq!(
        b.defaults,
        Some(RunDefaults {
            shell: Some(SourceText::Unevaluated("${{ matrix.s }}".into())),
            working_directory: Some(SourceText::Literal("src".into())),
        })
    );
    assert_eq!(c.env, None);
    assert_eq!(
        c.defaults,
        Some(RunDefaults {
            shell: None,
            working_directory: None,
        })
    );
}

// Categories follow the pinned schema's expression contexts: workflow-env, job-env and
// job-defaults-run have one (Unsupported); workflow-defaults, workflow-defaults-run and
// job-defaults have none (Invalid).
#[test]
fn env_and_run_default_failures_have_category_and_route() {
    let job = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";
    let in_job = |field: &str| {
        format!("jobs:\n  b:\n    runs-on: linux\n    {field}\n    steps:\n      - run: echo\n")
    };
    for (source, expected) in [
        (
            format!("env: '${{{{ x }}}}'\n{job}"),
            (Category::Unsupported, "env"),
        ),
        (
            format!("env: {{A: ~}}\n{job}"),
            (Category::Invalid, "env.A"),
        ),
        (
            in_job("env: '${{ x }}'"),
            (Category::Unsupported, "jobs.b.env"),
        ),
        (
            in_job("env: {a: x, A: y}"),
            (Category::Invalid, "jobs.b.env.A"),
        ),
        (
            format!("defaults: '${{{{ x }}}}'\n{job}"),
            (Category::Invalid, "defaults"),
        ),
        (
            format!("defaults: [run]\n{job}"),
            (Category::Invalid, "defaults"),
        ),
        (
            format!("defaults: {{shell: bash}}\n{job}"),
            (Category::Invalid, "defaults.shell"),
        ),
        (
            format!("defaults: {{run: '${{{{ x }}}}'}}\n{job}"),
            (Category::Invalid, "defaults.run"),
        ),
        (
            format!("defaults: {{run: {{shell: '${{{{ x }}}}'}}}}\n{job}"),
            (Category::Invalid, "defaults.run.shell"),
        ),
        (
            format!("defaults: {{run: {{env: x}}}}\n{job}"),
            (Category::Invalid, "defaults.run.env"),
        ),
        (
            format!("defaults: {{run: {{'${{{{ x }}}}': bash}}}}\n{job}"),
            (Category::Invalid, "defaults.run.${{ x }}"),
        ),
        (
            in_job("defaults: {run: {'${{ x }}': bash}}"),
            (Category::Unsupported, "jobs.b.defaults.run.${{ x }}"),
        ),
        (
            format!("defaults: {{run: {{shell: true}}}}\n{job}"),
            (Category::Unsupported, "defaults.run.shell"),
        ),
        (
            format!("defaults: {{run: {{shell: ~}}}}\n{job}"),
            (Category::Invalid, "defaults.run.shell"),
        ),
        (
            in_job("defaults: '${{ x }}'"),
            (Category::Invalid, "jobs.b.defaults"),
        ),
        (
            in_job("defaults: {run: '${{ x }}'}"),
            (Category::Unsupported, "jobs.b.defaults.run"),
        ),
        (
            in_job("defaults: {run: {shell: ''}}"),
            (Category::Invalid, "jobs.b.defaults.run.shell"),
        ),
        (
            in_job("defaults: {run: {shell: '  '}}"),
            (Category::Invalid, "jobs.b.defaults.run.shell"),
        ),
        (
            in_job("defaults: {run: {working-directory: ''}}"),
            (Category::Invalid, "jobs.b.defaults.run.working-directory"),
        ),
        (
            in_job("defaults: {run: {working-directory: '  '}}"),
            (
                Category::Unsupported,
                "jobs.b.defaults.run.working-directory",
            ),
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
