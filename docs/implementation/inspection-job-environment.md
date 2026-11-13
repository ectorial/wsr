# Job-environment iteration: job `environment` (backlog B-19)

Status: **delivered on merge of its PR**, branch `feat/b19-job-environment` from `e39fe9e`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; `wsr-types`
`Environment` under D-033); cross-reviewed by Codex gpt-6.1-sol (high): no findings. Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

New module `crates/wsr-gha/src/environment.rs` decodes job `environment` into
`JobSummary.environment: Option<Environment { name, url, deployment }>` of `SourceText` values, rendered
as `Environment (not selected; no protection rules evaluated):`. The concurrency and environment blocks
share one host helper. See the [CLI contract](../cli/README.md#job-environments).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
`job-environment` (context `github, inputs, vars, needs, strategy, matrix`) is `string |
job-environment-mapping`, with `name` (`job-environment-name`, required string), `url`
(`string-runner-context-no-secrets`) and `deployment` (`boolean`). The pinned converter
(`model/converter/job/environment.ts`) asserts string keys, skips expression values and asserts a
boolean `deployment`. Hence:

- `${{` text in any value is `Unevaluated`; an expression key is `Invalid` (as for B-16).
- Empty or blank text and B-13 conversions (number, boolean, null) of the string form, `name` or `url`
  are `Unsupported`: the schema accepts any string.
- Unknown keys (routed to the key), a missing `name`, a non-boolean `deployment` and other shapes are
  `Invalid`.

## Acceptance and expected red

`crates/wsr-gha/tests/environment.rs` and two CLI tests (hostile-text golden with an `executed`
sentinel; five exact diagnostics). Expected red: ``jobs.b: unknown field `environment` ``.
