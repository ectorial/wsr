mod common;
use std::collections::BTreeMap;

use wsr_gha::Category;
use wsr_types::inspection::{PermissionLevel, Permissions};

const JOB: &str = "    runs-on: linux\n    steps:\n      - run: echo\n";

#[test]
fn permissions_summary_keeps_workflow_and_job_requests_separate() {
    let summary = common::inspect(
        format!(
            "permissions:\n  contents: read\n  id-token: none\njobs:\n  a:\n{JOB}  b:\n    permissions: read-all\n{JOB}  c:\n    permissions: {{}}\n{JOB}"
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(
        summary.permissions,
        Some(Permissions::Scopes(BTreeMap::from([
            ("contents".to_owned(), PermissionLevel::Read),
            ("id-token".to_owned(), PermissionLevel::None),
        ])))
    );
    let jobs: Vec<_> = summary
        .jobs
        .iter()
        .map(|job| (job.id.as_str(), job.permissions.as_ref()))
        .collect();
    assert_eq!(
        jobs,
        [
            ("a", None),
            ("b", Some(&Permissions::ReadAll)),
            ("c", Some(&Permissions::Scopes(BTreeMap::new()))),
        ]
    );
}

#[test]
fn permissions_outside_the_pinned_schema_are_invalid_at_their_route() {
    for (source, route) in [
        (
            format!("permissions: none\njobs:\n  b:\n{JOB}"),
            "permissions",
        ),
        (
            format!("permissions: ${{{{ x }}}}\njobs:\n  b:\n{JOB}"),
            "permissions",
        ),
        (
            format!("permissions: {{bogus: read}}\njobs:\n  b:\n{JOB}"),
            "permissions.bogus",
        ),
        (
            format!("jobs:\n  b:\n    permissions: {{vulnerability-alerts: write}}\n{JOB}"),
            "jobs.b.permissions.vulnerability-alerts",
        ),
        (
            format!("jobs:\n  b:\n    permissions: {{copilot-requests: read}}\n{JOB}"),
            "jobs.b.permissions.copilot-requests",
        ),
        (
            format!("jobs:\n  b:\n    permissions: {{contents: true}}\n{JOB}"),
            "jobs.b.permissions.contents",
        ),
    ] {
        let error = common::inspect(source.as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some(route)),
            "{source}"
        );
    }
}

#[test]
fn duplicate_permission_scopes_keep_decoder_evidence() {
    let error = common::inspect(
        format!("permissions: {{contents: read, contents: write}}\njobs:\n  b:\n{JOB}").as_bytes(),
    )
    .unwrap_err();
    assert_eq!(error.category(), Category::Invalid);
    assert!(
        error
            .to_string()
            .contains("duplicate entry with key \"contents\""),
        "{error}"
    );
}
