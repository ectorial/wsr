# Inspection iteration: consistent diagnostic routes and validation order (backlog B-1)

Status: **in progress** on `fix/inspect-diagnostic-routes` (Schedule A, [backlog B-1](backlog.md), audit F3, F5).
Baseline `main`: `d9712ec`. Evidence lives in the [checkpoint](inspection-progress.md).

## Developer outcome

`wsr inspect` diagnostics render one route grammar, `jobs.<id>.<field>[<n>]` with one-based list
indices, for both YAML-decoding and semantic failures. The rendered prefix comes from the typed
route built by B-0, not from serde_yaml's zero-based path, while parser line/column locations stay.

- A failing step field names the field: `id: ~` on step 1 → `jobs.b.steps[1].id: expected a string`.
- An unknown step or job key names its enclosing mapping, whose serde detail names the key:
  `with:` on step 1 → ``jobs.b.steps[1]: unknown field `with`, …``. The typed `route()` stays the
  key's own route (`jobs.b.steps[1].with`), as in B-0.
- A bad `runs-on` or `needs` list entry → `jobs.b.runs-on[2]: …`, `jobs.b.needs[2]: …`.
- Job identifiers are validated before `needs` resolution, so an invalid ID plus an unknown need
  reports the ID. The pseudo-route `jobs identifier` becomes the field route `jobs.<id>`.
- Identifiers in diagnostic text use one quoting style, `'…'`, with one escaping pass; routes keep
  raw identifiers and the host escapes control characters once. A duplicate job names its ID.
- Manual-input identifiers follow the same rule: the pseudo-route `inputs identifier` becomes
  `on.workflow_dispatch.inputs[.<id>]`, and `"…"` quoting becomes `'…'`.

## Acceptance

CLI tests in `crates/wsr/tests/inspection.rs` assert the exact diagnostic for each case above,
plus a control byte in a job ID rendered as `jobs.b\u{1}: invalid job identifier 'b\u{1}'`
(escaped exactly once). Existing assertions stay, except the contracts this slice changes.

Expected red: current zero-based `jobs.b.steps[0]: …`, `jobs.b: runs-on[2]: …`, field-less
`jobs.b.steps[0]: expected a string`, needs-first ordering, `jobs identifier:` pseudo-route,
double-escaped `"b\\u{1}"` and an anonymous `jobs: duplicate job identifier`.

## Known remaining gap

Errors raised inside serde_yaml itself (structural shape mismatches such as a scalar step,
YAML duplicate keys inside a value, recursion/alias limits) carry no detail WSR can separate
from serde_yaml's own path text, so their message keeps serde_yaml's wording and zero-based path.
Their typed `route()` stays `None`, as in B-0. Closing this needs an owned decoder for structural nodes.

Non-goals: owning-crate graph tests (B-2), new accepted fields (B-3..B-5, blocked on DR-002),
expression evaluation, execution.
