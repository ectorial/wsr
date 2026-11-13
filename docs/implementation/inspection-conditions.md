# Conditions iteration: job and step `if` as unevaluated text (backlog B-12)

Status: **delivered on merge of its PR**, branch `feat/b12-if-conditions` from `b686358` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol
(high): no blockers; one P2 fixed (a null condition is schema-valid, so `Unsupported`, not `Invalid`). Evidence lives in the
[checkpoint](inspection-progress.md).

## Scope

Job and step `if` were rejected as valid-but-unsupported schema keys. A string condition is now kept
as written in a new `condition` field of the job and step summaries (D-033 temporary GHA-derived
types) and rendered as `Condition: <text> (not evaluated)` with `escape_debug` escaping. Bare and
`${{ }}`-fenced text are treated alike: the pinned
[`job-if`/`step-if`](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
are strings with expression contexts, implicitly expressions. No expression parsing or evaluation;
a condition never changes which jobs or steps are reported. See the
[CLI contract](../cli/README.md#unevaluated-job-and-step-conditions).

Numbers and booleans (converted to strings by the pinned reader) and null, empty or blank conditions
(admitted by the pinned reader and defaulted by GitHub to `success()`) are `Unsupported`, following
B-11's defaults rule and the empty-`run` precedent; sequences and mappings are `Invalid`. Decoder diagnostics keep parser locations.

## Acceptance and expected red

CLI tests for bare, fenced and hostile text with a no-execution sentinel, and six exact diagnostics;
`wsr-gha` tests for kept text and category/route. Expected red: the existing Unsupported ``unknown
field `if` `` diagnostic. Contract change: the B-0 job-`if` Unsupported example is replaced by
`continue-on-error`, and decoder "expected one of" lists now include `if`.
