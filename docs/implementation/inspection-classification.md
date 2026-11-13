# Inspection iteration: expression category by schema context (backlog B-8, audit F13, F14, F16)

Status: **delivered** in the Schedule B architecture PR `arch/review-2026-11-13-run2`
(baseline `main` `e3a8373`). Evidence lives in the [review log](architecture-review.md#2026-11-13-run-2).

## Behavior

The inspection fence still treats any `${{` in a literal-only field as unevaluated; it does no
expression parsing. What changes is the category, which now follows the
[pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json):
a definition with a `context` list allows expressions, so WSR reports `Unsupported` (valid GHA it
does not evaluate); a definition without one allows none, so the same text is `Invalid`. For F13 and
F14, CLI stderr text and exit codes are unchanged; only `wsr_gha::Category` differs. Call sites pass the category
explicitly (`check_literal`/`check_required`); the helper has no default.

| Field | Pinned definition | Expression context | Category |
| --- | --- | --- | --- |
| workflow `name` | `workflow-name` | none | Invalid |
| `on` (scalar, event keys) | `on` | none | Invalid |
| job ID | `job-id` | none | Invalid |
| `needs` (scalar and list entries) | `needs` | none | Invalid |
| step `id` | `step-id` | none | Invalid |
| step `uses` | `step-uses` | none | Invalid |
| dispatch input ID | `workflow-dispatch-input-name` | none | Invalid |
| dispatch input `description`, `type`, `required`, `default`, `options` | `workflow-dispatch-input` | none | Invalid |
| job `name` | `string-strategy-context` | github, inputs, vars, needs, strategy, matrix | Unsupported |
| step `name` | `step-name` | steps-level contexts | Unsupported |
| step `run` | `string-steps-context` | steps-level contexts | Unsupported |
| `runs-on` and labels | `runs-on` | github, inputs, vars, needs, strategy, matrix | Unsupported |

This reverses the acceptance assertions in `crates/wsr-gha/tests/diagnostics.rs` (workflow `name`, `on`)
and `compiler.rs` (job ID, step ID), which came from erroneous B-0 backlog text. It is a contract
correction, not a weakened test.

### Schema-valid values that were reported Invalid (F14)

Reclassified to `Unsupported` with message, route and exit unchanged: job `timeout-minutes` zero or
negative (`number-strategy-context` has no minimum), step `working-directory: ''` and `run: ''`
(`string-steps-context` allows empty strings). A blank `shell` stays `Invalid`: the runner rejects it
semantically. Non-finite numbers and non-numbers in `timeout-minutes` stay `Invalid`.
`Category::Invalid` is now documented as invalid per the pinned schema or semantically rejected by
GitHub, and `Unsupported` as valid GHA outside WSR's inspection subset.

### Forged routes (F16)

`unique_jobs` decoded a job body before validating its ID, so job ID `a.steps[9]` with an unknown field
reported `jobs.a.steps[9].bogus`, a route that names a different location. The ID is now validated before
its body is decoded and reports `jobs.a.steps[9]: invalid job identifier 'a.steps[9]'`, with the
identifier diagnostic's existing text (no parser location suffix). This is an intentional precedence
change: a source with both an invalid job ID and a malformed body in that job now reports the
identifier, not the body error, so stderr differs from `e3a8373` for that combination only. The
exact-stderr CLI case in `crates/wsr/tests/inspection.rs` pins it; on `e3a8373` it fails, reporting
``jobs.a.steps[9]: unknown field `bogus`, … at line 3 column 5``.

## Evidence

- Red (baseline `e3a8373` plus the new tests): `expressions_in_fields_without_a_schema_expression_context_are_invalid_at_their_route`
  got `Unsupported` for workflow `name`; `a_job_identifier_is_validated_before_its_body_so_it_cannot_forge_a_route`
  got route `jobs.a.steps[9].bogus`; `non_positive_timeouts_are_schema_valid_but_unsupported` and
  `empty_run_and_working_directory_are_schema_valid_but_unsupported` got `Invalid`. The
  with-context group (`..._are_unsupported_at_their_route`) and the blank-`shell` assertion were green
  characterization.
- Green: `cargo test -p wsr-gha` and the `-p wsr` CLI tests (unchanged except the F16 stderr case), then `cargo xtask ci`.

## Limitations

- The pinned reader folds a sole quoted-string literal expression (such as `${{ 'x' }}`) into a plain
  literal. WSR's `${{` fence does not model that and still treats the text as an expression. Documented, not
  fixed; no expression parsing is added.
- Invalid-versus-unsupported for a new field is decided by its schema expression context
  (the B-8 rule); fields not yet inspected keep their existing classification.
