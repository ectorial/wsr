# Behavioral test coverage

Reviewed October 26, 2026 against base `faf418eb0afd4cc567ad9b69cd6414a88bde1c47`.
This index records observed behavior, not feature completeness. The test/documentation
change is on `codex/test-coverage-index`; the added run-path regression is not yet on main.
No ready failing implementation task exists. No product behavior was implemented.

## Reviewed product scope

| Source scope | Tested observable behavior and tests | Focused command | Status and limits |
| --- | --- | --- | --- |
| [wsr-cli](../crates/wsr-cli/src/lib.rs), including run/cache/hook/daemon arguments | [Parser tests](../crates/wsr-cli/tests/cli.rs): command tree, required arguments, run defaults/options/formats, rejection of invalid values/options, Unix OS-string preservation for inspect and run. Run regression also covers a leading-dash filename after `--`. | `cargo test -p wsr-cli --test cli --locked` | Passing on review branch, October 26. New regression already passes existing code; no red phase. Unix path checks are conditional. No file access or execution coverage. |
| [wsr entrypoint](../crates/wsr/src/lib.rs), binary and all eight [handler modules](../crates/wsr/src/commands/mod.rs) | [Executable tests](../crates/wsr/tests/cli.rs): all eleven operational paths exit 1 with exact placeholder diagnostic; help/version exit 0; invalid arguments exit 2; disposable repository sentinels remain unchanged. | `cargo test -p wsr --test cli --locked` | Passing October 26. Placeholder checks protect scaffold behavior only; they are not implementation acceptance tests for future operations, and do not prove isolation. |
| [wsr corpus bridge](../crates/wsr/tests/corpus.rs) | Invokes Python harness regressions and offline baseline from an unrelated working directory. Catalog integrity, pinned snapshot digests, owned Python native pass/failure checks. | `cargo test -p wsr --test corpus --locked` | Passing October 26. Requires Python 3.11+ and Git. [Catalog](../tests/cases.toml), [fixtures](../tests/fixtures/), [snapshots](../tests/ecosystem/), and examples are supporting inputs. No upstream execution/download or WSR execution. Python harness internals were not exhaustively reviewed here. |
| [wsr-gha scaffold](../crates/wsr-gha/src/lib.rs) and [example guards](../crates/wsr-gha/tests/workflow_examples.rs) | All example YAML workflows have only manual dispatch and nonempty jobs; exact shell workflow script produces native pass/failure markers and exit status with explicitly bound mode. | `cargo test -p wsr-gha --test workflow_examples --locked` | Passing October 26. Requires POSIX sh. Uses canonical [shell](../.github/workflows/example-shell.yml) and [Python](../.github/workflows/example-python.yml) YAML. No GHA expression evaluation, translation, WSR engine or isolation evidence. |
| [wsr-types](../crates/wsr-types/src/lib.rs) | No behavioral tests present. Early structs/serde derives, trait and errors inspected. | `cargo test -p wsr-types --locked` | Build/doctest baseline only. Canonical schema and provider interface are not settled; avoid freezing legacy serialization as an accepted future contract. |

The remaining twelve library entrypoints were inspected and contain documentation-only
placeholders: `wsr-bench`, `wsr-cache`, `wsr-client`, `wsr-engine`, `wsr-expr`, `wsr-fs`,
`wsr-git`, `wsr-resolver`, `wsr-sandbox`, `wsr-shell`, `wsr-tracing`, `wsr-wasix`.
Their source is `crates/<name>/src/lib.rs`; no behavioral tests exist. Each can be
checked with `cargo test -p <name> --locked`, but an empty passing suite establishes
only build compatibility. No execution, resolution, filesystem, security, or performance
claim follows. All are included in the workspace baseline below.

## Repository tooling and areas not reviewed

Private `tools/xtask` has two parser unit tests in [cli.rs](../tools/xtask/src/cli.rs),
run with `cargo test -p xtask --locked`. They cover canonical/legacy command parsing
and tool-group defaults. Other xtask modules, Python tooling/scaffold regression scripts,
release workflows and policy correctness have not received a behavioral audit in this run.
The full baseline running their Cargo paths does not make them reviewed. Preserved prototype
code/stash was not restored or tested.

[Native hosted evidence](../tests/reference-runs.md) remains dated reference evidence.
No workflows were dispatched during this review. Canonical example YAML must be reused
for future real translation tests and later execution tests; do not fork local workflow copies.

## Pending implementation preparation: blocked on a narrow contract

**Ready failing tasks: none. Pending failing branch/PR: none.** Existing remote PRs
#12 and #13 are dependency updates, not implementation handoffs. No additional speculative
implementation suite should be accumulated.

The smallest next decision is to approve a first translation-only observable contract:
which public entrypoint and result/diagnostic schema should represent one manual-dispatch
workflow containing one `sh` run step? Recommendation: accept the canonical shell example
as the positive input, retain its manual mode input/environment binding and source provenance,
and reject unsupported constructs explicitly without running commands. Settle the minimal
result fields, expression treatment and diagnostic expectations before choosing assertions.
This is a recommendation, not an approved interface or ready task. The legacy
`WorkflowProvider`/`WorkflowIR` declarations do not settle it; [PLAN.md](../PLAN.md)
D-003 through D-005 and its implementation gate remain authoritative.

Once approved, prepare exactly one ordinary Rust integration test in `wsr-gha/tests/`
using that real entrypoint and canonical YAML. Record its exact command and observed
behavioral assertion failure here and on its actual work branch/draft PR. Compilation/setup
failures or generic CLI placeholder errors do not qualify. Non-goals: engine/runtime,
execution, isolation, backend selection, plugin host, new framework and broad IR design.
Follow [the handoff contract](TESTING-PLAN.md#11-tdd-and-agent-implementation-contract),
including prerequisite decisions, negative fixture, regression commands and completion evidence.
Do not implement production behavior as part of scheduled test preparation.

## Validation and resumption

October 26 review branch: `cargo xtask ci` passed (format, locked compilation/Clippy,
workspace tests including corpus/native references, and strict rustdoc); `git diff --check`
passed. The focused product suites above are included in that run. No failing implementation
test is concealed, ignored or wrapped as expected failure. Hosted CI for the base main revision
[40261792393](https://github.com/ectorial/wsr/actions/runs/40261792393) succeeded; branch
CI is separate and must be checked on its PR before any merge. Auto-merge remains disabled.

Next run: inspect live state and this branch/PR before changing branches; preserve dirty work.
Keep useful passing regression work bounded while the translator contract is unsettled.
Update this index only for material changes, and do not repeatedly notify the same blocker.
