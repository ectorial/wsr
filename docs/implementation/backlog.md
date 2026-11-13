# Inspection backlog

Ready slices for Schedule A, in order. Schedule B appends; Schedule A owns entries once started and
marks them delivered with PR/commit. Every slice follows the
[TDD contract](../testing/strategy.md#11-tdd-and-agent-implementation-contract), keeps stdout empty
and exit 1 on failure, preserves deterministic output, and claims no evaluation, binding or execution.
Slices marked **blocked** wait on an open [decision request](decision-requests.md) and are not ready;
Schedule A takes only ready slices.

## B-0 Provisional typed inspection errors in `wsr-gha` (audit F1)

- Status: **delivered** in [PR #29](https://github.com/ectorial/wsr/pull/29), `d9712ec` (Schedule A).

- Behavior: `wsr_gha::inspect` returns a provisional, explicitly non-stable typed error (in `wsr-gha`, not
  `wsr-types`) carrying a structured route and a category separating invalid source from valid-but-unsupported
  GHA. Classify at the point of detection, never from message wording; stop erasing typed failures through
  `serde::de::Error::custom` in `dispatch.rs`; preserve parser locations and duplicate-key evidence. `Unevaluated`
  stays successful summary state, not an error kind. Unknown fields not known to be valid GHA stay conservatively
  invalid. The host renders; CLI output stays byte-identical unless B-1 changes it. No WIT or stable protocol.
- Acceptance: `wsr-gha` integration tests assert category and route for: job `if:` (unsupported), unknown key
  (invalid), dispatch unsupported event (unsupported), expression in a literal-only field (unsupported); existing
  CLI tests unchanged.
- Expected red: compiler tests assert categories that `inspect` does not yet report; land the type with a
  single catch-all variant first so the red is an assertion failure, not a build failure.
- Files: `crates/wsr-gha/src/{lib.rs,needs.rs,dispatch.rs}`, new `crates/wsr-gha/src/diagnostic.rs`,
  `crates/wsr/src/commands/inspect.rs`, `crates/wsr-gha/tests/`.

## B-1 Consistent diagnostic routes and validation order (audit F3, F5)

- Status: **delivered** in [PR #30](https://github.com/ectorial/wsr/pull/30), `1545ab3` (Schedule A); see
  [slice record](inspection-diagnostic-routes.md). Unknown keys render at the enclosing mapping as
  specified; dispatch input identifiers received the same route/quoting rule.

- Behavior: every inspection diagnostic uses one route grammar `jobs.<id>.<field>[<n>]` with one-based
  indices, for both decode and semantic errors (steps, `runs-on`, `needs` entries). Job identifiers are
  validated before `needs` resolution. One quoting style for identifiers (`'…'` with a single escaping
  pass); duplicate-job diagnostic names the identifier. Replace pseudo-routes ("jobs identifier") with
  field routes. Build on B-0's typed routes.
- Acceptance: `with:` on the first step → `jobs.b.steps[1]: …`; `id: ~` on step 1 → `jobs.b.steps[1].id: …`;
  bad `runs-on` entry → `jobs.b.runs-on[2]: …`; invalid job ID plus unknown need reports the ID error;
  control byte in a job ID is escaped exactly once; duplicate job names the ID.
- Expected red: current zero-based `steps[0]`, `jobs.b: runs-on[2]` shapes and needs-first ordering.
- Files: `crates/wsr-gha/src/{lib.rs,needs.rs,dispatch.rs}`, `crates/wsr/tests/inspection.rs`, `docs/cli/README.md`.

## B-2 Owning-crate tests for the GHA inspection compiler (audit F6)

- Status: **delivered** in [PR #31](https://github.com/ectorial/wsr/pull/31), `a939203` (Schedule A); see
  [slice record](inspection-compiler-tests.md).

- Behavior: `wsr-gha` integration tests call `wsr_gha::inspect` directly for graph (missing, self, multi-job
  cycle, diamond, repeated edge), job/step ID rules, shape errors, and hostile YAML (alias bomb, deep nesting).
- Acceptance: tests in `crates/wsr-gha/tests/` assert summary structure and error presence/route, not host rendering.
- Expected red: none — characterization coverage; report as regression evidence, not TDD red/green.
- Files: `crates/wsr-gha/tests/*.rs`, `docs/testing/coverage.md`.

## B-3 Workflow and job `permissions` (requested, not granted)

- Status: **delivered on merge of the B-3 PR** (Schedule A); see [slice record](inspection-permissions.md).
  Expressions are `Invalid`, not `Unsupported`: the pinned schema gives `permissions` no expression context.

- Behavior: accept literal `read-all`/`write-all` or a scope→`read|write|none` map at workflow and job level;
  render as requested permissions; expressions rejected as unevaluated/unsupported; unknown scopes rejected.
- Acceptance: CLI tests for scalar, map, job override display, unknown scope, expression; no grant claim in output or docs.
- Expected red: ``unknown field `permissions` ``.
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-types/src/inspection.rs`, `crates/wsr/src/commands/inspect.rs`,
  `crates/wsr/tests/inspection.rs`, `docs/cli/README.md`.

## B-4 Job `timeout-minutes` and step `shell`

- Status: **delivered on merge of the B-4 PR** (Schedule A); see [slice record](inspection-timeout-shell.md).
  `shell` expressions are `Invalid`: the pinned schema gives `shell` no expression context.

- Behavior: literal positive number timeout and literal shell string reported as source data, never applied
  or selected; expressions shown as unevaluated `SourceText`, following the run-name precedent.
- Acceptance: literal, expression, invalid type (bool, negative) cases.
- Expected red: unknown field `timeout-minutes` / `shell`.
- Files: as B-3.

## B-5 Step `env`, `with` and `working-directory`

- Status: **delivered on merge of the B-5 PR** (Schedule A); see [slice record](inspection-step-inputs.md).
  Numbers and booleans keep their YAML type (the schema's loose string values); B-6 is now ready.

- Behavior: values reported as literal or unevaluated `SourceText` with escaped rendering; never bound, resolved
  or passed to anything; keys sorted deterministically; hostile values (control bytes, `${{`) escaped.
- Acceptance: per-field CLI tests plus a hostile-content sentinel test.
- Expected red: unknown field `env` / `with` / `working-directory`.
- Files: as B-3.

## B-6 Dogfood acceptance on `.github/workflows/example-*.yml`

- Status: **delivered on merge of the B-6 PR** (Schedule A; B-3 #33, B-4 #34, B-5 #35 merged); see
  [slice record](inspection-dogfood.md). No red on current main; red observed on pre-B-5 `85a1c30`.

- Behavior: `wsr inspect` succeeds on both example workflows with golden output; inspection performs no execution
  (sentinel files untouched). Translation-only; native reference success is not WSR support.
- Acceptance: golden-output integration tests reading the real dogfood files.
- Expected red: current `permissions` (and later field) rejections. Depends on B-3..B-5.
- Files: `crates/wsr/tests/inspection.rs`, `docs/testing/coverage.md`, `docs/context/status.md`.

## B-7 GitHub 500 KB workflow-size warning (D-032)

- Status: **ready**.

- Behavior: when the inspected file exceeds 500 KB (512,000 bytes), `wsr inspect` still succeeds and writes a
  warning to stderr that GitHub would not run this workflow (documented 500 KB limit); stdout is unchanged.
  No file-size rejection.
- Acceptance: CLI tests at 511,999/512,000/512,001-byte sizes (exit 0, warning only above the limit, identical stdout).
- Expected red: no warning for a 512,001-byte workflow.
- Files: `crates/wsr/src/commands/inspect.rs`, `crates/wsr/tests/inspection.rs`, `docs/cli/README.md`.
