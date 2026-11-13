# Dogfood-guard iteration: every example workflow has a golden case (backlog B-10, audit F20)

Status: **delivered on merge of its PR**, branch `test/b10-dogfood-set` from `fb31d1c` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol
(high): no blockers; two P3s fixed (only regular files count; non-UTF-8 names are skipped, not a panic). Evidence lives in the
[checkpoint](inspection-progress.md).

## Scope

`crates/wsr/tests/dogfood.rs` held its golden table inline in the B-6 test, so a new
`.github/workflows/example-*.yml` could land without golden inspection coverage. The table is now a
`CASES` constant; a guard test asserts the sorted `example-*.yml` names equal its keys. A second test
copies the workflows into a temporary directory, adds `example-extra.yml` there (never in the repo)
and asserts the guard reports the mismatch. Test-only; no product or CLI change.

## Acceptance and expected red

The guard passes on the current two examples and fails on the temporary copy with an extra example.
Expected red: none (characterization guard), proven by the temp-copy fixture.
