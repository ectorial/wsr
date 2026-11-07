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
