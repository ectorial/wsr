# Job-outputs iteration: job `outputs` (backlog B-18)

Status: **delivered on merge of its PR**, branch `feat/b18-job-outputs` from `5ec43bf` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; `wsr-types` field under D-033).
Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

New module `crates/wsr-gha/src/outputs.rs` decodes job `outputs` into
`JobSummary.outputs: Option<BTreeMap<String, SourceText>>`, rendered sorted as
`Outputs (not resolved):`. See the [CLI contract](../cli/README.md#job-outputs).

Per the [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
`job-outputs` is a loose mapping (`non-empty-string` keys, `string-runner-context` values) with no
context of its own. The pinned reader's loose-mapping handler rejects expression keys there, rejects
keys repeated ignoring case, and converts scalar keys and values to strings. Hence a whole-mapping or
key expression, empty and duplicate keys, non-mappings and nested values are `Invalid`; number,
boolean and null values and non-string keys are `Unsupported` (B-13); `${{` values are `Unevaluated`.
The converter only asserts a mapping.

## Acceptance and expected red

`crates/wsr-gha/tests/outputs.rs` and two CLI tests (hostile-text golden with an `executed` sentinel;
four exact diagnostics). Expected red: ``jobs.b: unknown field `outputs` ``.
