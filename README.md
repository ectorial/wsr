# wsr — workflow sandboxed runner

WSR is a pre-alpha project for running and debugging CI locally, then reusing the same
execution engine inside GitHub Actions. The WebAssembly Component Model is central to the
planned deny-by-default security model; existing tools use a separately isolated system backend.

## Current status

The checked-out implementation is a Rust workspace and CLI scaffold. Argument parsing and
shared types exist, but all eight command handlers return “not yet implemented.” This checkout
does not parse or execute workflows, enforce permissions, install hooks, or run components.

Substantial unpublished prototype code is preserved in a local July 29 stash. It includes
inspection/planning, source snapshots, a development command backend, and GitHub adapter work.
That prototype is not in this checkout or a supported release. Its tests have not been run as
part of the current audit or documentation reconciliation.

## Accepted product direction — planned

- Run and debug a documented subset of GitHub Actions workflows locally first.
- Reuse the engine inside GitHub Actions next; independent CI is a later possibility.
- Compile provider input into a provider-neutral workflow model with source traceability.
- Use signed Component Model provider plugins and explicit capability-based component execution.
- Run ordinary commands through an isolated Linux system backend; reject execution when required
  restrictions cannot be enforced.
- Support Linux jobs from macOS and Linux development machines first. Native macOS and Windows
  jobs come later; CPU architecture support remains open.
- Assume workloads may be malicious. Workflow requests do not authorize their own permissions.
- Offer explicitly owner-authorized broad runner access as an exception to the default policy.
  Wildcard scope and syntax remain open; broader permissions do not establish full compatibility.

These are design decisions, not available features. Concrete runtime versions, system backends,
outer isolation, grant syntax, and the first tested compatibility subset remain unresolved.

## Development

There is no supported workflow-execution quick start yet. To inspect the scaffold's CLI:

```sh
cargo run -p wsr -- --help
cargo xtask check
cargo xtask test
cargo xtask ci
```

The workspace uses the [rust-template tooling snapshot](docs/TEMPLATE-MIGRATION.md), with sixteen
product crates and private `tools/xtask` automation. Product crates declare Rust 1.85 as their minimum
version and the development toolchain tracks stable. Future runtime/toolchain choices remain open. No installer or schema
URL is advertised as a working onboarding path.

## Documentation

| Document | Responsibility |
| --- | --- |
| [PLAN.md](PLAN.md) | Authoritative accepted decisions, proposals, and open questions |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Current scaffold and target component boundaries |
| [ROADMAP.md](ROADMAP.md) | Delivery order and completion gates, without release-date promises |
| [CHECKLIST.md](CHECKLIST.md) | Evidence-based implementation accounting |
| [Security model](docs/SECURITY-MODEL.md) | Planned security contract and its limits |
| [Documentation index](docs/README.md) | Subsystem guides and documentation maintenance rules |
| [Template migration](docs/TEMPLATE-MIGRATION.md) | Tooling provenance, local commands, and update/recovery rules |
| [Crate inventory](crates/README.md) | Existing workspace contents, not a required future layout |

The [organization repository](https://github.com/ectorial/.github) owns the profile and governance.
It summarizes this repository's direction rather than defining a separate technical roadmap.

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the
[Code of Conduct](CODE_OF_CONDUCT.md). Licensed under [MIT](LICENSE).
