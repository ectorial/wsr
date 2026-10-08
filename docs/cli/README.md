# CLI

## Current scaffold

[Typed argument definitions](../../crates/wsr-cli/src/lib.rs) use clap;
[dispatch](../../crates/wsr/src/commands/mod.rs) selects eight handlers: `init`, `run`, `daemon`,
`list`, `inspect`, `cache`, `hook`, and `status`. These cover eleven operational leaf paths,
including `cache list/verify/purge` and `hook install/remove`.

Every operational leaf writes `error: wsr <command>: not implemented yet` followed by a newline
to stderr, leaves stdout empty, and exits with code 1. The command name includes the nested
leaf, such as `wsr cache verify`. No workflow file is opened, and no configuration, hook, cache,
service, provider, or execution backend is initialized. Supplied nonexistent paths reach the
same placeholder.

Help and root version reporting exit with code 0 and write to stdout. Invalid syntax, omitted
required arguments, and bare `wsr`, `wsr cache`, or `wsr hook` exit with code 2 and write Clap
diagnostics or usage/help to stderr.

`run` defines an optional file and `--event`, `--dry-run`, `--verbose`, `--yes`, and `--format`.
Workflow paths are `PathBuf` values, and `--format` accepts only `human` (default) or `gha`.
These are parsed arguments, not implemented execution/planning/reporting behavior. `--yes` does
not grant access. Help text can be inspected with `cargo run -p wsr -- --help`.

## Skeleton implementation and verification

[The CLI skeleton plan](SKELETON-PLAN.md) records the implemented slice, using uv's
argument/application separation as a reference. It retains the existing commands, adds typed
arguments and CLI contract tests, and gives each operational leaf consistent unimplemented
output. The final product interface remains open.

[Parser tests](../../crates/wsr-cli/tests/cli.rs) verify the command graph, options/defaults,
required inputs, format validation, and OS-string paths.
[Executable tests](../../crates/wsr/tests/cli.rs) run every leaf, check help/version and error
codes, exercise nonexistent paths, and compare repository sentinels before/after invocation.
The smoke contract runs with `cargo xtask test smoke_every_operational_subcommand`; complete
standard verification uses `cargo xtask ci`. These checks validate the skeleton, not workflow
execution, isolation, or GitHub Actions compatibility.

## Accepted design and open interface

Local inspection, execution, and debugging come first; GitHub Actions engine reuse follows.
Final command structure, selection, event binding, and output formats are not settled.
A planning-only path must not execute repository commands. Effective permissions and unsupported
requirements must be visible before execution.

`plan` exists in preserved prototype work, not the current command enum. `--allow '*'` is proposed
syntax for an explicit broad-access profile, not an implemented option. There is no `actions`
subcommand. Hooks/daemon behavior is not the primary product commitment.

See [PLAN.md](../../PLAN.md) and [the security model](../SECURITY-MODEL.md).
