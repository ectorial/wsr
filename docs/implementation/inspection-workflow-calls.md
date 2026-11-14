# Workflow-call iteration: reusable-workflow call jobs (backlog B-23)

Status: **delivered on merge of its PR**, branch `feat/b23-reusable-calls` from `568793f`
(Schedule A). Implemented by Claude
(claude-opus-5-5, burn mode; `wsr-types` call types under D-033); cross-reviewed by Codex gpt-6.1-sol
(high) in four rounds: placement by key order and eager decoding (two P2s), then a materializing
pre-scan (three P2s), then scan amplification and a lost call form (two P2s), then job-body alias
amplification (one P2); all fixed with regression tests or measured probes. Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

New module `crates/wsr-gha/src/call.rs` decodes a job with `uses` into
`JobSummary.call: Option<WorkflowCall { uses, with, secrets }>` (D-033 temporary GHA-derived types,
with `CallSecrets::{Inherit, Values}`); such a job has `runner: None` and no steps. The host renders
`Calls workflow: <uses> (not loaded; not resolved)`, `With (not passed):` and `Secrets`. See the
[CLI contract](../cli/README.md#reusable-workflow-calls).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
`job` is one of `job-factory` and `workflow-job`; `workflow-job` has `name`, `uses` (required
`non-empty-string`), `with` (`workflow-job-with`), `secrets` (mapping or the constant `inherit`),
`needs`, `if`, `permissions`, `concurrency` and `strategy`. The pinned converter (`model/converter/job.ts`)
selects a reusable-workflow job when `uses` is present; `workflows/file-reference.ts`
`parseFileReference` accepts `./…`, `$/…` and `owner/repo[/path]@ref`. Hence:

- keys only `job-factory` has (`runs-on`, `timeout-minutes`, `cancel-timeout-minutes`,
  `continue-on-error`, `container`, `services`, `env`, `environment`, `outputs`, `defaults`, `steps`,
  `snapshot`) with `uses`, and `with`/`secrets` without it, are `Invalid` when the key is read,
  whatever the key order and before the value is decoded: `call::scan` streams the source once,
  skipping values with `IgnoredAny` (never materialized) and reading keys as the typed decoder does,
  to record each job's form by position; `diagnostic::Level::placement` then rejects the key with
  its parser location. The scan reads only the first `jobs` and stops after one job key per source
  byte (aliases can repeat job bodies); a job it did not finish without reading `uses` falls back to
  post-decode checks;
- `uses` with `${{`, empty or not a reference is `Invalid`; `$/` is `Unsupported` (as for step `uses`);
- `with` and a `secrets` mapping reuse the step `with` decoder; an expression replacing the mapping or
  in a key is `Invalid` (the definitions have no context); `secrets` other than `inherit` or a mapping
  is `Invalid`; `strategy` stays `Unsupported`.

`runs-on` and `steps` became optional on the decoded job; a `try_from` wrapper keeps the derived
``missing field `runs-on` `` / `` `steps` `` wording and parser location for ordinary jobs.
Nothing reads the referenced file.

## Acceptance and expected red

`crates/wsr-gha/tests/calls.rs` and two CLI tests (golden with a hostile input, an existing target
with a `touch executed` step and a missing target, both never opened; exact diagnostics); a temporary
`ci.yml` copy exits 0 under the PATH traps. Expected red:
``jobs.check-fmt: unsupported field `uses`; this field is not inspected yet``.
