# Env and defaults iteration: workflow/job `env` and `defaults.run` (backlog B-11)

Status: **delivered on merge of its PR**, branch `feat/b11-env-defaults` from `5c6e284` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode). Evidence lives in the
[checkpoint](inspection-progress.md).

## Scope

Workflow and job `env` reuse the step `env` decoder (sorted keys, typed scalars, case-insensitive
duplicates, literal/unevaluated text). Workflow and job `defaults.run` `shell` and
`working-directory` are reported as `SourceText` in a new `RunDefaults` summary (D-033: temporary
GHA-derived summary types in `wsr-types`). Nothing is bound, merged across levels or applied to steps;
the host renders `Env (not bound)` and `Run defaults (not applied)` sections. See the
[CLI contract](../cli/README.md#workflow-and-job-environment-and-run-defaults).

Categories follow B-8 and the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json):

| Position | Definition | Expression context | Expression category |
| --- | --- | --- | --- |
| workflow `env` | `workflow-env` | github, inputs, vars, secrets | Unsupported (mapping), unevaluated (values) |
| job `env` | `job-env` | github, inputs, vars, needs, strategy, matrix, secrets | Unsupported (mapping), unevaluated (values) |
| workflow/job `defaults` | `workflow-defaults`, `job-defaults` | none | Invalid |
| workflow `defaults.run` and values | `workflow-defaults-run` | none | Invalid |
| job `defaults.run` and values | `job-defaults-run` | github, inputs, vars, strategy, matrix, needs, env | Unsupported (mapping), unevaluated (values) |

`shell` and `working-directory` require nonempty strings, so `''` is invalid. Whitespace-only text
follows the step precedent: a blank shell is invalid (rejected by the runner), a blank directory
unsupported. Unknown keys, non-mappings and non-string values are invalid.

## Acceptance and expected red

CLI tests for the full report (including hostile text and a no-execution sentinel) and 15 exact
diagnostics; `wsr-gha` tests for summary values and 17 category/route cases. Expected red:
``unknown field `env` `` (and `defaults`), previously reported as valid-but-unsupported schema keys. Intended contract change: the decoder's
``expected one of …`` list for an unknown job key now names `env` and `defaults`, so the exact message
in `crates/wsr-gha/tests/diagnostics.rs` (`typed_errors_keep_parser_locations_and_duplicate_key_evidence`)
was updated; its category, route and location assertions are unchanged.
