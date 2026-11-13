# Inspection iteration: scalar conversion categories (backlog B-13, audit F21, F22, F23, F24)

Status: **delivered** in the Schedule B architecture PR `arch/review-2026-11-13-run3`
(baseline `main` `6f58e96`). Evidence lives in the [review log](architecture-review.md#2026-11-13-run-3).

## Behavior

The pinned reader ([template-reader.ts L411-445](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/templates/template-reader.ts))
converts an unmatched null, boolean or number to a string (a null to `""`) and validates that string
against the definition (null-token.ts L21-23 gives the empty string). WSR followed this unevenly:
`if`, step `env`/`with` numbers and `defaults.run` working directories were already `Unsupported`,
while `name`, `run`, `runs-on`, dispatch `description` and others reported the same conversions as
`Invalid`. Under the [B-8 rule](inspection-classification.md) a schema-valid value WSR does not inspect
is `Unsupported` (`non-string value is not inspected yet`, the existing message style), and a value the
schema or the runner rejects is `Invalid`. Each field is classified on its own; no shared helper changed
silently. Routes, parser locations, run/uses exclusivity, placement checks and all successful output are
unchanged, the converted text is never rendered, YAML `5` and `"5"` stay distinct, and a missing field
stays distinct from an explicit null.

### Target table

| Field | Value | Before | After | Reason |
| --- | --- | --- | --- | --- |
| workflow, job, step `name` | number, boolean | Invalid | Unsupported | converts to a valid string |
| workflow, job, step `name` | null | accepted as absent | accepted as absent | unchanged |
| `run-name` | number, boolean, null | Invalid | Unsupported | converts to a string the schema accepts |
| step `run` | number, boolean, null | Invalid | Unsupported | `string-steps-context` allows any string |
| step `working-directory` | number, boolean, null | Invalid | Unsupported | `string-steps-context` allows any string |
| step `shell` | number, boolean, null | Invalid | Invalid | runner rejects a bare shell that is not built in and has no `{0}` |
| `defaults.run.shell` (workflow, job) | number, boolean | Unsupported | **Invalid** | same runner reason as step `shell`; B-11 predated this |
| `defaults.run.shell` | null | Invalid | Invalid | nonempty required |
| `defaults.run.working-directory` | number, boolean | Unsupported | Unsupported | unchanged |
| `defaults.run.working-directory` | null | Invalid | Invalid | nonempty required |
| `runs-on` scalar or list entry | number, boolean | Invalid | Unsupported | converts to a label |
| `runs-on` | null (scalar or entry) | Invalid | Invalid | empty label |
| runner selector `group`, `labels` (scalar), label entry | number, boolean | Invalid | Unsupported | the whole selector is already Unsupported when well formed |
| runner selector `group`, `labels`, entry | null | Invalid | Invalid | empty label |
| dispatch input `description` | number, boolean, null | Invalid | Unsupported | converts to a string |
| choice-input `options` entry | number, boolean, null | Invalid | Unsupported | converts to a string |
| dispatch input `type` | number, boolean, null | Invalid | Invalid | not a member of the type enumeration |
| dispatch input `required` | number, null, `"true"` | Invalid | Invalid | must be a boolean |
| string, choice, environment input `default` | number, boolean | Invalid | Invalid | type mismatch |
| string, choice, environment, number input `default` | null | Invalid | Unsupported | converts to `""`, which these inputs hold |
| boolean input `default` | any non-boolean | Invalid | Invalid | type mismatch |
| workflow, job, step `env` value | null | Invalid | Unsupported | converts to `""`; number and boolean stay accepted with their source type |
| action-step `with` value | null | Invalid | Unsupported | as `env` |
| job, step `if` | number, boolean, null | Unsupported | Unsupported | unchanged |
| step `id` | `true`, `false` | Invalid | Unsupported | converts to a valid identifier |
| step `id` | number, null | Invalid | Invalid | `5` starts with a digit; null is empty |
| job ID | number, null | Invalid | Invalid | unchanged |
| step `uses` | number, boolean, null | Invalid | Invalid | unchanged |
| job `timeout-minutes` | `"5"` | Invalid | Invalid | unchanged |

Sequences and mappings where a scalar is expected stay `Invalid` everywhere.

### CLI text (F23, F24)

`Condition:` lines (job and step) now end `(not evaluated; expression syntax not validated)`, since any
text is accepted. The goldens in `crates/wsr/tests/inspection.rs` changed accordingly; the dogfood
goldens contain no condition and stay byte-identical. The CLI guide's claim that success has empty
stderr now excepts the 500 KB size warning (F24), and its "numbers and booleans are rejected" sentence
describes these categories. The CLI shows no category, only the message and exit 1, so the visible
change is the message text for the Unsupported rows.

## Evidence

- Red (baseline `6f58e96` plus the new and updated tests): `cargo test -p wsr-gha -p wsr --no-fail-fast`
  failed 8 of 12 tests in the new `crates/wsr-gha/tests/scalar_conversion.rs` on assertions, for example
  `"jobs:\n  b:\n    runs-on: 5 …": expected Unsupported at Some("jobs.b.runs-on"), got Invalid at Some("jobs.b.runs-on")`,
  `step_id_booleans_convert_to_identifiers_but_numbers_and_null_stay_invalid` (`id: true` got Invalid),
  `run_defaults_shell_conversions_are_invalid_and_directory_conversions_unsupported`
  (`defaults: {run: {shell: 5}}` expected Invalid, got Unsupported) and the dispatch, `env`/`with`, `name`
  and step `run`/`working-directory` cases. The updated `diagnostics.rs`, `env_defaults.rs` and
  `step_inputs.rs` cases failed the same way, and 8 CLI inspection tests failed on message text or on the
  new `Condition:` suffix. Four new tests were green characterization of rows that stay as they are
  (step `shell`, dispatch `type`/`required`, `if`/`uses`/job ID/timeout, quoted scalars).
- Green: `cargo test -p wsr-gha -p wsr`, then `cargo xtask ci` (format, locked Clippy with `-D warnings`,
  workspace tests including the dogfood goldens, strict rustdoc).
- Contract changes to existing assertions, all citing B-13: `diagnostics.rs` (workflow `name: 5` and
  `runs-on: {group: 1}` move to the Unsupported list, a `{group: ~}` case keeps Invalid), `env_defaults.rs`
  (`env: {A: ~}` Unsupported, `defaults.run.shell: true` Invalid), `step_inputs.rs` (`env: {A: ~}`
  Unsupported), and CLI message text in `crates/wsr/tests/inspection.rs`.

## Upstream references

All pinned to actions/languageservices `4043eda158e16579cc5fb1b0b07a4bce2a76f0b5` unless noted:

- `template-reader.ts` L411-445: unmatched null, boolean and number are converted to strings and validated.
- `null-token.ts` L21-23: a null token's string form is empty.
- `workflow-v1.0.json` L2729-2739 (`shell`, `working-directory` in `defaults.run`), L1732-1738
  (`defaults`), L2180-2193 (step definition) and L1540-1578 (`workflow-dispatch-input`).
- actions/runner `8cfa1049f86781d141056082d73802d56cd2b433`, `ScriptHandler.cs` L224-245: a `shell` that
  is neither built in nor contains `{0}` is rejected at run time.

## Limitations

- Boolean mapping keys are kept as before and unverified: the upstream YAML object reader appears to
  mishandle them (a defect), and WSR does not model the conversion there.
- A converted value may fail a later schema check that WSR does not run (for example a number-like
  `runs-on` label); `Unsupported` means only that inspection does not read it.
- Quoted `shell: "true"` and `uses: "5"` are still accepted although the runner rejects them; that is
  audit F28, backlog B-14.
