# Event-filter iteration: `push` and pull request filters (backlog B-22)

Status: **delivered on merge of its PR**, branch `feat/b22-event-filters` from `7c2225e`
(Schedule A). Implemented by Claude
(claude-opus-5-5, burn mode; `wsr-types` variant under D-033); cross-reviewed by Codex gpt-6.1-sol
(high): no findings. Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

`crates/wsr-gha/src/dispatch.rs` decodes non-empty `push`, `pull_request` and `pull_request_target`
mappings into `Trigger::Filtered { event, filters: Vec<EventFilter { name, patterns }> }` (D-033
temporary GHA-derived types in `wsr-types`), in source order; a single string is one pattern. The host
renders `Filters (not matched):` with one `name: [patterns]` line per filter. See the
[CLI contract](../cli/README.md#event-filters).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
(`push-mapping`, `pull-request-mapping`, `pull-request-target-mapping`, `event-*`): `push` allows
`branches`, `branches-ignore`, `tags`, `tags-ignore`, `paths`, `paths-ignore`; the pull request
events allow `types`, `branches`, `branches-ignore`, `paths`, `paths-ignore`. Each filter is
`non-empty-string` or `sequence-of-non-empty-string` with no expression context. Hence:

- keys absent from the event's mapping, `${{` text, empty strings, null, mappings, nested sequences
  and a filter given with its `-ignore` counterpart are `Invalid`;
- numbers and booleans (B-13 conversions) and pull request `types` are `Unsupported`;
- an empty sequence is reported as `[]`.

The pair rejection follows the schema descriptions ("You cannot use both the `branches` and
`branches-ignore` filters for the same event"); the pinned converter (`model/converter/events.ts`
L86-115) keeps both without rejecting them, so this is a documented-GitHub rule, not a reader one.

Audit F33: the repository-workflow test in `crates/wsr/tests/inspection.rs` traps every PATH command
with the dogfood shim and an `executed` marker and pins the first diagnostic of temporary `ci.yml`
(`uses`, B-23) and `release.yml` (`strategy`) copies.

## Acceptance and expected red

`crates/wsr-gha/tests/triggers.rs` (forms and categories, including `tags`/`tags-ignore` rejected under
both pull request events and accepted under `push`) and two CLI tests (hostile golden with sentinel;
exact diagnostics). Expected red: ``on.push: event configuration is not inspected yet``.
