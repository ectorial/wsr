# Workflow inspection worker checkpoint

## Current checkpoint — B-29 job container implemented; PR pending checks

Same Schedule A run, after the PR #58 wake (lock retaken 2026-11-14). PR #58 squash-merged as
`094c0e0` after all four required checks passed on `e1364d1`, with no reviews or comments. The
`wsr` and `wsr-gha` suites were verified on synced main. B-29 is on `feat/b29-job-container` from
`094c0e0`; slice record [inspection-job-container.md](inspection-job-container.md). Steady rotation
slice 2: implemented by Claude (claude-opus-5-5, orchestrator; `wsr-types` `Container` and
`ContainerCredentials` under D-033, plus a new `wsr-gha` `container` module). Cross-reviewed by
Codex gpt-6.1-sol (high): one P1, one P2 and one P3, all fixed with regressions; no second review
was run. Next rotation: B-30 Codex (gpt-6.1-sol, reasoningEffort medium), with a Claude review.

- Red (observed): `container.rs` failed with ``unsupported field `container`; this field is not
  inspected yet`` (Unsupported at `jobs.a.container`). A second red: past the B-23 scan budget, a
  call job's `container` before `uses` was silently accepted. The fix adds `container` to the
  call-job field check.
- Review fixes: P1, a mismatched-tag password was quoted by a decoder error. Red was observed in a
  `wsr-gha` test, and a CLI regression asserts secret-free stderr, empty stdout and exit 1. P2,
  whitespace-only `image`/`username`/`password` are kept as written. P3, the table now matches the
  null handling.
- Green: `container.rs` (5 tests, 32 category cases) and 2 CLI tests (a hostile golden with a PATH
  shim for `docker`, `podman` and `sh`, an `executed` sentinel and an unprinted password; plus the
  redaction test). The CLI golden was written after the renderer, so its red was observed only at
  the `wsr-gha` level. `cargo xtask ci` and `git diff --check` pass.
- Process note: this run once invoked the read-only `git stash list` by mistake. It changed
  nothing, and stash@{0} was verified unchanged.

B-29 is delivered on merge of this PR. Next action: watch the PR and merge on green with no
unresolved findings, then take B-30 (job `strategy`) with Codex as the implementer.

## Previous checkpoint — B-28 run expressions delivered (PR #58, `094c0e0`)

Schedule A run 2026-11-14 (lock taken fresh, none present). PR #55 had merged as `0792966` and Schedule
B's PR #57 as `e4a7ac8`; main synced. B-28 on `feat/b28-run-expressions` from `e4a7ac8`; slice record
[inspection-run-expressions.md](inspection-run-expressions.md). **Steady mode** began at the
2026-11-14 00:00 America/Denver reset. Rotation (2 Claude : 1 Codex): B-28 is steady slice 1,
implemented by Claude (claude-opus-5-5, orchestrator; interface-touching: `StepSummaryKind::Run`
gains `unevaluated`, D-033); cross-reviewed by Codex gpt-6.1-sol (high): no code findings, one P3
fixed (stale classification table). Next rotation: B-29 Claude, B-30 Codex.

- Baseline: the type change landed first with all `wsr-gha` and `wsr` suites green.
- Red (observed): `run_expressions.rs` failed with ``jobs.a.steps[1].run: expression is unevaluated;
  dynamic inspection is not supported yet`` (`Unsupported`); the CLI golden exited 1.
- Contract change: the `diagnostics.rs` expression-context sample moved from `run` to a whole step
  `env` expression (still `Unsupported`, same route shape).
- F40: the repository-workflow test asserts its cases equal sorted `.github/workflows/*.yml` minus
  `example-*` (passes with the current six).
- Green: `run_expressions.rs` (2 tests), CLI golden with hostile script text, the `executed`
  sentinel and a no-script-text assertion; CLI suite 64; `cargo xtask ci`; `git diff --check`.

B-28 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-29 (job `container`, Claude).

## Previous checkpoint — B-25 required `on` delivered (PR #55, `0792966`)

Same Schedule A run, after the PR #54 wake (lock retaken 2026-11-14). PR #54 squash-merged as
`d061843` after all four required checks passed on `153af59` with no reviews or comments; `wsr` and
`wsr-gha` tests (150) verified on synced main. B-25 on `feat/b25-require-on` from `d061843`; slice
record [inspection-require-on.md](inspection-require-on.md). Implemented by Claude (claude-opus-5-5,
orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no P1/P2; one P3 fixed (flow
roots in the CLI helper).

- Red (observed): `a_workflow_without_on_is_invalid` failed because inspection succeeded.
- Contract changes: fixtures route through `common::inspect` / `with_on` (append `on:
  workflow_dispatch` after the content); 19 CLI goldens gained the trigger line; the
  `dispatch_inputs.rs` and `triggers.rs` empty-trigger assertions now expect the missing-field error;
  size-boundary fixtures carry `on` inside their measured source.
- Green: all `wsr-gha` suites, CLI suite 63; `cargo xtask ci`; `git diff --check`.

B-25 is delivered on merge of this PR. With B-21..B-25 delivered, no ready slice remains. Next
action: watch the PR; merge on green with no unresolved findings; then report and exit until
Schedule B or the owner adds a slice. Open follow-up for Schedule B: alias expansion in step/call
`with` values (B-23 checkpoint).

## Previous checkpoint — B-24 name and runner expressions delivered (PR #54, `d061843`)

Same Schedule A run, after the PR #53 wake (lock retaken 2026-11-14). PR #53 squash-merged as
`4136fab` after all four required checks passed on `3449c00` with no reviews or comments; `wsr` and
`wsr-gha` tests (148) verified on synced main. B-24 on `feat/b24-name-runs-on-expressions` from
`4136fab`; slice record [inspection-name-runner-expressions.md](inspection-name-runner-expressions.md).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; names and runner labels became
D-033 `SourceText`); cross-reviewed by Codex gpt-6.1-sol (high): no findings.

- Red (observed): `name_runner_expressions.rs` failed with `jobs.a.name: expression is unevaluated;
  dynamic inspection is not supported yet` (Unsupported); the type change landed first with all
  existing suites green.
- Contract changes: `diagnostics.rs` keeps only `run` as its expression-context sample; the CLI
  `runs-on` expression rejection became a success golden and two rows left the runner-label table;
  name/label assertions use `SourceText`.
- Green: 2 `wsr-gha` tests, 1 CLI golden; CLI suite 62; `cargo xtask ci`; `git diff --check`.

B-24 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-25 (require `on`).

## Previous checkpoint — B-23 reusable-workflow calls delivered (PR #53, `4136fab`)

Same Schedule A run, after the PR #52 wake (lock retaken 2026-11-14). PR #52 squash-merged as
`568793f` after all four required checks passed on `4d6791b` with no reviews or comments; `wsr` and
`wsr-gha` tests (143) verified on synced main. B-23 on `feat/b23-reusable-calls` from `568793f`; slice
record [inspection-workflow-calls.md](inspection-workflow-calls.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; `wsr-types` `WorkflowCall`/`CallSecrets` under D-033);
cross-reviewed by Codex gpt-6.1-sol (high), four rounds, eight P2s, all fixed (see the record). The
last fix (a scan budget of one job key per source byte) is small and measured (0.16 s, 8 MB on the
reviewer's probe); no fifth review was run.

- Red (observed): `calls.rs` failed with ``jobs.b: unsupported field `uses` `` (Unsupported); types
  landed first so the red was an assertion failure.
- Contract changes: `JobSummary.runner` is an `Option` (`compiler.rs`, `runner_labels.rs` wrap it in
  `Some`); the repository-workflow test expects a temporary `ci.yml` copy to exit 0.
- Green: `calls.rs` (3 tests, 36 category cases), 2 CLI tests; missing `runs-on`/`steps` keep wording
  and location; CLI suite 62; `cargo xtask ci`; `git diff --check`.
- Follow-up for Schedule B (pre-existing, not introduced here): YAML aliases inside step or call
  `with` values are materialized and rendered expanded (a 65,536-character anchor aliased 1,024 times
  peaks at ~140-200 MB and expands stdout).

B-23 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-24 (job/step `name` and `runs-on` expressions).

## Previous checkpoint — B-22 event filters delivered (PR #52, `568793f`)

Same Schedule A run, after the PR #51 wake (lock retaken 2026-11-14). PR #51 squash-merged as
`7c2225e` after all four required checks passed on `90cee4d` with no reviews or comments; `wsr` and
`wsr-gha` tests (139) verified on synced main. B-22 on `feat/b22-event-filters` from `7c2225e`; slice
record [inspection-event-filters.md](inspection-event-filters.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; `wsr-types` `Trigger::Filtered`/`EventFilter` under
D-033); cross-reviewed by Codex gpt-6.1-sol (high): no findings (pair-rejection basis noted in the
record).

- Red (observed): `triggers.rs` failed with `on.pull_request: event configuration is not inspected yet`
  (Unsupported at `on.pull_request`); the type landed first so the red was an assertion failure.
- Contract changes: `push` Unsupported samples in `diagnostics.rs`, `triggers.rs` and the CLI dispatch
  test became `release: {types: [published]}` / `pull_request.types`.
- Green: 2 new `wsr-gha` tests, 2 CLI tests; F33 repository-workflow test with PATH traps pins
  `ci.yml` (`uses`) and `release.yml` (`strategy`); CLI suite 60; `cargo xtask ci`; `git diff --check`.

B-22 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-23 (reusable-workflow call jobs).

## Previous checkpoint — B-21 case-insensitive identifiers delivered (PR #51, `7c2225e`)

New Schedule A run (lock created 2026-11-14T02:24Z; none present). PR #49 (B-19) had merged as
`7ecb135` and Schedule B's PR #50 as `ba2c704`; no loop PR open. B-21 on
`feat/b21-case-insensitive-ids` from `ba2c704`; slice record
[inspection-case-ids.md](inspection-case-ids.md). Implemented by Claude (claude-opus-5-5,
orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no findings.

- Red (observed): the `build`/`Build` (`compiler.rs`) and `mode`/`MODE` (`dispatch_inputs.rs`)
  probes inspected successfully (`unwrap_err` on `Ok`). The CLI table row was added with them and run
  only after implementation (acceptance).
- Green: 2 `wsr-gha` tests, 1 CLI row; `cargo test -p wsr --test inspection` (58); `cargo xtask ci`;
  `git diff --check`.

B-21 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-22 (push/pull_request filters).

## Previous checkpoint — B-19 job environment delivered (PR #49, `7ecb135`)

Same Schedule A run, after the PR #48 wake (lock retaken 2026-11-13). PR #48 squash-merged as
`e39fe9e` after all four required checks passed on `f581333` with no reviews or comments; `wsr` and
`wsr-gha` tests (126) verified on synced main. B-19 on `feat/b19-job-environment` from `e39fe9e`;
slice record [inspection-job-environment.md](inspection-job-environment.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; `wsr-types` `Environment` under D-033); cross-reviewed by
Codex gpt-6.1-sol (high): no findings.

- Red (observed): `environment.rs` failed with ``jobs.b: unknown field `environment` `` (Unsupported).
- Contract change: the job "expected one of" list in `diagnostics.rs` names `environment`.
- Green: 3 `wsr-gha` tests, 2 CLI tests (written after the `wsr-gha` green; acceptance); full suites
  (131); `cargo xtask ci`.

B-19 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings. After that no ready slice remains (B-0..B-19 delivered); report and exit until Schedule B
or the owner adds one. Open follow-ups for Schedule B are in the B-17 and B-18 checkpoints below.

## Previous checkpoint — B-18 job outputs delivered (PR #48, `e39fe9e`)

Same Schedule A run, after the PR #47 wake (lock retaken 2026-11-13). PR #47 squash-merged as
`5ec43bf` after all four required checks passed on `e4a96c2` with no reviews or comments; `wsr` and
`wsr-gha` tests (121) verified on synced main. B-18 on `feat/b18-job-outputs` from `5ec43bf`; slice
record [inspection-job-outputs.md](inspection-job-outputs.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; `wsr-types` field under D-033); cross-reviewed by Codex
gpt-6.1-sol (high): no blockers; one P2 fixed (boolean/number keys Invalid: the pinned YAML adapter
keeps them raw and the reader rejects them), one P3 fixed (null keys documented). Follow-up for
Schedule B: `step_inputs.rs` (step/workflow/job `env`, step `with`) still classifies boolean and number
keys as `Unsupported` under the same reader path.

- Red (observed): `outputs.rs` failed with ``jobs.b: unknown field `outputs` `` (Unsupported).
- Contract change: the job "expected one of" list in `diagnostics.rs` names `outputs`.
- Green: 3 `wsr-gha` tests, 2 CLI tests (written after the `wsr-gha` green; acceptance); full suites
  (126); `cargo xtask ci`.

B-18 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-19 (job `environment`).

## Previous checkpoint — B-17 continue-on-error and step timeouts delivered (PR #47, `5ec43bf`)

Same Schedule A run, after the PR #46 wake (lock retaken 2026-11-13). PR #46 squash-merged as
`472887c` after all four required checks passed on `e766d96` with no reviews or comments; `wsr` and
`wsr-gha` tests (117) verified on synced main. B-17 on `feat/b17-continue-timeout` from `472887c`;
slice record [inspection-continue-timeout.md](inspection-continue-timeout.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; `wsr-types` fields under D-033); cross-reviewed by Codex
gpt-6.1-sol (high): no blockers; one P2 fixed (step `background` is not a valid Unsupported sample).
Follow-up for Schedule B: the `Step` schema-key list classifies `background`, `cancel`, `parallel`,
`wait` and `wait-all` as `Unsupported`, but the pinned reader rejects at least `background` unless
`allowBackgroundSteps` (default false) is enabled, so they may belong in `Invalid`.
Resolved in B-20 ([record](inspection-diagnostic-text.md)): the reader rejects `background`, `wait`,
`wait-all` and `cancel` by default (template-reader.ts L29-30, L221), so they are `Invalid`; only
`parallel` stays `Unsupported`.

- Red (observed): `continue_timeout.rs` failed with ``jobs.b: unknown field `continue-on-error` ``
  (Unsupported).
- Contract changes: `diagnostics.rs` Unsupported samples → job `cancel-timeout-minutes`, step
  `name: 5`; job and step "expected one of" lists name the new keys (two CLI tests).
- Green: 2 `wsr-gha` tests, 2 CLI tests (written after the `wsr-gha` green; acceptance); full suites
  (121); `cargo xtask ci`.

B-17 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-18 (job `outputs`).

## Previous checkpoint — B-16 concurrency delivered (PR #46, `472887c`)

Same Schedule A run, after the PR #45 wake (lock retaken 2026-11-13). PR #45 squash-merged as
`e95b0d4` after all four required checks passed on `bb1ec38` with no reviews or comments; `wsr` and
`wsr-gha` tests (112) verified on synced main. B-16 on `feat/b16-concurrency` from `e95b0d4`; slice
record [inspection-concurrency.md](inspection-concurrency.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; interface-touching: `wsr-types` `Concurrency`, D-033);
cross-reviewed by Codex gpt-6.1-sol (high): no blockers; one P2 fixed (expression keys Invalid per
the pinned converter).

- Red (observed): `concurrency.rs` failed with ``unknown field `concurrency`, expected one of `name`,
  `run-name`, `on`, `permissions`, `env`, `defaults`, `jobs` ``.
- Contract changes: `diagnostics.rs` workflow Unsupported sample `concurrency` → `description`; the
  job "expected one of" list names `concurrency`.
- Green: 3 `wsr-gha` tests, 2 CLI tests (written after the `wsr-gha` green; acceptance); full suites;
  `cargo xtask ci`.

B-16 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-17 (job/step `continue-on-error`, step `timeout-minutes`).

## Previous checkpoint — B-15 trigger events delivered (PR #45, `e95b0d4`)

Same Schedule A run, after the PR #44 wake (lock retaken 2026-11-13). PR #44 squash-merged as
`c4472b5` after all four required checks passed on `eb82681` with no reviews or comments; `wsr` and
`wsr-gha` tests (106) verified on synced main. B-15 on `feat/b15-trigger-events` from `c4472b5`; slice
record [inspection-trigger-events.md](inspection-trigger-events.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; interface-touching: `wsr-types` `Trigger`, D-033);
cross-reviewed by Codex gpt-6.1-sol (high): no blockers; one P2 fixed (valid cron in a sample), one
P3 fixed (empty `on` collections Unsupported, not Invalid).

- Red (observed): `triggers.rs` failed with `on: unsupported trigger; only workflow_dispatch is
  inspected yet` (`workflow_call`) and `on: expected a mapping (other trigger forms are unsupported)`.
- Contract changes: `WorkflowSummary.trigger` → `triggers: Vec<Trigger>` (`dispatch_inputs.rs`);
  `diagnostics.rs` bare-`push` Unsupported samples → configured events, sequence routes → `on[n]`;
  four CLI `on` diagnostics in the dispatch test; orphaned `Error::with_category` removed.
- Green: 3 `wsr-gha` tests, 3 CLI tests (written after the `wsr-gha` green; acceptance), including
  temporary copies of `check-docs`/`check-fmt`/`check-lint`/`test`; full suites (112); `cargo xtask ci`.

B-15 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-16 (workflow and job `concurrency`).

## Previous checkpoint — B-14 shell and action-reference validation delivered (PR #44, `c4472b5`)

New Schedule A run (lock created 2026-11-13T21:24Z; no prior lock). PR #42 (B-12) was already merged
as `6f58e96`; Schedule B's PR #43 (B-13) merged as `51bb95d`; main synced. B-14 on
`feat/b14-shell-uses-validation` from `51bb95d`; slice record
[inspection-shell-uses.md](inspection-shell-uses.md). Implemented by Claude (claude-opus-5-5,
orchestrator, burn mode; mechanical, no `wsr-types` change); cross-reviewed by Codex gpt-6.1-sol
(high): no blockers; four P2s fixed (runner path normalization, `.\` paths and `$/` Unsupported,
case-insensitive built-ins, first-space split instead of a whole-string trim).

- Red (observed): `shell_uses.rs` failed with `unwrap_err()` on `Ok` for `shell: 'true'` and `uses: '5'`.
- Green: 5 `wsr-gha` tests, 1 CLI test (written after the `wsr-gha` green; acceptance, not red
  evidence); full `wsr`/`wsr-gha` suites, dogfood goldens unchanged; `cargo xtask ci`; `git diff --check`.
- No existing assertion changed.

B-14 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-15 (non-dispatch `on:` events with empty bodies).

## Previous checkpoint — B-12 job and step conditions delivered (PR #42, `6f58e96`)

Same Schedule A run, after the PR #41 wake (lock retaken 2026-11-13). PR #41 squash-merged as
`b686358` after all four required checks passed on `b22ae10` with no reviews or comments; `wsr` and
`wsr-gha` tests (84) verified on synced main. B-12 on `feat/b12-if-conditions` from `b686358`; slice
record [inspection-conditions.md](inspection-conditions.md). Implemented by Claude (claude-opus-5-5,
orchestrator, burn mode; interface-touching: `condition` fields in `wsr-types`, D-033);
cross-reviewed by Codex gpt-6.1-sol (high): no blockers; one P2 fixed (null `if` is Unsupported).

- Red (observed): both new CLI tests failed on ``jobs.b: unknown field `if` `` (Unsupported) / exit 1.
- Contract change (cited by the backlog): B-0's job-`if` Unsupported example in
  `wsr-gha/tests/diagnostics.rs` now uses `continue-on-error` (still Unsupported) and the location
  test's expected-fields list names `if`; CLI unknown-step-key messages name `if` too.
- Green: 2 CLI tests, 2 `wsr-gha` tests; full `wsr`/`wsr-gha` suites; `cargo xtask ci`.

B-12 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings. After that no ready slice remains (B-0..B-12 delivered); report and exit until Schedule B
or the owner adds one.

## Previous checkpoint — B-11 workflow/job env and run defaults delivered (PR #41, `b686358`)

Same Schedule A run, after the PR #40 wake (lock retaken 2026-11-13). PR #40 squash-merged as
`5c6e284` after all four required checks passed on `334b19e` with no reviews or comments; `wsr` and
`wsr-gha` tests (80) verified on synced main. B-11 on `feat/b11-env-defaults` from `5c6e284`; slice
record [inspection-env-defaults.md](inspection-env-defaults.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode; interface-touching: new `wsr-types` fields under D-033);
cross-reviewed by Codex gpt-6.1-sol (high): no blockers; three P2s fixed (number/boolean values
Unsupported; job `defaults.run` expression keys Unsupported; unknown keys routed to the key).
Follow-up for Schedule B: step `shell`/`working-directory` still treat numbers/booleans as Invalid.

- Schema contexts read from the pinned `workflow-v1.0.json`: workflow-env, job-env and
  job-defaults-run have contexts; workflow-defaults, workflow-defaults-run and job-defaults do not;
  `shell`/`working-directory` require nonempty strings.
- Red (observed): both new CLI tests failed with ``unknown field `env`, expected one of `name`,
  `run-name`, `on`, `permissions`, `jobs` `` / exit 1.
- Intended contract change: the unknown-job-key ``expected one of …`` list in
  `wsr-gha/tests/diagnostics.rs` now names `env` and `defaults` (category/route/location unchanged).
- Green: 2 CLI tests, 2 `wsr-gha` tests; full `wsr`/`wsr-gha` suites; `cargo xtask ci`.

B-11 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-12 (job and step `if` as unevaluated conditions).

## Previous checkpoint — B-10 dogfood workflow-set guard delivered (PR #40, `5c6e284`)

Same Schedule A run, after the PR #39 wake (lock retaken 2026-11-13). PR #39 squash-merged as
`fb31d1c` after all four required checks passed on `8508157` with no reviews or comments; `wsr` and
`wsr-gha` tests (78) verified on synced main. B-10 on `test/b10-dogfood-set` from `fb31d1c`; slice
record [inspection-dogfood-guard.md](inspection-dogfood-guard.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
two P3s fixed (regular files only; non-UTF-8 names skipped). Test-only slice; no product code changed.

- No product red (characterization guard). Guard red proven twice: the temp-copy test with
  `example-extra.yml`, and an untracked `example-zz.yml` placed briefly in the real directory
  (failed with the extra name, then removed).
- Green: 3 dogfood tests; the golden table is now one `CASES` constant shared by the guard.

B-10 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-11 (workflow/job `env` and `defaults.run`), then B-12.

## Previous checkpoint — B-9 format-character escaping delivered (PR #39, `fb31d1c`)

Schedule A run 2026-11-13 (lock created 16:24 UTC; no prior lock). Gate: stash@{0} intact, main clean
at `3c8c95c`, only Dependabot PR #23 open. B-7 (#37) and Schedule B's B-8 (#38) were merged. B-9 on
`impl/b9-format-escape` from `3c8c95c`; slice record [inspection-format-escape.md](inspection-format-escape.md).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol
(high): no blockers; one P3 doc nit fixed.

- Red (observed on `3c8c95c` build and in the new CLI test): raw U+202E/U+200B on stderr in
  ``unknown field `k‮q​` `` and in the job-ID route `jobs.b‮⁦`.
- Green: `inspect_errors_escape_unicode_format_characters_on_stderr`; all 41 `wsr` inspection tests;
  `cargo xtask ci` and `git diff --check` pass. Stdout, exit codes, categories and routes unchanged.

B-9 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings; then take B-10 (dogfood workflow-set guard), then B-11, B-12.

## Previous checkpoint — B-7 size warning implemented; PR pending checks

Same Schedule A run, after the PR #36 wake (lock retaken 2026-11-13). PR #36 squash-merged as
`07d1892` after all four required checks passed on `8b5b3a2` with no reviews or comments; `wsr` and
`wsr-gha` tests verified on synced main. B-7 on `feat/inspect-size-warning` from `07d1892`; slice
record [inspection-size-warning.md](inspection-size-warning.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
P2 fixed with observed red (closed stderr made the warning panic: exit 101, empty stdout) and the
512,000-byte reading of GitHub's "500 KB" documented as WSR's interpretation.

- Red (observed on `07d1892` build): no stderr warning at 512,001 bytes.
- Green: the size test (3 boundary sizes); the closed-stdout test now expects exactly the warning
  for its 638,896-byte fixture (intended D-032 contract change).
- Roadmap Gate 1a (static inspection) is met on merge of this PR; Gate 1b remains unmet.

B-7 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings. After that the backlog has no ready slice (Gate 1b is out of scope unless the backlog marks
a slice ready): report and exit until Schedule B or the owner adds one.

## Previous checkpoint — B-6 dogfood acceptance delivered (PR #36, `07d1892`)

Same Schedule A run, after the PR #35 wake (lock retaken 2026-11-13). PR #35 squash-merged as
`de3e910` after all four required checks passed on `df98f13` with no reviews or comments; `wsr` and
`wsr-gha` tests verified on synced main. B-6 on `test/dogfood-inspection` from `de3e910`; slice record
[inspection-dogfood.md](inspection-dogfood.md). Implemented by Claude (claude-opus-5-5, orchestrator,
burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers; two nits fixed (marker via
`WSR_EXECUTION_MARKER` plus a per-trap positive control; trap evidence scoped to Unix). Test-only
slice; no product code changed.

- Red: none on current main (behavior delivered by B-3..B-5). Observed on pre-B-5 `85a1c30` in a
  temporary worktree (removed): ``jobs.python.steps[1]: unknown field `with` ``.
- Green: `crates/wsr/tests/dogfood.rs` golden test; marker sentinel verified to fire under the
  restricted `PATH` (a bare-`touch` draft was vacuous and was fixed).

B-6 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings, then take B-7 (500 KB warning), which completes roadmap Gate 1a.

## Previous checkpoint — B-5 step inputs delivered (PR #35, `de3e910`)

Same Schedule A run, after the PR #34 wake (lock retaken 2026-11-13). PR #34 squash-merged as
`85a1c30` after all four required checks passed on `136a811` with no reviews or comments; `wsr` and
`wsr-gha` tests verified on synced main. B-5 on `feat/inspect-step-inputs` from `85a1c30`; slice
record [inspection-step-inputs.md](inspection-step-inputs.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers.
Fixed: case-insensitive duplicate keys (pinned parser rule), `env`/`with` interpreted after
placement and run/uses/shell checks (a `with` expression on a run step was `Unsupported`), number/
boolean keys reclassified `Unsupported`, and hostile key/value/directory escaping tests. These
review-fix cases were added with their fixes; their red was not separately observed.

- Red (observed on `85a1c30` build): both new CLI tests failed with ``jobs.b.steps[1]: unknown field
  `with` ``.
- Green: 2 CLI tests (2 accepted workflows, 19 exact-stderr rejections), 3 `wsr-gha` tests. Intended
  contract update: the B-1 unknown-step-key example moved from `with` to `bogus`.
- Dogfood: `wsr inspect` now exits 0 on both `.github/workflows/example-*.yml` (static inspection only).

B-5 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings, then take B-6 (golden dogfood acceptance) or B-7 (500 KB warning).

## Previous checkpoint — B-4 job timeouts and step shells delivered (PR #34, `85a1c30`)

Same Schedule A run, after the PR #33 wake (lock retaken 2026-11-13). PR #33 squash-merged as
`40f1354` after all four required checks passed on `e2cc637` with no reviews or comments; `wsr` and
`wsr-gha` tests verified on synced main. B-4 on `feat/inspect-timeout-shell` from `40f1354`; slice
record [inspection-timeout-shell.md](inspection-timeout-shell.md). Implemented by Claude
(claude-opus-5-5, orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
two nits fixed (shell checks now follow the run/uses rule, with precedence tests; positivity documented
as a subset rule, not the schema's).

- Red (observed on `40f1354` build): both new CLI tests failed with ``jobs.b: unknown field
  `timeout-minutes` ``.
- Green: 2 CLI tests (3 accepted cases, 14 exact-stderr rejections), 2 `wsr-gha` tests. Intended
  contract updates: unknown-field lists name `shell`/`timeout-minutes`; B-0 unsupported step-key case
  `shell` → step `timeout-minutes`.
- Dogfood: `example-python.yml` now stops at step `with`, `example-shell.yml` at step `env` (B-5).

B-4 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings, then take B-5 (step `env`, `with`, `working-directory`).

## Previous checkpoint — B-3 requested permissions delivered (PR #33, `40f1354`)

Schedule A run 2026-11-13T15:24Z (no prior lock; lock taken). Baseline `319fc07` = origin/main;
Dependabot #23 untouched; no open Schedule B PR. Branch `feat/inspect-permissions`; slice record
[inspection-permissions.md](inspection-permissions.md). Implemented by Claude (claude-opus-5-5,
orchestrator, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): no blockers; two nits fixed
(backlog B-3 now states expressions are `Invalid`; rejection tests assert complete stderr).

- Red (observed on `319fc07` build): both new CLI tests failed with ``unknown field `permissions`,
  expected one of `name`, `run-name`, `on`, `jobs` ``.
- Green: 2 CLI tests (4 accepted forms, 11 exact-stderr rejections), 3 `wsr-gha` permissions tests;
  B-0 diagnostics contract updated intentionally (unsupported-key case `permissions` → `concurrency`;
  job unknown-field list now names `permissions`). `cargo xtask ci` and `git diff --check` pass locally.
- Dogfood: both `example-*.yml` now pass `permissions` and stop at `jobs.<id>: unknown field
  `timeout-minutes``, which B-4 owns.

B-3 is delivered on merge of this PR. Next action: watch the PR; merge on green with no unresolved
findings, then take B-4 (job `timeout-minutes`, step `shell`) from a fresh branch.

## Previous checkpoint — owner decisions D-032..D-034 recorded; backlog ready

PR #31 squash-merged as `a939203` after all four required checks passed on `c397622`. The owner
resolved DR-002 (D-033), DR-003 (D-032) and DR-004 (D-034); B-3..B-5 and B-7 are ready and B-6
follows B-3..B-5.

Incident (2026-11-13T02:11Z): during B-0, an empty-ref `git stash apply` applied the protected July
29 prototype stash to the project-root checkout. The run reset tracked files and removed the files the
partial apply created; stash `daee088` stayed intact and no work was lost. Workers no longer use
`git stash`, `git clean` or hard resets in the project root, and record delivery inside the feature
PR instead of leaving uncommitted edits on `main`.

Next action: take B-3 from a fresh branch.

## Previous checkpoint — B-2 compiler tests implemented; PR pending checks

Same Schedule A run (lock retaken 2026-11-13 after the PR #30 wake). PR #30 squash-merged as
`1545ab3` after all four required checks passed on `a533c3e` with no reviews or comments;
`wsr-gha` and `wsr` tests verified on synced main. B-2 on `test/gha-compiler-characterization`
from `1545ab3`: new `crates/wsr-gha/tests/compiler.rs` (8 tests) — characterization, no red
expected; product code unchanged. Writing the alias-bomb case showed a top-level anchor fails as an
unknown key before expansion, so the bomb sits in `needs`. `cargo xtask ci` and `git diff --check`
pass locally. Implemented by Claude (claude-opus-5-5); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
identifier-coverage gaps and overstated precedence claims fixed (step-ID syntax/length/expression,
duplicate job, expression job ID with `needs`, unknown-before-cycle). Full-message equality on
graph diagnostics kept deliberately: the cycle path is the behavior under test.

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
