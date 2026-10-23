# Agent instructions

## Start with the workspace CLI

Use `cargo xtask` for repository automation and scaffolding. Prefer an existing
command over manually creating boilerplate, duplicating scripts, or assembling
the same sequence of Cargo commands. Use ordinary Cargo commands for focused
operations that `xtask` does not expose.

Use the canonical commands: `cargo xtask add <semantic-name>` creates a product
crate, `cargo xtask init <capability>` enables optional project capabilities,
and `cargo xtask tools <group>` syncs development tools. The old `scaffold` and
`tools sync` forms exist only for compatibility; do not use them in new commands,
scripts, or documentation.

Before editing, inspect `git status --short`, `README.md`, and the relevant
manifests and source. Read `CONTRIBUTING.md` and any more specific `AGENTS.md`
when present. Discover commands with `cargo xtask --help` and subcommand
`--help`; do not guess flags. Use `rg` and `rg --files` for targeted searches.

State assumptions and the intended verification briefly. Ask when an unresolved
choice changes the requested behavior; resolve routine implementation choices
from the existing code. Keep changes small and preserve unrelated user work.

## Scaffold through commands

Run these from the workspace root, and add only capabilities required by the
task. Use `add` instead of manually creating a crate with `cargo new` or
`cargo init`; it includes the workspace wiring and test examples. Use `init`
for CLI, CI, docs, and agent instructions instead of recreating their boilerplate.
Preview writes with `--dry-run` when their scope needs inspection.

| Need | Command |
| --- | --- |
| Semantic library crate | `cargo xtask add <semantic-name>` |
| Private library crate | `cargo xtask add <semantic-name> --private` |
| Private binary crate | `cargo xtask add <semantic-name> --bin --private` |
| CLI alongside a public library | `cargo xtask init cli --entrypoint companion` |
| CLI executable in the primary crate | `cargo xtask init cli --entrypoint primary` |
| Basic GitHub Actions CI | `cargo xtask init ci` |
| CI with dependency policy and typo checks | `cargo xtask init ci --preset full` |
| Contribution guide and crate index | `cargo xtask init docs` |
| Claude instructions referring to this file | `cargo xtask init agents --claude` |
| Preview a scaffold | `cargo xtask add <semantic-name> --dry-run` |

Scaffolds are idempotent and refuse conflicting user-owned files. Inspect a
conflict and adapt the existing file deliberately; do not delete files or strip
ownership markers just to force a scaffold through. Review the resulting diff.

Keep behavior in the primary crate until extraction gives a coherent ownership
boundary. Name new crates for a domain responsibility, not generic buckets such
as `core`, `common`, or `utils`. Product crates belong under `crates/`;
development automation belongs in `tools/xtask`. Declare dependency versions
once in root `[workspace.dependencies]` and inherit them in member manifests.

## Generated test obligations

Library scaffolds include unit, integration, and documentation examples marked
`TEST_OBLIGATION`. They intentionally fail. Replace the obligations with meaningful
behavioral assertions before expecting a green suite. Binary-only crates include
unit and executable integration tests; Cargo does not run binary doctests.
CLI scaffolds include all three library examples plus an end-to-end test in the
package owning the executable.

After `cargo xtask add` or `cargo xtask init cli`:

1. Run `cargo test --no-fail-fast` and confirm the new obligations fail for their
   intended reason, rather than a compilation or setup error.
2. Replace each `TEST_OBLIGATION` with assertions for the requested behavior,
   including the public API and documentation examples; implement that behavior.
   Do not delete, ignore, or mark tests `should_panic` just to make the suite pass.
3. Run focused tests during implementation, then the applicable `cargo xtask`
   verification command below. Confirm existing tests still pass.

If the task is only scaffolding, leave the obligations failing and report them
explicitly; do not invent domain behavior to remove the failures.

## Verify without repeating work

During iteration, use focused tests, for example `cargo test -p <package> <filter>`.
Format Rust edits with `cargo fmt --all`. Before finishing, choose the command
that covers the change:

| Scope | Command and included checks |
| --- | --- |
| Compilation and lint validation | `cargo xtask check`: formatting, workspace compilation, Clippy |
| Behavior changes | `cargo xtask test`: check, then workspace tests |
| Filtered workspace tests | `cargo xtask test <filter>`: check, then matching tests |
| Release build validation | `cargo xtask build`: test, then release build |
| CI validation | `cargo xtask ci`: test, then documentation build |
| Full configured CI | `cargo xtask ci --full`: CI, dependency policy, typo checks |
| Coverage when needed | `cargo xtask coverage`: HTML coverage report |

Do not run `check`, `test`, and `build` back-to-back when `build` already covers
all three. Repeat checks after further edits or to investigate failures.
For bug fixes, add a regression test that exercises the failing behavior.
Finish with `git diff --check` and review `git diff`; report what changed, which
checks passed, and any check that could not run.

## Tools and releases

Use `cargo xtask doctor` to inspect missing optional tools. When a required tool
is missing, use `cargo xtask tools <group>` with `test`, `coverage`, `ci`,
or `release`. Choose the needed group instead of installing everything. Sync
requires network access, updates the shared stable Rust toolchain, and installs
tools under `.xtask/tools`; do not run it as routine session setup.

For requested release work, use `cargo xtask release init` to set up release
configuration, `cargo xtask release plan` to preview binary distribution, and
`cargo xtask release prepare [patch|minor|major]` for cargo-release's dry run.
Library-only workspaces do not support the distribution plan. Use `--execute`,
commit, push, publish, or create remote resources only when authorized by the
user's task. Prefer an available service CLI such as `gh` for authorized remote
actions; inspect its help and current repository state first.
