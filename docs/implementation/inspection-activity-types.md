# Pull request activity types iteration (backlog B-36)

Scope: `types` under `pull_request` and `pull_request_target` is retained in source order as a
`types` entry of the existing B-22 `Filters (not matched):` block (`types: ["opened", "closed"]`),
beside branch and path filters. Activity types are never matched against an event. No new type:
`EventFilter` (D-033) carries the names.

Pinned basis (actions/languageservices `4043eda158e16579cc5fb1b0b07a4bce2a76f0b5`):
[schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
`pull-request-activity` and `pull-request-target-activity` are one activity type or a sequence of
them, with no expression context. Each activity-type definition lists 22 `allowed-values`, which
[template-schema.ts](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/templates/schema/template-schema.ts)
turns into case-sensitive string constants (`ignoreCase` defaults to false). The events converter
(`model/converter/events.ts`) keeps them as a string list.

| Source | Category |
| --- | --- |
| an allowed name, or a sequence of them (order and repetitions kept) | Inspected |
| `types: []` | Inspected (the sequence has no minimum) |
| another name, a different case, or `${{` text | Invalid (no matching constant; no expression context) |
| number, boolean, null, nested list or mapping | Invalid (a converted number or boolean matches no constant) |
| `types` under `push` | Invalid (B-22; `push-mapping` has no `types`) |

Acceptance: `crates/wsr-gha/tests/activity_types.rs` and a CLI golden with hostile text, the
`executed` sentinel and the positive-controlled PATH trap.

Contract change: `triggers.rs` (two rows) and the CLI event-filter table used `types` as their
Unsupported sample; it is now inspected, so those rows assert Invalid samples.

Expected red (live probe): ``on.pull_request.types: unsupported field `types`; this field is not inspected yet``.

## Evidence

- Red (observed): `activity_types.rs` failed with ``on.pull_request.types: unsupported field
  `types`; this field is not inspected yet`` (Unsupported) for both the retention and category tests.
- Green: `activity_types.rs` (2 tests; all 22 names for both events, 12 invalid shapes for each event),
  `triggers.rs`, the CLI suite (golden with hostile text in a neighbouring branch filter and in a
  rejected type, `executed` sentinel and PATH traps), `cargo xtask ci`, `git diff --check`. The CLI
  golden was written after the implementation, so its red was observed only at the `wsr-gha` level.
