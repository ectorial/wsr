# Run expression iteration: `${{` in step `run` (backlog B-28)

Status: **delivered on merge of its PR**, branch `feat/b28-run-expressions` from `e4a7ac8`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, steady mode); cross-reviewed by
Codex gpt-6.1-sol (high): no code findings; one P3 fixed (the B-8 classification table still listed
`run`, and the B-24 fields, as `Unsupported`). Evidence lives in the [checkpoint](inspection-progress.md).

Contract change: `diagnostics.rs` used `run: ${{ x }}` as its expression-context `Unsupported`
sample; it now uses a whole step `env` mapping given as an expression (`jobs.b.steps[1].env`).

## Scope

The [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
types step `run` as `string-steps-context`, so `${{` text in a script is valid GHA that WSR keeps
without evaluating. `StepSummaryKind::Run` gains an `unevaluated` flag (D-033 `wsr-types` field);
the former `Unsupported` rejection is removed. The script is still never printed or retained: the
CLI heading reads `Step N: run (script has unevaluated expressions; not evaluated)`, before any step
name. Empty and blank scripts stay `Unsupported`; scalar conversions stay `Unsupported` (B-13). No
expression is evaluated, no context is bound and no command is executed.

Audit F40: the repository-workflow test asserts that its case names equal the sorted
`.github/workflows/*.yml` names minus `example-*`, so a new workflow cannot go unpinned.

## Acceptance and expected red

- `crates/wsr-gha/tests/run_expressions.rs`: the flag is set for `${{` text and clear otherwise.
- A CLI golden with hostile text in the script (`$(touch executed)`, escape and bidi characters)
  proving the script is not printed, and the `executed` sentinel.
- The repository-workflow case-name assertion.

Expected red: ``jobs.a.steps[1].run: expression is unevaluated; dynamic inspection is not supported
yet`` (`Unsupported`).
