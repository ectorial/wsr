# Step-input iteration: step `env`, `with` and `working-directory` (backlog B-5)

Status: **delivered on merge of its PR**, branch `feat/inspect-step-inputs` from `85a1c30` (Schedule A).
Implemented by Claude (claude-opus-5-5); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
two P2s and a nit fixed (see below). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

Report step `env`, `with` and `working-directory` as source values. Inspection never binds an
environment variable, passes an input to an action, resolves or creates a directory, or evaluates
an expression.

Per the [pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json):

- `step-env` (any step) and `step-with` (`regular-step`, that is `uses`, only) are mappings with
  `non-empty-string` keys and loose `string` values, with an expression context. YAML strings,
  numbers and booleans are therefore accepted and keep their type (`persist-credentials: false`
  in the canonical example is a boolean). Strings containing `${{` are unevaluated. Keys are sorted.
- `working-directory` is `string-steps-context` on `run-step` only: literal or unevaluated text.

`env` and `with` are interpreted after the existing run/uses and `shell` checks, so a misplaced
`with` on a `run` step is `Invalid` whatever its content. Rejected before stdout: a non-mapping
`env`/`with` (`Invalid`); a whole-mapping expression, an expression in a key, or a number/boolean
key, which the [pinned parser](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/templates/template-reader.ts)
converts to a string (`Unsupported`); empty or null keys; null, sequence or mapping values; keys
duplicated ignoring case, as the pinned parser rejects them (`env.MODE: duplicate key 'MODE';
already defined as 'Mode'`), while exact YAML duplicates keep decoder evidence without a route;
`with` on a `run` step; `working-directory` on a `uses` step; and a non-string or blank `working-directory`.
Null values and blank directories are rejected by this subset's rules; whether GitHub coerces them
is not claimed.

```text
  Step 1: uses actions/checkout@…
    With (not passed to any action):
      persist-credentials: false
  Step 3: run (…)
    Working directory: examples/python-basic (not resolved)
    Env (not bound):
      WSR_EXAMPLE_MODE: "${{ inputs.mode }}" (unevaluated)
```

## Acceptance and expected red

CLI tests for typed values, expressions, empty mappings, a hostile `$(touch executed)` plus ESC
value with a sentinel check, and exact-stderr rejections; `wsr-gha` tests for summary structure,
category/route and duplicate keys. Expected red: ``unknown field `with` `` / `` `env` `` /
`` `working-directory` ``. After this slice both canonical `example-*.yml` workflows inspect
successfully; their golden acceptance is B-6.
