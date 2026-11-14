//! Backlog B-15: non-dispatch `on:` events with empty bodies are reported in source order.

mod common;

use wsr_gha::Category;
use wsr_types::inspection::{DispatchBody, EventFilter, ManualTrigger, Trigger};

const JOBS: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

fn triggers(on: &str) -> Vec<Trigger> {
    common::inspect(format!("on: {on}\n{JOBS}").as_bytes())
        .unwrap_or_else(|error| panic!("{on}: {error}"))
        .triggers
}

fn event(name: &str) -> Trigger {
    Trigger::Event(name.into())
}

#[test]
fn events_are_kept_in_source_order() {
    // B-25 contract change: a missing `on` is invalid, no longer an empty trigger list.
    assert_eq!(
        wsr_gha::inspect(JOBS.as_bytes()).unwrap_err().to_string(),
        "missing field `on`"
    );
    assert_eq!(triggers("workflow_call"), [event("workflow_call")]);
    assert_eq!(triggers("schedule"), [event("schedule")]);
    assert_eq!(
        triggers("[push, workflow_dispatch, pull_request]"),
        [
            event("push"),
            Trigger::Manual(ManualTrigger::Scalar),
            event("pull_request")
        ]
    );
    assert_eq!(
        triggers("{workflow_call: null, push: {}, create: null, workflow_dispatch: {inputs: {}}}"),
        [
            event("workflow_call"),
            event("push"),
            event("create"),
            Trigger::Manual(ManualTrigger::Mapping(Some(DispatchBody {
                inputs: Some(vec![])
            }))),
        ]
    );
}

#[test]
fn event_configuration_stays_unsupported() {
    for (on, route) in [
        // B-36 contract change: pull request `types` now inspect (activity_types.rs).
        ("{release: {types: [published]}}", "on.release"),
        ("{workflow_call: {inputs: {}}}", "on.workflow_call"),
        (
            "{workflow_call: {secrets: {token: {required: true}}}}",
            "on.workflow_call",
        ),
        ("{schedule: [{cron: '0 0 * * *'}]}", "on.schedule"),
        ("{schedule: []}", "on.schedule"),
        ("[push, push]", "on[2]"),
        // The strict schema has no minimum size for either collection.
        ("[]", "on"),
        ("{}", "on"),
    ] {
        let error = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Unsupported, Some(route)),
            "{on}: {error}"
        );
    }
}

#[test]
fn unknown_events_and_malformed_bodies_are_invalid() {
    for (on, route) in [
        ("bogus", "on"),
        ("'${{ x }}'", "on"),
        ("null", "on"),
        ("[push, bogus]", "on[2]"),
        ("[push, 5]", "on[2]"),
        ("[push, [pull_request]]", "on[2]"),
        ("{bogus: null}", "on.bogus"),
        ("{create: {}}", "on.create"),
        ("{push: []}", "on.push"),
        ("{push: main}", "on.push"),
        ("{schedule: null}", "on.schedule"),
    ] {
        let error = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some(route)),
            "{on}: {error}"
        );
    }
}

// B-20: invalid mapping keys under `on` read like the scalar form, and keys inside dispatch
// bodies name the field.
#[test]
fn invalid_keys_render_by_what_they_are() {
    for (on, message) in [
        ("bogus", "on: unknown event 'bogus'"),
        ("{bogus: null}", "on: unknown event 'bogus'"),
        (
            "{push: null, \"x\\e\": null}",
            "on: unknown event 'x\\u{1b}'",
        ),
        (
            "{workflow_dispatch: {bogus: 1}}",
            "on.workflow_dispatch: unknown field `bogus`",
        ),
        (
            "{workflow_dispatch: {inputs: {a: {bogus: 1}}}}",
            "on.workflow_dispatch.inputs.a: unknown field `bogus`",
        ),
    ] {
        let error = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
        assert_eq!(error.category(), Category::Invalid, "{on}");
        assert_eq!(error.to_string(), message, "{on}");
    }
}

fn filter(name: &str, patterns: &[&str]) -> EventFilter {
    EventFilter {
        name: name.into(),
        patterns: patterns.iter().map(|&p| p.into()).collect(),
    }
}

// Backlog B-22: branch, tag and path filters are reported in source order, never matched.
#[test]
fn branch_tag_and_path_filters_are_reported() {
    assert_eq!(
        triggers(
            "{push: {branches: [main, 'releases/**'], tags-ignore: v1, paths: []}, pull_request: {paths-ignore: [docs/**], branches-ignore: main}, pull_request_target: {branches: main}}"
        ),
        [
            Trigger::Filtered {
                event: "push".into(),
                filters: vec![
                    filter("branches", &["main", "releases/**"]),
                    filter("tags-ignore", &["v1"]),
                    filter("paths", &[]),
                ],
            },
            Trigger::Filtered {
                event: "pull_request".into(),
                filters: vec![
                    filter("paths-ignore", &["docs/**"]),
                    filter("branches-ignore", &["main"]),
                ],
            },
            Trigger::Filtered {
                event: "pull_request_target".into(),
                filters: vec![filter("branches", &["main"])],
            },
        ]
    );
}

#[test]
fn filter_categories_follow_the_pinned_schema() {
    for (on, category, route) in [
        // `tags` and `tags-ignore` are absent from the pull request mappings.
        (
            "{pull_request: {tags: v1}}",
            Category::Invalid,
            "on.pull_request.tags",
        ),
        (
            "{pull_request: {tags-ignore: v1}}",
            Category::Invalid,
            "on.pull_request.tags-ignore",
        ),
        (
            "{pull_request_target: {tags: v1}}",
            Category::Invalid,
            "on.pull_request_target.tags",
        ),
        (
            "{pull_request_target: {tags-ignore: v1}}",
            Category::Invalid,
            "on.pull_request_target.tags-ignore",
        ),
        (
            "{push: {types: [created]}}",
            Category::Invalid,
            "on.push.types",
        ),
        ("{push: {bogus: main}}", Category::Invalid, "on.push.bogus"),
        // No expression context.
        (
            "{push: {branches: '${{ x }}'}}",
            Category::Invalid,
            "on.push.branches",
        ),
        (
            "{push: {branches: ['${{ x }}']}}",
            Category::Invalid,
            "on.push.branches[1]",
        ),
        (
            "{push: {branches: ''}}",
            Category::Invalid,
            "on.push.branches",
        ),
        (
            "{push: {branches: [main, '']}}",
            Category::Invalid,
            "on.push.branches[2]",
        ),
        (
            "{push: {branches: null}}",
            Category::Invalid,
            "on.push.branches",
        ),
        (
            "{push: {branches: [null]}}",
            Category::Invalid,
            "on.push.branches[1]",
        ),
        (
            "{push: {branches: {main: 1}}}",
            Category::Invalid,
            "on.push.branches",
        ),
        (
            "{push: {branches: [[main]]}}",
            Category::Invalid,
            "on.push.branches[1]",
        ),
        (
            "{push: {branches: main, branches-ignore: dev}}",
            Category::Invalid,
            "on.push.branches-ignore",
        ),
        (
            "{pull_request: {paths-ignore: a, paths: b}}",
            Category::Invalid,
            "on.pull_request.paths",
        ),
        // B-13: the reader converts these to non-empty strings.
        (
            "{push: {branches: 5}}",
            Category::Unsupported,
            "on.push.branches",
        ),
        (
            "{push: {tags: [true]}}",
            Category::Unsupported,
            "on.push.tags[1]",
        ),
        // B-36 contract change: `types` now inspect; an unknown activity type is invalid.
        (
            "{pull_request: {types: open}}",
            Category::Invalid,
            "on.pull_request.types",
        ),
    ] {
        let error = common::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (category, Some(route)),
            "{on}: {error}"
        );
    }
    assert_eq!(
        triggers("{push: {tags: v1}}"),
        [Trigger::Filtered {
            event: "push".into(),
            filters: vec![filter("tags", &["v1"])],
        }]
    );
}
