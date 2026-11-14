# Case-insensitive identifier iteration: job and dispatch input IDs (backlog B-21)

Status: **delivered on merge of its PR**, branch `feat/b21-case-insensitive-ids` from `ba2c704`
(Schedule A). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

The pinned reader (`template-reader.ts` L208-216) rejects mapping keys repeated ignoring case. Job
identifiers (`crates/wsr-gha/src/lib.rs`, `unique_jobs`) and `workflow_dispatch` input identifiers
(`crates/wsr-gha/src/dispatch.rs`) now keep a map from the uppercased identifier to its first
spelling, as `step_inputs.rs` already does for `env`/`with` keys. A repeat is `Invalid` at
`jobs.{id}` or `on.workflow_dispatch.inputs.{id}` with
``duplicate job|input identifier 'Build'; already defined as 'build'``. Exact duplicates keep their
current diagnostics (the exact job check runs first; exact input keys are rejected by the decoder).

## Acceptance and expected red

`crates/wsr-gha/tests/compiler.rs` (`build`/`Build`), `crates/wsr-gha/tests/dispatch_inputs.rs`
(`mode`/`MODE`) and one row in the CLI identifier table of `crates/wsr/tests/inspection.rs`.
Expected red: both probes inspect with exit 0.
