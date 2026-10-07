# CLI

## Current scaffold

[Argument definitions](../../crates/wsr-cli/src/lib.rs) use clap;
[dispatch](../../crates/wsr/src/commands/mod.rs) selects eight handlers: `init`, `run`, `daemon`,
`list`, `inspect`, `cache`, `hook`, and `status`. Every handler returns “not yet implemented.”

`run` defines an optional file and `--event`, `--dry-run`, `--verbose`, `--yes`, and `--format`.
These are parsed arguments, not implemented execution/planning/reporting behavior. Help text can
be inspected with `cargo run -p wsr -- --help`.

## Accepted design and open interface

Local inspection, execution, and debugging come first; GitHub Actions engine reuse follows.
Final command structure, selection, event binding, and output formats are not settled.
A planning-only path must not execute repository commands. Effective permissions and unsupported
requirements must be visible before execution.

`plan` exists in preserved prototype work, not the current command enum. `--allow '*'` is proposed
syntax for an explicit broad-access profile, not an implemented option. There is no `actions`
subcommand. Hooks/daemon behavior is not the primary product commitment.

See [PLAN.md](../../PLAN.md) and [the security model](../SECURITY-MODEL.md).
