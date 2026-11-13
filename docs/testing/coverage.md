# Behavioral test coverage

```mermaid
flowchart LR
    S[Reviewed source scope] --> B[Behavioral tests]
    B --> E[Observed revision and result]
    E --> G[Limits and gaps]
```

## Current status — November 5, 2026

The October 26 coverage work landed through [PR #16](https://github.com/ectorial/wsr/pull/16)
in merged baseline `a64136113d0a42f20c96a585ead5cc76dcbe0a94`.
[PR #17](https://github.com/ectorial/wsr/pull/17) added design documentation at
`031b583e60ce93b6b0af3b41afdec02b393aedff`. No ready failing implementation task is recorded.
Product execution remains scaffold. Dates below identify observed evidence, not fresh test runs.

The [Java checkpoint](../implementation/java-progress.md#current-checkpoint--november-5-2026)
owns the current worker handoff under the [Java contract](../implementation/java-e2e.md).
The earlier test-preparation-only mandate is superseded. Review live checkout/remote state and
owner stop/hold instructions before selecting work.

## October 26 review provenance

The product scope below was reviewed against preparation base
`faf418eb0afd4cc567ad9b69cd6414a88bde1c47`. Preserve its observation dates and limits.

## Reviewed product scope

| Source scope | Tested observable behavior and tests | Focused command | Status and limits |
| --- | --- | --- | --- |
| [wsr-cli](../../crates/wsr-cli/src/lib.rs), including run/cache/hook/daemon arguments | [Parser tests](../../crates/wsr-cli/tests/cli.rs): command tree, required arguments, run defaults/options/formats, rejection of invalid values/options, Unix OS-string preservation for inspect and run. Run regression also covers a leading-dash filename after `--`. | `cargo test -p wsr-cli --test cli --locked` | Passed during October 26 preparation; available on main through PR #16. New regression already passes existing code; no red phase. Unix path checks are conditional. No file access or execution coverage. |
| [wsr entrypoint](../../crates/wsr/src/lib.rs), binary and all eight [handler modules](../../crates/wsr/src/commands/mod.rs) | [Executable tests](../../crates/wsr/tests/cli.rs): all eleven operational paths exit 1 with exact placeholder diagnostic; help/version exit 0; invalid arguments exit 2; disposable repository sentinels remain unchanged. | `cargo test -p wsr --test cli --locked` | Passing October 26. Placeholder checks protect scaffold behavior only; they are not implementation acceptance tests for future operations, and do not prove isolation. |
| [wsr corpus bridge](../../crates/wsr/tests/corpus.rs) | Invokes Python harness regressions and offline baseline from an unrelated working directory. Catalog integrity, pinned snapshot digests, owned Python native pass/failure checks. | `cargo test -p wsr --test corpus --locked` | Passing October 26. Requires Python 3.11+ and Git. [Catalog](../../tests/cases.toml), [fixtures](../../tests/fixtures), [snapshots](../../tests/ecosystem), and examples are supporting inputs. No upstream execution/download or WSR execution. Python harness internals were not exhaustively reviewed here. |
| [wsr-gha scaffold](../../crates/wsr-gha/src/lib.rs) and [example guards](../../crates/wsr-gha/tests/workflow_examples.rs) | All example YAML workflows have only manual dispatch and nonempty jobs; exact shell workflow script produces native pass/failure markers and exit status with explicitly bound mode. | `cargo test -p wsr-gha --test workflow_examples --locked` | Passing October 26. Requires POSIX sh. Uses canonical [shell](../../.github/workflows/example-shell.yml) and [Python](../../.github/workflows/example-python.yml) YAML. No GHA expression evaluation, translation, WSR engine or isolation evidence. |
| [wsr-types](../../crates/wsr-types/src/lib.rs) | No behavioral tests present. Early structs/serde derives, trait and errors inspected. | `cargo test -p wsr-types --locked` | Build/doctest baseline only. Canonical schema and provider interface are not settled; avoid freezing legacy serialization as an accepted future contract. |

The remaining twelve library entrypoints were inspected and contain documentation-only
placeholders: `wsr-bench`, `wsr-cache`, `wsr-client`, `wsr-engine`, `wsr-expr`, `wsr-fs`,
`wsr-git`, `wsr-resolver`, `wsr-sandbox`, `wsr-shell`, `wsr-tracing`, `wsr-wasix`.
Their source is `crates/<name>/src/lib.rs`; no behavioral tests exist. Each can be
checked with `cargo test -p <name> --locked`, but an empty passing suite establishes
only build compatibility. No execution, resolution, filesystem, security, or performance
claim follows. All are included in the workspace baseline below.

## November 6 static inspection slice

`cargo test -p wsr --test inspection --locked` exercises the real binary against temporary source
files: static summaries, deterministic job order and source step order, unreadable/malformed YAML,
invalid/unsupported fields, duplicate identifiers, and unevaluated-expression rejection. A malicious
`run` script does not execute; an action reference is not opened; repository sentinels and the
file inventory remain unchanged. Remaining placeholder assertions continue in `tests/cli.rs`;
inspection assertions are replaced by actual behavior, not removed without coverage.

Observed red: the first acceptance test failed on `error: wsr inspect: not implemented yet`.
Subsequent red/green checks exposed dynamic runner acceptance, empty jobs, and missing manual-trigger
support. Focused tests and `cargo xtask ci` passed after implementation. The October audit table
above remains dated evidence; its eleven-placeholder and GHA-scaffold observations no longer
represent this revision. See [supported subset](../cli/README.md#static-workflow-inspection).
This is not full GHA lowering, executable canonical IR, Component-provider or workload execution evidence.

## Repository tooling and areas not reviewed

Private `tools/xtask` has two parser unit tests in [cli.rs](../../tools/xtask/src/cli.rs),
run with `cargo test -p xtask --locked`. They cover canonical/legacy command parsing
and tool-group defaults. Other xtask modules, Python tooling/scaffold regression scripts,
release workflows and policy correctness have not received a behavioral audit in this run.
The full baseline running their Cargo paths does not make them reviewed. Preserved prototype
code/stash was not restored or tested.

[Native hosted evidence](../../tests/reference-runs.md) remains dated reference evidence.
No workflows were dispatched during this review. Canonical example YAML must be reused
for future real translation tests and later execution tests; do not fork local workflow copies.

## Historical October 26 implementation-preparation proposal

This section preserves the October 26 proposal and its then-current instructions. It is not
the active worker handoff. At that review, **ready failing tasks: none; pending failing branch/PR: none.** PRs
#12 and #13 contain dependency updates, not implementation handoffs. No additional speculative
implementation suite should be accumulated.

The smallest next decision is to approve a first translation-only observable contract:
which public entrypoint and result/diagnostic schema should represent one manual-dispatch
workflow containing one `sh` run step? Recommendation: accept the canonical shell example
as the positive input, retain its manual mode input/environment binding and source provenance,
and reject unsupported constructs explicitly without running commands. Settle the minimal
result fields, expression treatment and diagnostic expectations before choosing assertions.
This is a recommendation, not an approved interface or ready task. The legacy
`WorkflowProvider`/`WorkflowIR` declarations do not settle it; [PLAN.md](../context/decisions.md)
D-003 through D-005 and its implementation gate remain authoritative.

Once approved, prepare exactly one ordinary Rust integration test in `wsr-gha/tests/`
using that real entrypoint and canonical YAML. Record its exact command and observed
behavioral assertion failure here and on its actual work branch/draft PR. Compilation/setup
failures or generic CLI placeholder errors do not qualify. Non-goals: engine/runtime,
execution, isolation, backend selection, plugin host, new framework and broad IR design.
Follow [the handoff contract](strategy.md#11-tdd-and-agent-implementation-contract),
including prerequisite decisions, negative fixture, regression commands and completion evidence.
The then-current scheduled test-preparation scope prohibited production implementation. The
November 1 Java worker authorization superseded that scope for settled, bounded TDD slices.

## Historical October 26 validation and resumption

October 26 review branch: `cargo xtask ci` passed (format, locked compilation/Clippy,
workspace tests including corpus/native references, and strict rustdoc); `git diff --check`
passed. The focused product suites above are included in that run. No failing implementation
test is concealed, ignored or wrapped as expected failure. Hosted CI for the base main revision
[40261792393](https://github.com/ectorial/wsr/actions/runs/40261792393) succeeded; branch
CI is separate and must be checked on its PR before any merge. Auto-merge remains disabled.

At that checkpoint, the next-run instruction was to inspect the preparation branch/PR and preserve dirty work.
Keep useful passing regression work bounded while the translator contract is unsettled.
Update this index only for material changes, and do not repeatedly notify the same blocker.


## Java reference input preparation — November 2, 2026

The [manual preparation checkpoint](../implementation/java-progress.md#november-2-manual-preparation--downloaded-inputs-only)
records a complete pinned `junit-jupiter-starter-maven` source subtree, upstream license/workflow,
verified GitHub commit/tree/blob identities and SHA256 manifest under ignored
`target/wsr-tests/java/manual-preparation-20261102/`. Original input commit:
`junit-team/junit-examples@5bd6594a2336f1eff52527733ba28682a3215d83`.
An explicitly derived manual Maven workflow is review input only; action metadata is captured,
not qualified under Bun. Parent Astra orchestrated a requested Sol fixture/docs delegation;
see the checkpoint for assignment verification limits and preserved branch/base.

`cargo xtask ci` passed (exit 0); `git diff --check` passed. Raw logs/result records are in that
ignored evidence directory. Read-only `docker info` failed (exit 1, absent desktop-linux daemon
socket); no daemon was started. Native Java pass/fail, JAR/Surefire verification, WSR translation
and execution, isolation and cache behavior remain unexecuted. Download/setup success is not
behavioral red/green. Next command: `docker info`; qualification and exact image/JDK/Maven
artifact verification must precede any upstream execution. No product behavior or test changed.


## Linux Java reference prerequisites — November 3, 2026

The [qualification checkpoint](../implementation/java-progress.md#november-3-linux-reference-prerequisite-qualification)
records an available Docker Desktop Linux ARM64 daemon and digest-pinned BellSoft JDK25 image.
Ignored evidence `target/wsr-tests/java/linux-qualification-20261103/commands.json` preserves exact
commands; image/container inspection, JDK hashes, probe/cleanup logs and CI results accompany it.
Two disposable UID10001 containers verified bounded CPU/memory/PID settings, read-only root,
zero capabilities, no-new-privileges/seccomp, no bind/volume/socket/credential mounts, denied
outbound networking and fresh private tmpfs. Containers were removed; final filtered list empty.
This is reference-environment prerequisite evidence, not a hostile-code security audit or WSR
sandbox implementation. `cargo xtask ci` and `git diff --check` passed. Requested Sol delegation
and runtime-verification limits are recorded in the checkpoint. Maven distribution/dependency
verification, native pass/fail and JAR/Surefire evidence remain pending. No upstream Maven/script,
WSR translation/execution, action/Bun, caching or product red/green behavior was exercised.


## Maven reference inputs — November 4, 2026

The [Maven preparation checkpoint](../implementation/java-progress.md#november-4-maven-input-preparation--no-workload-execution)
records exact Maven 3.10.0 ZIP acquisition, matching upstream SHA512/SHA1, and a valid detached
signature against HTTPS-fetched official Maven KEYS (independent signer trust not established).
Ignored `target/wsr-tests/java/maven-inputs-20261104/` retains input provenance, archive inventory,
GPG command/results and the restricted dependency acquisition plan. No archive extraction,
Maven/upstream script execution or dependency resolution occurred. Reuse of the qualified
image-contained JDK is reference-only. Next prerequisite is qualification of restricted acquisition
and container input delivery; native pass/fail and all Java/WSR product acceptance lanes remain pending.

Validation for this slice: `cargo xtask ci` exit 0 (format, locked compilation/Clippy, workspace
tests and strict docs); `git diff --check` passed. Logs/results remain in the ignored evidence root.
This is repository regression evidence only; no Java behavioral red/green claim.

## Java offline dependency preparation — November 4, 2026

The [dependency checkpoint](../implementation/java-progress.md#november-4-dependency-preparation--bounded-offline-discovery)
qualifies validated source/Maven transfer and wrapper bootstrap inside the pinned Linux ARM64
reference container. All fetched workload execution remained nonroot and network-none, with
explicit private settings/repository and prior resource restrictions; no host Maven ran.
Explicit fixed Central HTTPS acquisition retained 51 checksum-checked POM/JAR artifacts
(2,745,759 bytes). This is a partial inventory, not a resolved dependency closure.

Ignored `target/wsr-tests/java/dependency-preparation-20261104/` preserves exact scripts/argv,
per-attempt failures, artifact URLs/hashes, delivery-integrity checks, runtime identities and cleanup.
Wrapper `--version` passed; final offline `clean verify` exited 1 during resources-plugin descriptor
resolution for maven-filtering 3.5.0 and plexus-utils 3.6.0. The checkpoint identifies their reviewed
missing parents and exact resume command. Compilation, tests, native pass/fail, Surefire/JAR and
all Java/WSR product acceptance lanes remain pending; setup failure is not behavioral red.

`CARGO_NET_OFFLINE=true cargo xtask ci` and `git diff --check` passed. This is repository regression
and reference-prerequisite evidence only.

## Static dependency-inspection iteration

Real-binary acceptance tests in `crates/wsr/tests/inspection.rs` now cover scalar/list `needs`,
no-edge output preservation, empty lists, forward references, chains, diamonds, disconnected jobs,
repeated edges, unknown targets, self-cycles and longer cycles. A cycle's downstream dependents
are excluded from its reported path. Malformed types and expressions fail explicitly before stdout.
A 4,096-job chain exercises nonrecursive traversal; hostile scripts/local action references and
repository/config/hook/cache sentinels verify that graph validation does not execute or acquire inputs.

Observed red/green: the shared valid fixture first failed on unknown `needs`; missing-target,
self-cycle and longer-cycle tests each failed because invalid graphs initially returned success,
then passed after their corresponding checks. The old unknown-field assertion for `needs` is
replaced by a semantic unknown-target assertion. All other existing inspection and placeholder
assertions remain. `cargo xtask ci` passes with 13 inspection acceptance tests; this is static
source/graph evidence, not execution, security enforcement or complete GHA compatibility.

## Explicit step-identity iteration

The real CLI acceptance suite now has 17 inspection tests. New red/green checks first observed
unknown `id` rejection, then acceptance of invalid/duplicate IDs, and passed after decoding,
validation and duplicate checks were implemented. Tests verify IDs on run/uses steps, unchanged
unidentified-step output/order, malformed types, expressions, syntax/reserved-prefix rejection,
99/100-character boundaries, same-job duplicate locations, cross-job reuse and case preservation.
The shared `tests/fixtures/inspection/step-identities.yml` includes a malicious command and an
unopened local action reference; source bytes/file inventory remain unchanged. Existing graph,
CLI parser and remaining-placeholder regressions still pass. `cargo xtask ci` passed. This is
static source/identity evidence, not output binding, expression evaluation or execution support.

## Literal runner-label iteration

The real CLI suite now has 20 inspection tests. Observed red: a valid list failed with the old
string-only decoder; after list support, an empty list incorrectly succeeded. Both went green.
New assertions cover scalar/list output, ordered/repeated/case-sensitive labels, quoting and
control characters, strict types, indexed blank/expression diagnostics, unsupported mappings,
steps/needs integration and unchanged source/config sentinels. A GHA public-API integration test
checks scalar versus list source-summary variants independently of rendering. Existing tests remain.
These establish source inspection only, not runner matching, provisioning or execution.

## Manual-trigger input iteration

25 real CLI inspection tests (plus 5 CLI regressions) and two new public parser-summary tests
cover dispatch mappings/input declarations. Observed red/green: mapping rejected by old string-only
parser; missing choice options accepted before cross-field checks; malformed identifiers accepted
before validation. New cases include five types/omitted type, strict typed defaults, options,
empty/null declaration states, deterministic input order, duplicate keys, expressions, unknown
metadata/events, indexed diagnostics, escaped text and unchanged repository/source sentinels.
No existing assertions were removed. Source summaries retain declaration states independently of
rendering. Numeric defaults are preserved rather than dropped; this is not complete input validation
or event binding. Existing examples still require other source fields before end-to-end acceptance.

## Run-name iteration and host hardening

The real CLI inspection suite (`crates/wsr/tests/inspection.rs`) now lists 31 tests, verified with
`cargo test -p wsr --test inspection -- --list`; `tests/cli.rs` keeps 5 placeholder/parser tests.
The run-name iteration ([PR #27](https://github.com/ectorial/wsr/pull/27), `fd574dc`) took the suite
to 28, plus the public parser-summary test in `crates/wsr-gha/tests/run_name.rs`. Observed red: three
new CLI tests failed with `unknown field run-name`. Coverage: literal, omitted and unevaluated
(`${{`) text, wrong shapes (null, boolean, number, sequence, mapping) and blank text rejected before
rendering, and control-character escaping. Run-name text is reported, never evaluated or bound to inputs.

Three host-boundary tests followed the architecture review ([log](../implementation/architecture-review.md),
F2/F7/F8). Observed red before implementation: an ESC byte in an unknown top-level key reached stderr raw
(serde's unknown-field message); a 5,000-job report piped to an immediately closed stdout panicked with
`failed printing to stdout: Broken pipe`; and a symlink to `/dev/zero` was read until the test's 2 s guard
killed it. After the fixes the tests assert: no 0x1b byte on stderr for top-level, job, step, `on:` and
job-identifier locations (exit 1, empty stdout); exit 0 with empty stderr and no panic on a closed pipe;
and exit 1 with `not a regular file` for a `/dev/zero` symlink, a directory and (when `mkfifo` exists)
a FIFO. The non-regular-file test is `#[cfg(unix)]`. These establish static-input hardening only, not a size limit
(an open [decision request](../implementation/decision-requests.md)), execution or sandboxing.

## Provisional typed inspection errors (backlog B-0)

`crates/wsr-gha/tests/diagnostics.rs` (4 tests) calls `wsr_gha::inspect` directly and asserts the
provisional `Category` and `route()` of failures. Observed red: with every failure first classified
`Invalid` without a route, three tests failed on `(Invalid, None)` assertions (for example
`jobs.b.if`, `name`, `jobs.b.bogus`); the location test passed as characterization. Coverage:
schema-listed but uninspected keys at workflow/job/step level (`permissions`, job `if`, step `shell`),
known `on` events in scalar, sequence and mapping forms, runner-selector mappings and expressions in
literal-only fields are `Unsupported`; unknown keys/events, unknown input types, unknown dependencies,
empty jobs and parse failures stay `Invalid`. Line/column locations and duplicate-key messages are
retained. A 53-case stderr/exit corpus of `wsr inspect` diagnostics was byte-identical before and
after on `3793749`; the 31 CLI inspection tests are unchanged. Classification is not a stable contract.
Cross-review (Codex) found two over-broad `Unsupported` classifications; red cases `on: [push, bogus]`
and `runs-on: {bogus: linux}` now stay `Invalid` unless every event is a schema event and every
selector key is a well-shaped `group`/`labels`.

## Owning-crate inspection compiler tests (backlog B-2)

`crates/wsr-gha/tests/compiler.rs` (8 tests, `cargo test -p wsr-gha --test compiler --locked`) calls
`wsr_gha::inspect` directly and asserts summary structure, category and typed route for the
dependency graph (diamond, repeated edge, forward reference, missing, self and three-job cycle),
job/step identifier rules, job-ID precedence over `needs`, unknown-target precedence over cycles, runner-list and step-kind summaries,
shape errors, and hostile YAML (alias expansion, deep nesting). Characterization only: no red was
expected or observed. It covers static source inspection, not evaluation, execution or isolation.

## Requested permissions (backlog B-3)

Two CLI tests in `crates/wsr/tests/inspection.rs` and three in `crates/wsr-gha/tests/permissions.rs`.
Observed red: both CLI tests failed with ``unknown field `permissions` `` before implementation.
Coverage: `read-all`, `write-all`, empty and populated mappings at workflow and job level with sorted
scope output and no merging; invalid scalars, null, sequences, unknown, non-string and escaped
control-byte scopes, scope-restricted levels (`models`, `id-token`, `vulnerability-alerts`,
`copilot-requests`), non-string levels, expressions and duplicate scopes, with category `Invalid`
and field routes. The B-0 unsupported-key case moved from `permissions` to `concurrency`, and the
job unknown-field message now lists `permissions`. Requests are reported, never granted or enforced.

## Job timeouts and step shells (backlog B-4)

Two CLI tests in `crates/wsr/tests/inspection.rs` and two in `crates/wsr-gha/tests/timeout_shell.rs`.
Observed red: both CLI tests failed with ``unknown field `timeout-minutes` `` before implementation.
Coverage: integer, fractional and unevaluated timeouts; literal and escaped hostile shell text; exact
stderr for boolean, negative, zero, `.inf`, string, null and sequence timeouts and for null, numeric,
blank, expression and `uses`-step shells. The step/job unknown-field messages now list `shell` and
`timeout-minutes`; the B-0 unsupported step-key case moved from `shell` to step `timeout-minutes`.
Values are reported, never applied, selected or invoked.

## Step environment, action inputs and working directories (backlog B-5)

Two CLI tests in `crates/wsr/tests/inspection.rs` and three in `crates/wsr-gha/tests/step_inputs.rs`
(the last grown after cross-review).
Observed red: both CLI tests failed with ``unknown field `with` `` before implementation. Coverage:
string, number, boolean and unevaluated values with sorted keys; empty mappings; literal and
unevaluated working directories; a hostile `$(touch executed)` value with an ESC byte rendered escaped
and the directory sentinel unchanged; exact stderr for non-mapping, null and nested values, empty,
numeric and expression keys, whole-mapping expressions, `with` on a run step, `working-directory`
on a uses step and non-string/blank directories; case-insensitive duplicate keys; placement and
run/uses/shell precedence over contents; hostile keys, values and directories with control bytes,
quotes and backslashes in stdout and stderr; exact duplicate keys keep decoder evidence. The B-1 route
test's unknown step key moved from `with` to `bogus`. Values are never bound, passed or resolved.

## Dogfood inspection acceptance (backlog B-6)

`crates/wsr/tests/dogfood.rs` asserts complete stdout, empty stderr and exit 0 for the real
`example-python.yml` and `example-shell.yml`, via the real path and a copy beside a `wsr.toml`
sentinel, with `PATH` replaced by marker-writing `sh`/`bash`/`python`/`python3`/`git`/`pwsh`; no marker,
unchanged directory and unchanged sources. No red on current main; observed red on pre-B-5 `85a1c30`
(``unknown field `with` ``). Translation-only: not WSR execution, isolation or GHA compatibility.

## GitHub workflow-size warning (backlog B-7)

`inspect_warns_above_the_github_workflow_size_limit_and_still_reports` writes valid workflows of
exactly 511,999, 512,000 and 512,001 bytes and asserts exit 0, identical stdout and a stderr warning
only above 512,000 bytes. Observed red: no warning at 512,001 bytes. The closed-stdout test's
638,896-byte fixture now asserts exactly the warning on stderr instead of empty stderr. A closed-stderr
test (red: exit 101, empty stdout) asserts exit 0 and the full report when the warning cannot be written.

## Scalar conversion categories (backlog B-13)

`crates/wsr-gha/tests/scalar_conversion.rs` (12 tests, `cargo test -p wsr-gha --test scalar_conversion --locked`)
asserts category and route for the null, boolean and number rows of the
[slice record](../implementation/inspection-scalar-conversion.md#target-table): `name`, `run-name`, step `run`
and `working-directory`, `shell`, `defaults.run`, `runs-on` and selectors, dispatch `description`,
`options`, `type`, `required` and `default` by input type, `env`/`with` nulls, step `id`, and the rows that
stay as they are, plus summary assertions that `5` and `"5"` and missing and null stay distinct. Eight
tests were red on assertions before the change (`expected Unsupported at Some("jobs.b.runs-on"), got
Invalid …`); four are green characterization. `diagnostics.rs`, `env_defaults.rs`, `step_inputs.rs` and
eight CLI inspection tests were updated as the B-13 contract change. The B-12 CLI goldens now expect
`(not evaluated; expression syntax not validated)` after a red change of the expected text; the dogfood
goldens are unchanged. The CLI inspection suite lists 45 tests.

## Job environments (backlog B-19)

`crates/wsr-gha/tests/environment.rs` asserts kept `Environment` values for four forms, nine `Invalid`
shapes and seven `Unsupported` conversions or empty names. Two CLI tests pin a two-job golden with
hostile name text escaped plus an `executed` sentinel, and five exact diagnostics. Observed red:
``jobs.b: unknown field `environment` ``. Contract change: the job "expected one of" list in
`diagnostics.rs` names `environment`. The CLI inspection suite lists 57 tests.

## Job outputs (backlog B-18)

`crates/wsr-gha/tests/outputs.rs` asserts kept sorted text (literal, empty, unevaluated; `{}`; absent),
eleven `Invalid` shapes (mapping and key expressions, non-mappings, empty, scalar and case-duplicate
keys, nested values) and three `Unsupported` value conversions. Two CLI tests pin a golden with hostile name and
value text escaped plus an `executed` sentinel, and four exact diagnostics. Observed red:
``jobs.b: unknown field `outputs` ``. Contract change: the job "expected one of" list in
`diagnostics.rs` names `outputs`. The CLI inspection suite lists 55 tests.

## Continue-on-error and step timeouts (backlog B-17)

`crates/wsr-gha/tests/continue_timeout.rs` asserts kept values (boolean and unevaluated
`continue-on-error` at both levels; integer, fractional and unevaluated step timeouts), five
`Invalid` `continue-on-error` values at both levels, five `Invalid` step timeouts and a non-positive
`Unsupported` one. Two CLI tests pin a golden with hostile unevaluated text and an `executed` sentinel,
and three exact diagnostics. Observed red: ``unknown field `continue-on-error` `` (Unsupported).
Contract changes: `diagnostics.rs` Unsupported samples moved to job `cancel-timeout-minutes` and step
`name: 5` (a converted scalar), its location test now uses `cancel-timeout-minutes`, and the job and two CLI step
"expected one of" lists name the new keys. The CLI inspection suite lists 53 tests.

## Workflow and job concurrency (backlog B-16)

`crates/wsr-gha/tests/concurrency.rs` asserts kept `Concurrency` values for five forms at both levels,
twelve `Invalid` shapes at both levels plus the job-only empty and null strings, and five `Unsupported`
values at both levels plus the workflow-only empty and null strings.
`inspect_reports_concurrency_without_applying_it` pins a two-level golden with hostile text escaped and
an `executed` sentinel; `inspect_rejects_concurrency_outside_the_inspected_subset` pins six exact
diagnostics. Observed red: ``unknown field `concurrency` ``. Contract changes: the `diagnostics.rs`
workflow Unsupported sample moved from `concurrency` to `description`, and the job "expected one of"
list names `concurrency`. The CLI inspection suite lists 51 tests.

## Trigger events (backlog B-15)

`crates/wsr-gha/tests/triggers.rs` asserts source-order `Trigger` lists for scalar, sequence and
mapping forms, eight `Unsupported` cases (configured events, repeated and empty lists) and eleven
`Invalid` shapes with routes.
CLI tests pin three golden reports (with an `executed` sentinel), seven exact escaped diagnostics, and
successful inspection of temporary copies of `check-docs`, `check-fmt`, `check-lint` and `test`.
Observed red: `on: unsupported trigger; only workflow_dispatch is inspected yet` and `on: expected a
mapping (other trigger forms are unsupported)`. Contract changes: `WorkflowSummary.trigger` became
`triggers` (`dispatch_inputs.rs` updated); the `diagnostics.rs` bare-`push` Unsupported samples became
configured events and sequence routes now name the entry (`on[2]`); four CLI `on` diagnostics changed.
The CLI inspection suite lists 49 tests.

## Shell and action-reference validation (backlog B-14)

`crates/wsr-gha/tests/shell_uses.rs` asserts `Invalid` at the step, workflow-defaults and
job-defaults shell routes for non-built-in shells without `{0}` (including ` bash`), acceptance of
the six built-ins in any case and `{0}` strings, nine malformed versus nine well-formed `uses`
references (runner separator normalization, `.\` paths) and `$/` as Unsupported.
`inspect_rejects_unknown_shells_and_malformed_action_references` pins three exact escaped CLI
diagnostics with an `executed` sentinel. Observed red: `'true'` shell and `'5'` uses inspected Ok.
No existing assertion changed; dogfood goldens unchanged. The CLI inspection suite lists 46 tests.

## Job and step conditions (backlog B-12)

`inspect_reports_job_and_step_conditions_without_evaluating_them` asserts the complete report for
bare, fenced and mixed job/step `if` text, hostile `$(touch executed)`/ESC/bidi text escaped, and an
untouched `executed` sentinel. `inspect_rejects_conditions_outside_the_inspected_subset` pins six
exact diagnostics. `crates/wsr-gha/tests/conditions.rs` asserts kept text and seven category/route
cases (null is Unsupported). Observed red: ``jobs.b: unknown field `if` `` (Unsupported). Contract change: the B-0 job-`if`
Unsupported example in `diagnostics.rs` now uses `continue-on-error`, and the decoders' "expected one
of" lists name `if`.

## Workflow and job env and run defaults (backlog B-11)

`inspect_reports_workflow_and_job_env_and_run_defaults_without_applying_them` asserts the complete
report for workflow and job `env` (sorted, typed, hostile ESC/bidi/zero-width text escaped) and
literal, unevaluated and empty `defaults.run`, with a `touch executed` sentinel untouched.
`inspect_rejects_invalid_workflow_and_job_env_and_run_defaults` pins 17 exact diagnostics.
`crates/wsr-gha/tests/env_defaults.rs` asserts summary values and, for 20 sources, category and route
by schema expression context. Observed red: ``unknown field `env` `` for every case.

## Dogfood workflow-set guard (backlog B-10)

`every_dogfood_example_has_a_golden_case` asserts the sorted `.github/workflows/example-*.yml` names
equal the golden table's keys, so a new example cannot land without golden coverage.
`dogfood_guard_detects_an_example_without_a_golden_case` copies the workflows to a temporary
directory, adds `example-extra.yml` there and asserts the guard sees the mismatch. Characterization
guard: no product red; an untracked `example-zz.yml` placed temporarily in the real directory made
the guard fail, then was removed.

## Format characters on stderr (backlog B-9)

`inspect_errors_escape_unicode_format_characters_on_stderr` puts U+202E and U+200B in an unknown step
key, and U+202E, U+200B, U+2066 and U+2069 in a job identifier (route and quoted text), and asserts
the exact escaped diagnostic with no raw format character on stderr. Observed red: raw
``unknown field `k\u{202e}q\u{200b}` `` bytes on stderr. The control-byte escaping test is unchanged.
