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

- Status: **delivered on merge of the B-7 PR** (Schedule A); see [slice record](inspection-size-warning.md).
  With B-6 this completes roadmap Gate 1a; no ready slice remains.

- Behavior: when the inspected file exceeds 500 KB (512,000 bytes), `wsr inspect` still succeeds and writes a
  warning to stderr that GitHub would not run this workflow (documented 500 KB limit); stdout is unchanged.
  No file-size rejection.
- Acceptance: CLI tests at 511,999/512,000/512,001-byte sizes (exit 0, warning only above the limit, identical stdout).
- Expected red: no warning for a 512,001-byte workflow.
- Files: `crates/wsr/src/commands/inspect.rs`, `crates/wsr/tests/inspection.rs`, `docs/cli/README.md`.

## B-8 Expression category by schema context (audit F13, F14, F16)

- Status: **delivered** in the Schedule B architecture PR `arch/review-2026-11-13-run2`; see
  [slice record](inspection-classification.md).

- Behavior: an expression in a field the pinned schema gives no expression context is `Invalid`, where a
  context exists it stays `Unsupported`; schema-valid `timeout-minutes: 0`, empty `working-directory` and empty
  `run` are `Unsupported`; a job ID is validated before its body so it cannot forge a route.

## B-9 Escape format characters in inspection stderr (audit F15)

- Status: **delivered on merge of the B-9 PR** (Schedule A); see [slice record](inspection-format-escape.md).

- Behavior: host stderr escaping (`crates/wsr/src/commands/inspect.rs` `escape_control_characters`) also escapes
  Unicode format/bidi characters (general category Cf, e.g. U+202E, U+200B, U+2066–U+2069) the way stdout's
  `escape_debug` does, leaving quotes and backslashes as today.
- Acceptance: CLI tests with U+202E and U+200B in an unknown step key and in a job ID never appear raw on
  stderr; existing escaping tests unchanged.
- Expected red: raw bytes on stderr.
- Files: `crates/wsr/src/commands/inspect.rs`, `crates/wsr/tests/inspection.rs`, `docs/cli/README.md`.

## B-10 Guard the dogfood workflow set (audit F20)

- Status: **delivered on merge of the B-10 PR** (Schedule A); see [slice record](inspection-dogfood-guard.md).

- Behavior: `crates/wsr/tests/dogfood.rs` enumerates `.github/workflows/example-*.yml` and asserts the set
  equals the golden table keys, so a new example workflow cannot go without golden coverage.
- Acceptance: the guard passes on the current two files; a temporary extra fixture in a temp copy (not in the
  repo) makes it fail.
- Expected red: none (characterization guard); prove with the temp-copy fixture.
- Files: `crates/wsr/tests/dogfood.rs`, `docs/testing/coverage.md`.

## B-11 Workflow and job `env` and job `defaults.run`

- Status: **delivered on merge of the B-11 PR** (Schedule A); see [slice record](inspection-env-defaults.md).

- Behavior: inspect workflow-level and job-level `env` maps and workflow/job `defaults.run.shell` and
  `working-directory` as source data (literal or unevaluated `SourceText`, sorted keys, escaped), reported as not
  bound and not applied, following B-5. D-033 label for any `wsr-types` additions. Categories follow B-8: the
  schema expression context decides Invalid vs Unsupported; check the workflow `env` and `defaults` contexts in
  the pinned schema.
- Acceptance: per-field CLI tests, `wsr-gha` category tests, hostile-content sentinel.
- Expected red: unknown field `env` / `defaults`.
- Files: `crates/wsr-gha/src/lib.rs` (and `step_inputs.rs` reuse), `crates/wsr-types/src/inspection.rs`,
  `crates/wsr/src/commands/inspect.rs`, tests, `docs/cli/README.md`.

## B-12 Job and step `if` as unevaluated conditions

- Status: **delivered on merge of the B-12 PR** (Schedule A); see [slice record](inspection-conditions.md).
  With B-9..B-12 delivered, no ready slice remains.

- Behavior: replace the current `Unsupported` rejection of job and step `if:` with a successful report of the
  condition as unevaluated source text (bare and `${{ }}` forms, both shown as written), stating it is not
  evaluated and does not affect inspection output. No expression parsing.
- Acceptance: CLI tests for bare, fenced and hostile text; the former `Unsupported` category test in `wsr-gha` is
  replaced with a success assertion (cite this slice as the contract change).
- Expected red: the current `Unsupported` `if` diagnostic.
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-types/src/inspection.rs`, `crates/wsr/src/commands/inspect.rs`,
  tests, `docs/cli/README.md`.

## B-13 Scalar conversion categories (audit F21, F22)

- Status: **delivered** in the Schedule B architecture PR `arch/review-2026-11-13-run3`; see
  [slice record](inspection-scalar-conversion.md).

- Behavior: the pinned reader converts an unmatched null, boolean or number to a string and validates it, so a
  conversion the schema accepts is `Unsupported` ("non-string value is not inspected yet") and one the schema or
  runner rejects stays `Invalid`, decided per field (`name`, `run-name`, `run`, `working-directory`, `runs-on`
  and selectors, dispatch `description`/`options`/null `default`, `env`/`with` nulls and a boolean step `id` move
  to `Unsupported`; `defaults.run.shell` numbers and booleans move to `Invalid`). `Condition:` lines also say the
  expression syntax is not validated (F23). Output of successful inspections is unchanged.

## B-14 Shell and action-reference semantic validation (audit F28)

- Status: **delivered on merge of the B-14 PR** (Schedule A); see [slice record](inspection-shell-uses.md).

- Behavior: step `shell` and `defaults.run.shell` literal text that is not a built-in shell (`bash`, `pwsh`,
  `python`, `sh`, `cmd`, `powershell`) and has no `{0}` placeholder is `Invalid` (the runner's ScriptHandler
  rejects it; the B-8 semantic-rejection allowance). Literal `uses` text that is not `./…`, `docker://…` or
  `owner/repo[/path]@ref` is `Invalid`. Keep stdout empty and exit 1 on failure, deterministic output, no
  evaluation, binding or execution claims, and a hostile-content escaping test.
- Acceptance: `wsr-gha` category tests for `shell: "true"`, `shell: "5"`, `shell: "perl {0}"` (accepted),
  `uses: "5"` and `uses: "actions/checkout"` (no ref, Invalid); dogfood goldens unchanged.
- Expected red: quoted `"true"` shell and `"5"` uses currently inspect with exit 0.
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-gha/src/defaults.rs`, tests, `docs/cli/README.md`.

## B-15 Non-dispatch `on:` events with empty bodies, starting with `workflow_call`

- Status: **delivered on merge of the B-15 PR** (Schedule A); see [slice record](inspection-trigger-events.md).

- Behavior: report `Trigger: <event> (not evaluated)` for scalar, sequence and null or empty-body mapping forms
  of non-dispatch events, in source order; non-empty bodies (filters, `workflow_call` inputs and secrets) stay
  `Unsupported`; unknown event names are categorized from `on` strict versus `on` mapping per B-8. Label any new
  `wsr-types` summary next to `ManualTrigger` as D-033 temporary GHA-derived types. Keep stdout empty and exit 1
  on failure, deterministic output, no evaluation, binding or execution claims, and a hostile-content escaping test.
- Acceptance: CLI goldens plus `wsr-gha` category tests; the repository's `check-docs`, `check-fmt`,
  `check-lint` and `test` workflows (as temporary copies only, never in the real workflow directory) then inspect.
- Expected red: `on.workflow_call: unsupported field or event`; `on: unsupported trigger; only workflow_dispatch is
  inspected yet`; `on: expected a mapping (other trigger forms are unsupported)`.
- Files: `crates/wsr-gha/src/dispatch.rs`, `crates/wsr-types/src/inspection.rs`,
  `crates/wsr/src/commands/inspect.rs`, `docs/cli/README.md`.

## B-16 Workflow and job `concurrency`

- Status: **delivered on merge of the B-16 PR** (Schedule A); see [slice record](inspection-concurrency.md).
  The pinned mapping also has `queue` (`single`/`max`), which is inspected.

- Behavior: a string or `{group, cancel-in-progress}` (check the pinned schema for any further keys) is reported
  "(not applied)"; expressions are `Unsupported` or unevaluated, since `workflow-concurrency` and
  `job-concurrency` have contexts; an empty `group` is `Invalid`; the string `"true"` for `cancel-in-progress` is
  `Invalid`. Scalar conversions follow B-13. Keep stdout empty and exit 1 on failure, deterministic output, no
  evaluation, binding or execution claims, and a hostile-content escaping test.
- Acceptance: per-level CLI tests, `wsr-gha` category tests, hostile-content sentinel.
- Expected red: ``unknown field `concurrency` `` at both levels.
- Files: `crates/wsr-gha/src/lib.rs` plus a new module, `crates/wsr-types/src/inspection.rs`,
  `crates/wsr/src/commands/inspect.rs`, tests.

## B-17 Job and step `continue-on-error`, step `timeout-minutes`

- Status: **READY** (Schedule A).

- Behavior: job and step `continue-on-error` take a boolean, step `timeout-minutes` a number (reuse the job
  timeout decoder for it only); either may instead be `${{` text, reported unevaluated; all are reported
  "(not applied)". The contexts exist (`boolean-strategy-context`, `step-continue-on-error`,
  `step-timeout-minutes`; pinned schema L2308-2325). Keep stdout
  empty and exit 1 on failure, deterministic output, no evaluation, binding or execution claims, and a
  hostile-content escaping test.
- Acceptance: CLI tests, `wsr-gha` category tests including rejection of `continue-on-error: 5` and
  `timeout-minutes: true`; `crates/wsr-gha/tests/diagnostics.rs` uses `continue-on-error`
  as its `Unsupported` sample, so pick a new sample key there and record it as an intended contract change.
- Expected red: ``unknown field `continue-on-error` `` and `` `timeout-minutes` `` (`Unsupported`).
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-types/src/inspection.rs`, `crates/wsr/src/commands/inspect.rs`,
  `crates/wsr-gha/tests/diagnostics.rs`, tests, `docs/cli/README.md`.

## B-18 Job `outputs`

- Status: **READY** (Schedule A).

- Behavior: an ordered map whose values are unevaluated `string-runner-context` text, reported "(not resolved)";
  an expression replacing the whole mapping is `Invalid` (`job-outputs` has no context); scalar values follow
  B-13. Keep stdout empty and exit 1 on failure, deterministic output, no evaluation, binding or execution
  claims, and a hostile-content escaping test.
- Acceptance: CLI tests, `wsr-gha` category tests, hostile-content sentinel.
- Expected red: ``jobs.b: unknown field `outputs` ``.
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-types/src/inspection.rs`, `crates/wsr/src/commands/inspect.rs`,
  tests, `docs/cli/README.md`.

## B-19 Job `environment`

- Status: **READY** (Schedule A).

- Behavior: a string or `{name, url}` (check the pinned schema), reported "(not selected; no protection rules
  evaluated)"; `name` is required; contexts per the schema decide `Invalid` versus `Unsupported` as in B-8. Keep
  stdout empty and exit 1 on failure, deterministic output, no evaluation, binding or execution claims, and a
  hostile-content escaping test.
- Acceptance: CLI tests, `wsr-gha` category tests, hostile-content sentinel.
- Expected red: ``unknown field `environment` ``.
- Files: `crates/wsr-gha/src/lib.rs`, `crates/wsr-types/src/inspection.rs`, `crates/wsr/src/commands/inspect.rs`,
  tests, `docs/cli/README.md`.
