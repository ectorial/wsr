# Runner selector iteration (backlog B-35)

Scope: a job `runs-on` mapping with `group` and/or `labels` is retained as written and rendered
without selecting, discovering or provisioning a runner. Labels keep their source form (scalar or
list, order, spelling, repetitions), as top-level `runs-on` does. `RunnerRequirement::Selector` is a
D-033 temporary GHA-derived variant.

Pinned basis (actions/languageservices `4043eda158e16579cc5fb1b0b07a4bce2a76f0b5`):
[schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
`runs-on` (contexts `github`, `inputs`, `vars`, `needs`, `strategy`, `matrix`) is one of a
non-empty string, a sequence of them, or `runs-on-mapping` (`group`: non-empty string; `labels`:
`runs-on-labels`, a non-empty string or sequence of them). Nested definitions inherit the
`runs-on` contexts. The
[converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/job/runs-on.ts)
skips expression values and rejects a literal group with more than one `/`, or with one `/` unless
the prefix is `org`, `organization`, `ent` or `enterprise` and the name after it is non-empty.

| Source | Category |
| --- | --- |
| `group` literal or `${{` text; `labels` scalar or list, literal or `${{` items | Inspected as written |
| `labels: []` | Inspected (the schema sequence has no minimum) |
| empty mapping `{}` | Unsupported; the pin accepts it but it names no runner, so it is not inspected |
| `group` or label number/boolean | Unsupported (B-13) |
| unknown or non-string key | Invalid |
| null, blank (whitespace-only, the existing `runs-on` label rule), nested list or mapping value | Invalid |
| literal group `a/b` with another prefix, `org/`, or `a/b/c` | Invalid (converter) |

Acceptance: `crates/wsr-gha/tests/runner_selector.rs` (retention and categories with routes such
as `jobs.a.runs-on.group` and `jobs.a.runs-on.labels[2]`) and a CLI golden with hostile text, the
`executed` sentinel and the positive-controlled PATH trap.

Contract change: `crates/wsr-gha/tests/diagnostics.rs` used `{group: fleet}` and
`{group: fleet, labels: [a]}` as Unsupported samples; `{}` replaces them. `{group: 1}` stays
Unsupported and `{group: ~}` stays Invalid, now routed at `jobs.b.runs-on.group`.

Expected red (live probe): `jobs.a.runs-on: unsupported runner-selector mapping at line 4 column 5`.

## Evidence

- Baseline: `RunnerRequirement::Selector` and the extracted `render_runner` landed first; CLI (72)
  and dogfood (3) suites stayed green.
- Red (observed): `runner_selector.rs` failed with `jobs.a.runs-on: unsupported runner-selector
  mapping at line 3 column 5`; the CLI table case `{group: fleet}` exited 0 instead of 1 once the
  decoder changed. The CLI golden was written after the renderer, so its red was observed only
  at the `wsr-gha` level.
- Further contract changes (same categories, more specific routes): `scalar_conversion.rs` selector
  rows and `name_runner_expressions.rs` (`{group: '${{ x }}'}` now inspects; `{}` replaces it).
- Green: `runner_selector.rs` (2 tests, 25 cases), the `wsr-gha` and CLI suites (73), `cargo xtask ci`,
  `git diff --check`.
