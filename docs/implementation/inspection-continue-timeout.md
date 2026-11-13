# Continue-on-error iteration: job/step `continue-on-error`, step `timeout-minutes` (backlog B-17)

Status: **delivered on merge of its PR**, branch `feat/b17-continue-timeout` from `472887c`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; `wsr-types` fields
under D-033). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

`JobSummary.continue_on_error`, `StepSummary.continue_on_error` and `StepSummary.timeout_minutes`
(`SourceText`) are rendered with `(not applied)`. See the
[CLI contract](../cli/README.md#continue-on-error-and-step-timeouts).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
(L2308-2325): job `continue-on-error` is `boolean-strategy-context`, step `continue-on-error` is
`step-continue-on-error` (boolean) and step `timeout-minutes` is `step-timeout-minutes` (number), all
with expression contexts. A boolean is `Literal`; `${{` text is `Unevaluated`; anything else is
`Invalid`, since the reader's string conversion of a number or null cannot produce a boolean. The step
timeout reuses the job decoder (finite positive number; non-positive `Unsupported`).

## Acceptance and expected red

`crates/wsr-gha/tests/continue_timeout.rs` and two CLI tests, including rejection of
`continue-on-error: 5` and `timeout-minutes: true`. Expected red: ``unknown field `continue-on-error` ``
and `` `timeout-minutes` `` (`Unsupported`). Contract change: `diagnostics.rs` samples move to job
`cancel-timeout-minutes` and step `name: 5` (a converted scalar).
