# Concurrency iteration: workflow and job `concurrency` (backlog B-16)

Status: **delivered on merge of its PR**, branch `feat/b16-concurrency` from `e95b0d4` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; interface-touching: `wsr-types`
`Concurrency`, D-033). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

New module `crates/wsr-gha/src/concurrency.rs` decodes workflow and job `concurrency` into
`Concurrency { group, cancel_in_progress, queue }` of `SourceText` values, rendered as a
`Concurrency (not applied):` block. See the [CLI contract](../cli/README.md#workflow-and-job-concurrency).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json):
`workflow-concurrency` is `string | concurrency-mapping` with context `github, inputs, vars`;
`job-concurrency` is `non-empty-string | concurrency-mapping` with the strategy contexts too.
`concurrency-mapping` has required `group` (`non-empty-string`), `cancel-in-progress` (`boolean`) and
`queue` (`concurrency-queue`: `single` or `max`). Hence:

- `${{` text in any value is `Unevaluated`; an expression key is `Unsupported` (B-11 precedent).
- B-13 conversions: a number or boolean group converts to a valid string, so `Unsupported`; a null
  group converts to an empty string, which is `Invalid` where the definition is non-empty (job
  string, `group`) and `Unsupported` for the workflow string. Blank non-empty text is schema-valid and
  `Unsupported`, as for `run`.
- Non-boolean `cancel-in-progress` (the string `'true'`, numbers, null), other `queue` values, unknown
  keys (routed to the key, shown at the mapping), a missing `group` and other shapes are `Invalid`.

## Acceptance and expected red

`crates/wsr-gha/tests/concurrency.rs` and two CLI tests (two-level golden with hostile text and an
`executed` sentinel; six exact diagnostics). Expected red: ``unknown field `concurrency` `` at both levels.
