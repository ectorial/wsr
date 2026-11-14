//! B-36: pull request activity `types` are retained in source order, never matched.
mod common;

use wsr_gha::Category::Invalid;
use wsr_types::inspection::{EventFilter, Trigger};

const JOBS: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

fn filters(on: &str) -> Vec<EventFilter> {
    match common::inspect(format!("on: {on}\n{JOBS}").as_bytes())
        .unwrap_or_else(|error| panic!("{on}: {error}"))
        .triggers
        .remove(0)
    {
        Trigger::Filtered { filters, .. } => filters,
        trigger => panic!("{on}: {trigger:?}"),
    }
}

fn filter(name: &str, patterns: &[&str]) -> EventFilter {
    EventFilter {
        name: name.into(),
        patterns: patterns.iter().map(|pattern| (*pattern).into()).collect(),
    }
}

#[test]
fn activity_types_are_retained_in_source_order_beside_filters() {
    assert_eq!(
        filters("{pull_request: {branches: [main], types: [synchronize, opened, opened]}}"),
        [
            filter("branches", &["main"]),
            filter("types", &["synchronize", "opened", "opened"]),
        ]
    );
    assert_eq!(
        filters("{pull_request_target: {types: closed, paths: [src]}}"),
        [filter("types", &["closed"]), filter("paths", &["src"])]
    );
    assert_eq!(
        filters("{pull_request: {types: []}}"),
        [filter("types", &[])]
    );
    for name in [
        "assigned",
        "unassigned",
        "labeled",
        "unlabeled",
        "opened",
        "edited",
        "closed",
        "reopened",
        "synchronize",
        "converted_to_draft",
        "locked",
        "unlocked",
        "enqueued",
        "dequeued",
        "milestoned",
        "demilestoned",
        "ready_for_review",
        "review_requested",
        "review_request_removed",
        "auto_merge_enabled",
        "auto_merge_disabled",
        "stacked",
    ] {
        for event in ["pull_request", "pull_request_target"] {
            assert_eq!(
                filters(&format!("{{{event}: {{types: {name}}}}}")),
                [filter("types", &[name])]
            );
        }
    }
}

#[test]
fn activity_types_outside_the_allowed_values_are_invalid() {
    for (types, route) in [
        ("Opened", ".types"),
        ("published", ".types"),
        ("'${{ x }}'", ".types"),
        ("''", ".types"),
        ("5", ".types"),
        ("true", ".types"),
        ("~", ".types"),
        ("{a: b}", ".types"),
        ("[opened, OPENED]", ".types[2]"),
        ("[opened, 5]", ".types[2]"),
        ("[opened, ~]", ".types[2]"),
        ("[[opened]]", ".types[1]"),
    ] {
        for event in ["pull_request", "pull_request_target"] {
            let on = format!("{{{event}: {{types: {types}}}}}");
            let Err(error) = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()) else {
                panic!("accepted {on}");
            };
            assert_eq!(
                (error.category(), error.route()),
                (Invalid, Some(format!("on.{event}{route}").as_str())),
                "{on}: {error}"
            );
        }
    }
}

// A number or boolean is named like any other unknown activity type (B-36 review P3).
#[test]
fn converted_scalars_are_named_as_unknown_activity_types() {
    for (types, text) in [("5", "'5'"), ("true", "'true'"), ("!!str 5", "'5'")] {
        let on = format!("{{pull_request: {{types: {types}}}}}");
        let Err(error) = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()) else {
            panic!("accepted {on}");
        };
        assert_eq!(
            error.to_string(),
            format!("on.pull_request.types: unknown activity type {text}"),
            "{on}"
        );
    }
}
