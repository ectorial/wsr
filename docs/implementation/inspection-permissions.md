# Permissions iteration: requested `GITHUB_TOKEN` permissions (backlog B-3)

Status: **delivered on merge of its PR**, branch `feat/inspect-permissions` from `319fc07` (Schedule A).
Implemented by Claude (claude-opus-5-5); cross-reviewed by Codex gpt-6.1-sol (high). Evidence lives in
the [checkpoint](inspection-progress.md).

## Scope

Accept workflow-level and job-level `permissions` and report them as *requested* source data.
Inspection grants nothing, computes no effective token, does not merge a job's request with the
workflow's and does not claim WSR enforces permissions.

Supported forms, per the
[pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
(`permissions`, `permissions-mapping`, `permission-level-*`):

- the scalar shorthands `read-all` and `write-all`;
- a mapping, possibly empty, from a schema scope to `read`, `write` or `none`. `id-token` and
  `copilot-requests` accept only `write`/`none`; `models` and `vulnerability-alerts` only
  `read`/`none`.

Output (scopes sorted by name; a job's request follows its runner and needs lines):

```text
Permissions: read-all (requested; not granted)
Permissions (requested; not granted):
  contents: read
  id-token: write
Permissions: {} (requested; not granted)
```

Rejected before stdout, as `Invalid` at the field route: null, other scalars, sequences, unknown
or non-string scopes (`permissions.foo`), levels outside the scope's set (`permissions.models`).
The pinned schema gives `permissions` no expression context, so `${{` text is reported as an
unevaluated expression that the field does not allow, also `Invalid`.

## Acceptance

- CLI: scalar, map, empty map, job request displayed alongside a workflow request, unknown scope,
  scope-restricted level, expression; output and docs never claim a grant.
- `wsr-gha`: summary structure for workflow/job permissions; category and route for rejections.
  The B-0 test that used `permissions` as an unsupported key moves to another unsupported key.

## Expected red

``unknown field `permissions` `` (`Unsupported`) for every accepted form.

Files: `crates/wsr-gha/src/{lib.rs,permissions.rs}`, `crates/wsr-types/src/inspection.rs`,
`crates/wsr/src/commands/inspect.rs`, `crates/wsr/tests/inspection.rs`, `crates/wsr-gha/tests/`,
`docs/cli/README.md`.
