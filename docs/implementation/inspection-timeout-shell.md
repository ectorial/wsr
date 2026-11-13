# Timeout and shell iteration: job `timeout-minutes` and step `shell` (backlog B-4)

Status: **delivered on merge of its PR**, branch `feat/inspect-timeout-shell` from `40f1354` (Schedule A).
Implemented by Claude (claude-opus-5-5). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

Report job `timeout-minutes` and run-step `shell` as source data. Inspection never applies a
timeout, selects or invokes a shell, or claims either is supported at runtime.

Per the [pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json):

- job `timeout-minutes` is `number-strategy-context`: a number, or an expression over
  `github`, `inputs`, `vars`, `needs`, `strategy` and `matrix`. The schema sets no minimum or
  integer restriction; requiring a finite positive number is this subset's own rule. Such a number is
  reported as `Literal` text from the number value; text containing `${{` is `Unevaluated`, as for
  `run-name`. Booleans, strings, null, sequences, zero, negatives and non-finite numbers are
  `Invalid` (`expected a positive number`).
- `shell` is a nonempty string on `run-step` only and has no expression context. Expressions,
  blank text, non-strings and `shell` on a `uses` step are `Invalid`, checked after the
  existing exactly-one-of-`run`/`uses` rule.
Cross-reviewed by Codex gpt-6.1-sol (high): no blockers; two nits fixed (shell-check precedence,
positivity attributed to this subset rather than the schema).

Job `cancel-timeout-minutes` remains unsupported; step `timeout-minutes` was added by
[B-17](inspection-continue-timeout.md).

```text
  Timeout minutes: 5 (not applied)
  Timeout minutes: ${{ inputs.minutes }} (unevaluated; expression syntax not validated; not applied)
  Step 1: run
    Shell: bash (not selected)
```

## Acceptance and expected red

CLI tests for literal (integer and fractional), expression and escaped hostile shell text, and
exact-stderr rejections; `wsr-gha` tests for summary structure and category/route. Expected red:
``unknown field `timeout-minutes` `` / `` `shell` ``.
