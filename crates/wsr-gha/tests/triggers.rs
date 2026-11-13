//! Backlog B-15: non-dispatch `on:` events with empty bodies are reported in source order.

use wsr_gha::Category;
use wsr_types::inspection::{DispatchBody, ManualTrigger, Trigger};

const JOBS: &str = "jobs:\n  b:\n    runs-on: linux\n    steps:\n      - run: echo\n";

fn triggers(on: &str) -> Vec<Trigger> {
    wsr_gha::inspect(format!("on: {on}\n{JOBS}").as_bytes())
        .unwrap_or_else(|error| panic!("{on}: {error}"))
        .triggers
}

fn event(name: &str) -> Trigger {
    Trigger::Event(name.into())
}

#[test]
fn events_are_kept_in_source_order() {
    assert_eq!(wsr_gha::inspect(JOBS.as_bytes()).unwrap().triggers, []);
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
        ("{push: {branches: [main]}}", "on.push"),
        ("{workflow_call: {inputs: {}}}", "on.workflow_call"),
        (
            "{workflow_call: {secrets: {token: {required: true}}}}",
            "on.workflow_call",
        ),
        ("{schedule: [{cron: '0 0 * * *'}]}", "on.schedule"),
        ("{schedule: []}", "on.schedule"),
        ("[push, push]", "on[2]"),
    ] {
        let error = wsr_gha::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
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
        ("[]", "on"),
        ("{}", "on"),
        ("[push, bogus]", "on[2]"),
        ("[push, 5]", "on[2]"),
        ("[push, [pull_request]]", "on[2]"),
        ("{bogus: null}", "on.bogus"),
        ("{create: {}}", "on.create"),
        ("{push: []}", "on.push"),
        ("{push: main}", "on.push"),
        ("{schedule: null}", "on.schedule"),
    ] {
        let error = wsr_gha::inspect(format!("on: {on}\n{JOBS}").as_bytes()).unwrap_err();
        assert_eq!(
            (error.category(), error.route()),
            (Category::Invalid, Some(route)),
            "{on}: {error}"
        );
    }
}
