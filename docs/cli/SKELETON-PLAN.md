# WSR CLI skeleton implementation plan

Status: implemented in the local checkout on 2026-10-08 following user approval. This document
records the scope and implementation plan for the completed skeleton slice. It supplements
[the CLI guide](README.md) and does not replace the product decisions in [PLAN.md](../../PLAN.md)
or settle the final product interface. See the verification record below.

## Goal and scope

Make `wsr` a coherent, discoverable CLI skeleton. Argument parsing, help, version reporting,
dispatch, diagnostic output, and exit codes will work. Every operational command will stop
with a “not implemented” diagnostic before accessing a repository or performing an operation.

Keep the existing command surface as the starting point. This is an incremental improvement to
the two existing crates, not a fresh scaffold or a commitment to the final product interface.
No workflow parsing, execution, provider loading, permission enforcement, hooks, daemon,
cache operations, configuration loading, or GitHub Actions adapter belongs in this slice.
It can be completed without choosing a Linux backend or resolving the open execution design.

## Reference: astral-sh/uv

Reference snapshot: uv commit `5411378eb76dc1ea1ad90aeb10e84e997e5bbd96`, inspected on
2026-10-08. Use these patterns at WSR's current scale:

| uv source | Pattern to apply to WSR |
| --- | --- |
| [uv-cli argument definitions](https://github.com/astral-sh/uv/blob/5411378eb76dc1ea1ad90aeb10e84e997e5bbd96/crates/uv-cli/src/lib.rs) | Separate Clap command definitions, typed arguments, and nested command namespaces from application operations. |
| [uv executable](https://github.com/astral-sh/uv/blob/5411378eb76dc1ea1ad90aeb10e84e997e5bbd96/crates/uv/src/bin/uv.rs) | A thin executable forwards process arguments into the application crate. |
| [uv application entrypoint](https://github.com/astral-sh/uv/blob/5411378eb76dc1ea1ad90aeb10e84e997e5bbd96/crates/uv/src/lib.rs) | The application owns parsing, dispatch, diagnostics, and process status. |
| [uv help tests](https://github.com/astral-sh/uv/blob/5411378eb76dc1ea1ad90aeb10e84e997e5bbd96/crates/uv/tests/it/help.rs) | Verify help and argument errors as observable output and exit-status contracts. |

WSR does not need uv's async runtime, thread setup, process environment mutations, configuration
resolution, global option collection, custom help system, or separate command-support crate.
Use a safe synchronous entrypoint and Clap's built-in help/version handling. Preserve WSR's
existing public type names rather than renaming them to match uv.

## Starting point before this slice

`wsr-cli` already defined `Cli`, `Cmd`, `RunArgs`, `DaemonArgs`, `CacheCmd`, and `HookCmd`.
`wsr` already contained a thin binary, exhaustive top-level dispatch, and eight handler modules.
All handlers returned an `anyhow` error saying they were not yet implemented.

The gaps were small but observable: command descriptions implied working capabilities, workflow
paths and output formats were untyped strings, nested stubs identified only their parent command,
and there were no CLI contract tests in these crates. Build on these files directly; do not run
`cargo xtask scaffold cli` over the existing implementation or restore the preserved prototype.

## Command surface for this slice

Retain these eleven operational leaf paths and their existing options:

| Invocation | Parsed input |
| --- | --- |
| `wsr init` | No arguments. |
| `wsr run [FILE]` | Optional workflow path; `--event EVENT`, `--dry-run`, `--verbose`, `--yes`, `--format human\|gha`. |
| `wsr daemon` | Optional `--install`. |
| `wsr list` | No arguments. |
| `wsr inspect FILE` | Required workflow path. |
| `wsr cache list` | No arguments. |
| `wsr cache verify` | No arguments. |
| `wsr cache purge` | No arguments. |
| `wsr hook install HOOK` | Required hook name. |
| `wsr hook remove HOOK` | Required hook name. |
| `wsr status` | No arguments. |

Use `PathBuf` for workflow paths, with Clap file-path hints; never resolve or open them in a stub.
Use a Clap `ValueEnum` for `human` and `gha`, retaining `human` as the default. Use `Args` for
argument groups such as `RunArgs` and `DaemonArgs`. Keep event and hook names as strings until
their domain contracts are settled. Keep `--verbose` and `--yes` scoped to `run`.

Options are syntax only: `--dry-run` does not produce a plan, `--format gha` does not emit
annotations, and `--yes` does not authorize execution or grant access. Every valid combination
still reaches the same placeholder contract. Do not add `plan`, `actions`, `--allow '*'`,
backend selectors, or other proposed interfaces in this slice.

## Crate responsibilities

### `wsr-cli`: argument schema

- Keep root metadata and the command enum in `src/lib.rs`.
- Group argument definitions into small modules for `run`, `daemon`, `cache`, and `hook` where
  this improves readability; keep simple unit commands and the inline `inspect` path at the root.
- Re-export existing public types from the root. Add the output-format enum alongside `RunArgs`.
- Keep this crate independent of WSR execution, configuration, and repository discovery.
- Make the root description explicit: a pre-alpha local CI CLI scaffold. Mark operational
  command help as unimplemented and avoid claims about default workflow selection, installed
  services, or enforced sandboxing. Describe option intent without implying working behavior.

### `wsr`: application boundary and placeholder dispatch

- Add a safe synchronous `wsr::main` that accepts OS-string arguments and returns
  `std::process::ExitCode`. Keep `src/bin/wsr.rs` limited to forwarding `std::env::args_os()`.
- Parse with `Cli::try_parse_from`. Print Clap errors/help using Clap's facilities and preserve
  their exit codes. Do not intercept help/version with runtime initialization.
- Keep the existing exhaustive dispatch in `commands/mod.rs` and the eight handler modules.
  Match nested cache/hook variants so diagnostics identify the complete leaf path.
- Keep the existing `anyhow::Result<()>` handler boundary. One small helper can construct the
  common placeholder error; the application entrypoint prints it once and returns code 1.
  Do not create an error hierarchy, output framework, or backend abstraction for these stubs.

The executable is the user-facing contract. The application glue is an internal entrypoint,
not a new stable embedding API. A future execution engine should remain separate from both
the Clap schema and process startup code.

## Output and exit-code contract

| Input | stdout | stderr | Exit code |
| --- | --- | --- | --- |
| Root or nested `--help` | Clap help | Empty | 0 |
| `wsr --version` | Workspace-derived version | Empty | 0 |
| Any valid operational leaf | Empty | One placeholder diagnostic | 1 |
| Invalid arguments, unknown command, missing required input | Empty | Clap diagnostic/usage | 2 |
| Bare `wsr`, `wsr cache`, or `wsr hook` | Empty | Clap usage/help for a required subcommand | 2 |

Example: `wsr cache verify` writes exactly `error: wsr cache verify: not implemented yet`
followed by a newline to stderr and exits with code 1. Paths, events, and other user-supplied
values are not echoed in this diagnostic. The same format applies to all eleven leaf paths.

A placeholder must not report success: exit code 1 prevents automation from treating an
unimplemented command as completed CI work. Help and version remain fully usable.

All invocations in this slice must leave repository files, hooks, caches, and services untouched.
They must not load configuration, download providers, inspect credentials, contact the network,
or launch subprocesses. Missing or unreadable workflow files still produce the placeholder
after valid argument parsing; filesystem validation belongs to later command implementation.

## Implementation sequence and verification

1. **Define the parser contract.** Tighten path and format types, use appropriate Clap derives,
   organize argument definitions, and correct help text. Add parser tests for every command
   family, required inputs, both accepted format values, the default `human` format when omitted,
   and rejection of an invalid format.
   Verify Clap's command graph with `CommandFactory::command().debug_assert()`.
2. **Implement process startup and placeholders.** Add the safe application entrypoint,
   preserve exhaustive dispatch, and provide the shared diagnostic format. Add executable
   tests covering all eleven leaf paths, including options such as `run --dry-run --yes
   --format gha`, `daemon --install`, and `cache purge`. Verify exit code, stdout, and stderr.
3. **Verify discoverability and absence of operations.** Test root, command, and nested leaf
   help; root version; no-argument usage; unknown commands/flags; and omitted required file/hook
   arguments. Separately test `run` and `inspect` with a supplied nonexistent workflow path:
   each must still return its exact placeholder with code 1, rather than a filesystem error.
   Run the binary from a temporary directory containing a workflow, hook, and cache sentinel;
   compare its complete directory contents before and after all operational invocations.
   Inspect the code to confirm every handler terminates at the placeholder and startup performs
   no discovery or initialization. Filesystem tests alone do not prove absence of network I/O.
4. **Align dependencies and documentation.** Keep only dependencies actually used by these
   two crates and their tests. `wsr-cli` only needs Clap; the `wsr` application source
   uses Clap, `wsr-cli`, and `anyhow`. Remove unused direct subsystem dependencies after
   confirming this remains true. Preserve the other workspace crates and their root dependency
   entries. Update `Cargo.lock` through Cargo, the CLI guide, and crate docs to describe the
   implemented skeleton accurately. Record verification evidence without marking runtime gates
   complete. Finish with `cargo xtask ci`.

Put parser tests in `crates/wsr-cli/tests/cli.rs` and executable tests in
`crates/wsr/tests/cli.rs`. Use `std::process::Command` with `CARGO_BIN_EXE_wsr`; add a temporary
directory dev dependency only if needed for reliable fixture cleanup. Assert selected help
content, argument spelling, and output/status contracts rather than importing uv's test harness
or snapshot infrastructure. Check the version against package metadata rather than hardcoding
`0.0.3`.

For iteration, use targeted Cargo tests when useful, or `cargo xtask test` for the workspace.
The final `cargo xtask ci` includes checks, tests, and warnings-denied documentation; do not
repeat earlier wrappers immediately before it. Verify the product packages on the declared
MSRV with `cargo +1.85.0 check --locked --all-targets --all-features`, which checks the default
product members without forcing the newer private xtask tooling onto the product MSRV.
There are no new crates to scaffold and no release/publish commands in this plan.

## Verification record — 2026-10-08

- Implemented the typed schema, safe process entrypoint, and shared placeholder contract.
  `wsr-cli` depends only on Clap; `wsr` depends on Clap, `wsr-cli`, and `anyhow`, with
  `tempfile` as a test-only dependency for automatic fixture cleanup.
- `cargo xtask ci` passed formatting, locked workspace compilation, Clippy with warnings denied,
  all workspace tests, and warnings-denied Rust documentation. The two CLI crates now have six
  parser tests and five executable tests; other placeholder crates still have no behavioral tests.
- `cargo +1.85.0 check --locked --all-targets --all-features` passed for the default product members.
- A separate smoke run executed all eleven operational leaf paths using `target/debug/wsr`
  from an empty temporary directory. All produced the expected leaf-specific diagnostic,
  empty stdout, and exit code 1; no files were created in that directory.
- Sentinel tests covered existing workflow, configuration, hook, and cache files, including
  invalid workflow/configuration content, and detected no directory or file-content changes.
  This is CLI boundary evidence, not a containment or network-isolation test.
- Verification was local. No hosted CI run, commit, publication, or release was performed.

## Completion criteria

- The documented command tree parses, help/version work, and every operational leaf reports
  its exact path as unimplemented with code 1.
- Invalid syntax and incomplete namespaces return code 2; valid paths are not read or validated.
- The CLI schema has no dependency on execution subsystems; the application contains only
  parsing, dispatch, placeholder diagnostics, and exit-status handling.
- Contract tests and the final xtask/MSRV checks pass; documentation remains explicit that
  workflow execution, isolation, and GitHub Actions compatibility are still unimplemented.
- Existing tooling changes, audit material, migration backups, and the preserved WSR stash
  remain intact. Completing this slice does not satisfy the operational v0.1.0 delivery gates.
