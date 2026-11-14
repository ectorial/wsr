# Name and runner expression iteration: job/step `name` and `runs-on` (backlog B-24)

Status: **delivered on merge of its PR**, branch `feat/b24-name-runs-on-expressions` from `4136fab`
(Schedule A). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

`JobSummary.name`, `StepSummary.name` and `RunnerRequirement::{Label, Labels}` now hold D-033
`SourceText`: text with `${{` is `Unevaluated`, other text `Literal`. The former `Unsupported`
rejection of expressions in these fields is removed; the host appends
`unevaluated; expression syntax not validated` to the name in parentheses, to a scalar runner, and
`(some unevaluated; …)` after a label list. See the
[CLI contract](../cli/README.md#literal-runner-labels).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
job `name` is `string-strategy-context` (also in `workflow-job`), step `name` is `step-name` and
`runs-on` has the `runs-on` context, so these are valid GHA that WSR keeps without evaluating. The
workflow `name` (no context) stays `Invalid`; blank labels stay `Invalid`; number/boolean conversions
stay `Unsupported` (B-13); runner-selector mappings stay `Unsupported`. No runner is selected.

## Acceptance and expected red

`crates/wsr-gha/tests/name_runner_expressions.rs` and a CLI golden with hostile text and an
`executed` sentinel. Expected red: `expression is unevaluated; dynamic inspection is not supported yet`
(`Unsupported`) at `jobs.a.name`.
