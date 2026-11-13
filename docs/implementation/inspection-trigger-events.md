# Trigger-event iteration: bare `on` events in source order (backlog B-15)

Status: **delivered on merge of its PR**, branch `feat/b15-trigger-events` from `c4472b5`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; interface-touching:
`wsr-types` `Trigger`, D-033); cross-reviewed by Codex gpt-6.1-sol (high): no blockers; one P2 fixed
(a valid cron in the Unsupported sample) and one P3 fixed (empty collections `Unsupported`, not
`Invalid`). The pinned parser uses `workflow-root-strict`. Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

`WorkflowSummary.trigger: Option<ManualTrigger>` becomes `triggers: Vec<Trigger>`, where `Trigger` is
`Manual(ManualTrigger)` or `Event(String)`, a D-033 temporary GHA-derived type next to
`ManualTrigger`. The host prints `Trigger: <event> (not evaluated)` per event in source order, with
dispatch inputs under the `workflow_dispatch` line. See the [CLI contract](../cli/README.md#trigger-events).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
(`workflow-root-strict` uses `on-strict`): `on-string-strict` and `on-sequence-strict` admit all 37
event names; `on-mapping-strict` maps each to a definition that is `null` or a mapping, `null` only
(`create`, `delete`, `deployment`, `deployment_status`, `fork`, `gollum`, `page_build`, `public`,
`status`), or a sequence (`schedule`). Hence:

- Bare names in a scalar or sequence, and null or `{}` bodies where the definition allows them, are
  reported. `{}` on a null-only event and null `schedule` are `Invalid`.
- Non-empty bodies (filters, `workflow_call` inputs/secrets/outputs) and any `schedule` sequence are
  `Unsupported`; their inner keys are not validated. A repeated sequence entry (the pinned parser
  accepts it) and an empty sequence or mapping (the strict schema sets no minimum; GitHub rejection
  is not established) are `Unsupported`.
- Unknown names (strict `on`), expressions (no context), non-string entries, `null` and other body
  shapes are `Invalid`; sequence diagnostics route to `on[n]`.

## Acceptance and expected red

`crates/wsr-gha/tests/triggers.rs` and three CLI tests (goldens, escaped diagnostics, temporary copies of
the repository's `check-docs`, `check-fmt`, `check-lint` and `test` workflows). Expected red:
`on: unsupported trigger; only workflow_dispatch is inspected yet`, `on: expected a mapping (other
trigger forms are unsupported)` and `on.workflow_call: unsupported field or event`.
