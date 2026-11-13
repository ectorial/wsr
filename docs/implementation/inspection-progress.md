# Workflow inspection worker checkpoint

## Current checkpoint — B-2 compiler tests implemented; PR pending checks

Same Schedule A run (lock retaken 2026-11-13 after the PR #30 wake). PR #30 squash-merged as
`1545ab3` after all four required checks passed on `a533c3e` with no reviews or comments;
`wsr-gha` and `wsr` tests verified on synced main. B-2 on `test/gha-compiler-characterization`
from `1545ab3`: new `crates/wsr-gha/tests/compiler.rs` (6 tests) — characterization, no red
expected; product code unchanged. Writing the alias-bomb case showed a top-level anchor fails as an
unknown key before expansion, so the bomb sits in `needs`. `cargo xtask ci` and `git diff --check`
pass locally. Implemented by Claude (claude-opus-5-5); cross-review by Codex gpt-6.1-sol (high).

Next action: watch the PR; merge on green with no unresolved findings. After B-2 every remaining
backlog item (B-3..B-6) is blocked on DR-002; the next run should report that and exit unless the
owner has decided DR-002 or Schedule B has appended a ready slice.

## Previous checkpoint — B-1 diagnostic routes delivered (PR #30, `1545ab3`)

Schedule A run 2026-11-13T02:57Z (no prior lock; lock taken). Baseline `d9712ec` = origin/main;
Dependabot #23 untouched; no open Schedule B PR. Branch `fix/inspect-diagnostic-routes`; slice
record [inspection-diagnostic-routes.md](inspection-diagnostic-routes.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no
blockers, one nit (substring assertion could accept a doubled location) fixed by asserting
complete stderr.

- Red (observed on `d9712ec` build): `jobs.b.steps[0]: unknown field `with``, field-less
  `jobs.b.steps[0]: expected a string` for `id: ~`, `jobs.b: runs-on[2]: …`, `jobs.b: needs[2]: …`,
  needs-first ordering (`jobs.b c.needs: unknown job 'missing'`), `jobs identifier:` pseudo-route,
  double-escaped `jobs: invalid job identifier "b\\u{1}"`, anonymous `jobs: duplicate job
  identifier`, `on.workflow_dispatch.inputs: invalid input identifier "a b"`. New CLI test
  `inspect_diagnostics_use_one_based_field_routes_and_validate_job_ids_first` failed on the first case.
- Green: that test (9 exact-stderr cases), updated dispatch identifier assertions, a wsr-gha
  typed-route case for `jobs.b.steps[1].id` and `name`; all B-0 category/route tests unchanged and
  passing. `cargo xtask ci` and `git diff --check` pass locally.
- Remaining gap (documented): serde_yaml/libyaml-raised errors keep decoder wording, zero-based
  path and no typed route.

Next action: watch the PR; merge when the four required checks pass with no unresolved findings,
then take B-2 (owning-crate graph/ID/shape/hostile-YAML characterization tests).

## Previous checkpoint — B-0 typed inspection errors delivered (PR #29, `d9712ec`)

PR #29 squash-merged as `d9712ec` after all four required checks passed on `1b266a2`, with no
review comments. Merged diagnostics (4) and CLI inspection (31) tests verified on synced main.

## Previous checkpoint — B-0 typed inspection errors implemented; PR pending checks

Schedule A run 2026-11-13T01:57Z (no prior lock). Baseline `3793749` = origin/main; Dependabot #23
untouched; no open Schedule B PR. Local uncommitted run-name checkpoint edits were superseded by
`3793749` and saved as a patch under `.git/wsr-worker/` before the fast-forward. Branch
`fix/inspect-typed-errors`. Implementer: Claude (claude-opus-5-5, orchestrator). Cross-review: Codex
gpt-6.1-sol high: no blockers; two P2 over-broad `Unsupported` classifications fixed with red/green.

Delivered per [typed-error iteration](inspection-typed-errors.md) (backlog B-0): `wsr_gha::inspect`
returns a provisional `wsr_gha::Error` with `Category::{Invalid, Unsupported}` and `route()`.
Red: three new `crates/wsr-gha/tests/diagnostics.rs` tests failed on `(Invalid, None)` with a catch-all
classification. Green: 4 diagnostics tests, 31 unchanged CLI inspection tests, `cargo xtask ci`,
`git diff --check`; a 53-case `wsr inspect` stderr/exit corpus was byte-identical to `3793749`.

Incident: an empty-SHA `git stash apply` during review fixes partially applied the preserved
prototype stash (`daee088`, still `stash@{0}`, not dropped) to the checkout. The tree was restored to
the committed slice; the conflicted status list is saved under `.git/wsr-worker/`. No stash commands
are used in this loop any more.

Next action: wait for required hosted checks on the PR, merge the verified head with no unresolved
findings, sync main, mark B-0 delivered. Then take B-1 (consistent diagnostic routes and validation
order), building on the typed routes. B-3..B-6 remain blocked on DR-002.

## Previous checkpoint — November 13, 2026

Run-name inspection merged as [PR #27](https://github.com/ectorial/wsr/pull/27), commit `fd574dc`
(`main` and `origin/main`). The iteration is delivered; see the [run-name iteration](inspection-run-name.md).
The Schedule B architecture review's first run ([log](architecture-review.md)) corrected host-boundary
hardening (escaped stderr diagnostics, regular-file requirement, broken-pipe handling) and refreshed
stale documentation; remaining ready work is in the [backlog](backlog.md) and owner questions are in
[decision requests](decision-requests.md). No expression evaluation, execution or runtime support is
claimed. Java worker remains paused.

Next action: Schedule A takes the next ready slice from [backlog.md](backlog.md).

## Historical checkpoint — run-name inspection implemented; PR pending checks (superseded)

Schedule A run 2026-11-13T00:57Z (no prior lock). Baseline `1c565a4` = origin/main; Dependabot #23
untouched. Branch `feat/inspect-run-name` from origin/main; prior planning edits included as owned work.
Implementer: Claude (claude-opus-5-5, orchestrator). Cross-review: Codex gpt-6.1-sol high — no code
findings; one P3 (stale checkpoint) fixed here.

Delivered per [run-name iteration](inspection-run-name.md): strict optional `run-name`
string, `SourceText::{Literal, Unevaluated}` summary, escaped rendering after the workflow name.
Red: three new CLI tests failed with `unknown field run-name`. Green: 28 CLI inspection tests,
public parser summary test (`crates/wsr-gha/tests/run_name.rs`), `cargo xtask ci` and
`git diff --check`. Pinned schema `4043eda` defines `run-name` as a string with
`github`/`inputs`/`vars` contexts; WSR binds none of them.

Next action: wait for required hosted checks on the PR, merge the verified head with no unresolved
findings, sync main, mark the iteration delivered. Then backlog item 1 continues with explicit
unsupported/dynamic diagnostics and source traceability gaps (permissions, job timeout, step shell,
env, action inputs, working-directory), then real `.github/workflows/example-*.yml` acceptance.
No expression evaluation, execution or runtime support is claimed. Java worker remains paused.
Older checkpoint sections below retain historical state.

## Bootstrap — November 6, 2026

Owner authorized a one-off run of the saved workflow-inspection schedule, readiness setup and
a first-beta dependency map. The recurring task remains scoped to inspection; the Java worker
remains paused. See [proposed beta dependencies](beta-dependencies.md).

Baseline: `main` and `origin/main` were both `04b4861`; checkout clean, no open feature PR.
`cargo xtask ci` passed (format, locked all-target/all-feature compilation and Clippy, workspace
tests including the offline corpus, and strict Rustdoc). This proves development readiness,
not workflow execution support. `cargo xtask doctor` found rustfmt, Clippy and all seven
optional tools ready. Rust was 1.99.1, Python 3.15.2; GitHub authentication was usable.
No custom `core.hooksPath` was configured. No hooks, toolchain, global configuration,
dependencies or variable names were changed. Optional tools need no refresh for inspection.

## Next action

Continue in the existing local checkout on `feat/workflow-inspection`. The uncommitted new
`beta-dependencies.md` and this checkpoint were created by this bootstrap, not unrelated user
work; include/update them alongside the first feature PR. No implementation or behavioral red
has been observed yet. Read the saved task's acceptance scope and current provider decisions
before coding. Resolve whether the first useful inspection slice fits the accepted compilation
boundary without inventing a native-provider bypass. If it does, deliver that bounded CLI
behavior through red/green and CI. If not, report the exact missing decision and alternatives.

The native manual-run control is used after bootstrap; its start acknowledgement is not a
completion claim. Keep the saved cadence, enabled state and prompt unchanged.

## First manual run — prerequisite decision

Inspected `04b4861` on `feat/workflow-inspection`; no open PR. Only the bootstrap documentation
is modified. `wsr inspect` is still a placeholder, `wsr-gha` is documentation-only, and the
legacy provider trait is explicitly not the accepted plugin contract. No WIT files or component
host implementation exist in the checkout. No product code, tests, dependencies or hooks changed;
no behavioral red or delivered feature is claimed.

D-007/D-009 require the host provider abstraction to delegate through versioned WIT to a signed
Component. D-010 requires verified, pinned activation; its pin schema remains open. The scheduled
scope explicitly forbids bypassing that architecture. Native parsing directly linked into the CLI
would introduce precisely that bypass, even if labelled inspection-only.

Owner decision required before the first successful workflow-inspection slice:

1. Authorize an explicitly temporary, built-in GHA compiler for local inspection only, with a
   provider-neutral result and no runtime compatibility claim. Record this scoped exception to
   D-009/D-010; later Component packaging must reuse the compiler and acceptance fixtures.
2. Keep the current boundary mandatory from the first slice, and authorize prerequisite work on
   the minimal versioned WIT/result/diagnostic contract, component host, trust and pinned activation
   before delivering inspection. Runtime/trust choices must be settled explicitly, not guessed.

Recommendation: option 1 for early inspection feedback, provided the owner accepts the temporary
exception; option 2 if exercising the production plugin boundary from day one is more important.
Neither option is selected by this checkpoint. Do not repeat environment preparation or substitute
documentation work on subsequent runs while this decision is pending. The schedule stays unchanged.

## Owner resolution and first implemented slice — November 6, 2026

Owner selected option 1. D-031 records the temporary built-in inspection exception; D-009 remains
mandatory for the target plugin architecture. No further approval is needed for this bounded path.
The beta dependency map remains a proposal outside the inspection milestone.

Implemented: the real CLI reads explicit source, invokes the pure built-in `wsr-gha` compiler,
and renders provider-neutral source summaries. Jobs are sorted; steps preserve source order;
runner requirements and static action references remain unresolved. Unsupported fields, expressions,
invalid shapes and duplicate identifiers fail explicitly before any stdout report. See the exact
[CLI subset](../cli/README.md#static-workflow-inspection).

Evidence: existing CLI tests passed before product changes. The first inspection acceptance test
failed on the intended placeholder diagnostic, then passed. Additional observed red/green cycles
covered dynamic fields, invalid/empty input, and static manual-trigger inspection. All four new
CLI acceptance tests, the five existing executable contract tests, and full `cargo xtask ci` pass.
No hooks, global environment, product versions, registry releases or legacy type declarations changed.
No workload execution or runtime enforcement is claimed.

Delivery: preparing the first feature PR from baseline `04b4861`; hosted checks and merge remain
pending. Next run must finish this PR before another slice. After merge, add step identifiers and
runner label lists with acceptance tests, then static `needs` graph validation, broader unsupported/
dynamic diagnostics and canonical example-workflow integration. Do not mark the entire milestone
complete or pause its schedule yet. Compiler/model-only changes do not count as delivered features.
