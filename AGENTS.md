# Developing WSR

Use `cargo xtask` for repository development, scaffolding, and releases. These instructions
supplement parent agent instructions. Keep changes focused on the requested behavior and preserve
existing local work.

## Read the project before changing it

- [README.md](README.md) describes current support. This is a pre-alpha CLI/types scaffold;
  workflow execution and security enforcement are not implemented. Compilation and publication
  do not prove those capabilities work.
- [PLAN.md](PLAN.md) owns accepted decisions, proposals, and open questions.
  [ARCHITECTURE.md](ARCHITECTURE.md) describes current and target boundaries.
- [ROADMAP.md](ROADMAP.md) owns delivery gates; [CHECKLIST.md](CHECKLIST.md) records implementation
  evidence. Update completion claims only when working behavior and verification justify them.
- [crates/README.md](crates/README.md) inventories existing crates; placeholder names do not
  prescribe the future design. [docs/README.md](docs/README.md) indexes subsystem documentation.
- [docs/TEMPLATE-MIGRATION.md](docs/TEMPLATE-MIGRATION.md) records the imported tooling snapshot,
  WSR-specific changes, and recovery rules. Template updates require a reviewed merge;
  there is no automatic synchronization.

Preserve the unpublished prototype stash and migration backups. Recover selected work in an
isolated checkout; do not apply old manifests, tooling, or documentation over the migrated tree.
When changing the product story, reconcile WSR documentation and the organization summaries in
`ectorial/.github`. Registry descriptions must distinguish working features from planned ones.

## Workspace and entrypoints

Run commands from this repository or a workspace subdirectory. `.cargo/config.toml` defines
`xtask = "run --package xtask --"`. The executable discovers the workspace through Cargo metadata
and runs its operations from the workspace root. It is a development helper, not the WSR runner.

- `crates/wsr` owns the `wsr` executable and command dispatch.
- `crates/wsr-cli` owns Clap argument definitions; it is not the installable binary package.
- Product packages live under `crates/` and inherit `[workspace.package]` version, edition, and MSRV.
- Private xtask also inherits the workspace version. Its Cargo metadata, Clap, and TOML editing
  dependencies inherit concrete requirements from `[workspace.dependencies]`.
- The development toolchain tracks stable with rustfmt and Clippy. Product MSRV is declared in
  the root manifest; currently Rust 1.85 with edition 2024. Private xtask requires Rust 1.86.
- Default Cargo members are `crates/*`; xtask's explicit `--workspace` checks also include
  private `tools/xtask`. `wsr-bench` is also private and remains in workspace checks.
- Publishable internal dependencies need both workspace paths and registry version requirements.
  Keep them, the shared product version, and `Cargo.lock` aligned. Do not advance one published
  crate independently of the synchronized workspace release.

Inspect the product CLI with `cargo run -p wsr -- --help`. This is separate from xtask help.
Use `cargo xtask --help`, `cargo xtask help <command>`, or a command's `--help` to inspect options.
`--version` reports xtask's version, which now inherits the shared workspace version.

## Development command reference

| Command | Actual behavior |
| --- | --- |
| `cargo xtask check` | `cargo fmt --all --check`, then locked compilation and Clippy for the workspace, all targets and all features. Clippy receives `-D warnings`. |
| `cargo xtask test [FILTER]` | Runs `check`, then `cargo test --workspace --locked`, forwarding the optional test-name filter. |
| `cargo xtask test [FILTER] --nextest` | Runs `check`, then available cargo-nextest with `run --workspace --locked` and the optional filter; separately runs all workspace doctests with Cargo. The filter is not applied to that doctest step. |
| `cargo xtask build` | Runs the unfiltered standard `test` sequence, then `cargo build --workspace --release --locked`. |
| `cargo xtask ci` | Runs the unfiltered standard `test` sequence, then `cargo doc --workspace --no-deps --locked`. Appends `-D warnings` to existing `RUSTDOCFLAGS`. |
| `cargo xtask ci --full` | Runs `ci`, then available `cargo deny check` and `typos .`. Rustdoc already runs in ordinary `ci`. |
| `cargo xtask ci --job format\|lint\|docs\|test` | Runs only that hosted CI job: formatting; locked all-target/all-feature compilation and Clippy; strict locked docs; or locked tests. Cannot combine with `--full`. |
| `cargo xtask coverage` | Delegates to available `cargo llvm-cov --workspace --html`. It does not first run `check`, and the wrapper does not add `--locked`. |
| `cargo xtask doctor` | Reports built-in rustfmt/Clippy availability and optional tool availability/locations. It accepts working executables, except dist must match `dist-workspace.toml`. It does not install tools or fail merely because an optional tool is missing. |
| `cargo xtask tools sync [GROUP]` | Updates stable Rust, adds its rustfmt/Clippy components, and installs current stable crates.io releases under `.xtask/tools`; WSR's configured cargo-dist version is the exception. The default group is `all`. |
| `cargo xtask scaffold <COMMAND> [OPTIONS]` | Plans and, unless `--dry-run` is supplied, writes the requested scaffold after validation. See below. |
| `cargo xtask release <COMMAND> [OPTIONS]` | Runs the selected release operation. See below; preparation, publishing, and Git pushing are distinct steps. |

For routine Rust edits, choose the smallest sequence that covers the change: `check` for lint/build
feedback, `test` for behavior, and `ci` for complete standard verification before committing.
These commands include earlier checks; avoid running `check`, `test`, and `build` consecutively
when one `build` suffices. Use meaningful tests for changed behavior. Passing an empty test suite
is build verification, not product validation.

`check` checks formatting without changing it. Use `cargo fmt --all` to apply required formatting;
there is no xtask formatting command. Direct Cargo commands remain appropriate for targeted package
tests, product CLI inspection, or MSRV checks that xtask does not expose. The wrappers do not accept
arbitrary trailing Cargo arguments. Report what passed, what failed, and what was not exercised.

## Optional tools

Action references are bundled in [tools/xtask/assets/tooling.toml](tools/xtask/assets/tooling.toml);
it no longer pins optional tool versions. A working project-local executable is preferred;
a working executable on `PATH` is the fallback. Commands that need optional tools return
installation guidance when they are missing. They do not install tools automatically.

| `tools sync` group | Installed tools |
| --- | --- |
| `ci` | cargo-deny and typos-cli |
| `coverage` | cargo-llvm-cov; also adds Rustup's `llvm-tools-preview` component |
| `release` | cargo-dist, cargo-release, and git-cliff |
| `test` | cargo-nextest |
| `all` (default) | All seven optional tools above |

Install only the group needed for the task. Each sync updates shared stable Rust and uses
`cargo +stable install --locked` to select current stable crates.io releases. `--locked` retains
each tool release's dependency lockfile; it does not pin the tool version. Existing working
versions remain accepted between syncs, except cargo-dist must match the distribution configuration.
Sync does not change workspace dependency requirements or the product MSRV. Cargo-dist uses the
upstream Git tag matching `dist.cargo-dist-version` in
[dist-workspace.toml](dist-workspace.toml), preserving local/generated release compatibility.
Without that file, cargo-dist also resolves through crates.io. `.xtask/tools` is ignored
development state. cargo-generate is a separate generation tool, not installed by any group.
`ci --full` also needs usable policy and typo
configuration; installing its tools does not scaffold configuration. Preserve existing CI when
merging any additional checks.

## Scaffold command reference

`--dry-run` is global within `scaffold` and can appear before or after its child command.
Start with a dry-run for requested additions. Scaffold plans validate file ownership and parent
paths before writing; dry-runs perform the same checks and may refuse a conflicting existing file.

| Command | Behavior and options |
| --- | --- |
| `cargo xtask scaffold crate <semantic-name>` | Creates `crates/wsr-<semantic-name>` as a library, inheriting workspace package settings and lints, and adds a root workspace dependency. |
| `cargo xtask scaffold crate <semantic-name> --bin` | Creates a binary with `src/main.rs`; does not add a root workspace dependency. `crates/*` makes it a workspace member. |
| `cargo xtask scaffold crate <semantic-name> --private` | Sets `publish = false`. Can combine with `--bin`; a private library's root dependency has a path but no registry version. |
| `cargo xtask scaffold cli --entrypoint companion` | Default CLI mode: creates private `wsr-cli` with the parser and a `wsr` binary depending on the primary library. |
| `cargo xtask scaffold cli --entrypoint primary` | Creates publishable `wsr-cli` argument types and a thin binary in the primary `wsr` crate; adjusts dependencies and the binary harness settings. |
| `cargo xtask scaffold ci --preset lean` | Default CI preset: renders a managed `.github/workflows/ci.yml` using bundled action references. |
| `cargo xtask scaffold ci --preset full` | Also renders managed `deny.toml`, `typos.toml`, an audit workflow, and Dependabot configuration. Its install action selects cargo-deny and typos-cli without tool version pins. |
| `cargo xtask scaffold docs` | Renders managed `CONTRIBUTING.md` and `crates/README.md`. |
| `cargo xtask scaffold agents [--claude]` | Renders a managed starter `AGENTS.md`; `--claude` also creates a `CLAUDE.md` reference to it. |

Semantic names start with a lowercase ASCII letter and use lowercase letters, digits, or single
hyphens; trailing and doubled hyphens are rejected. `cli` and `xtask` are reserved. Start new
behavior as a module and extract a crate only when it has a justified ownership boundary.
After applying a new crate scaffold, update `Cargo.lock` through Cargo, verify with xtask, and
maintain the crate inventory. A scaffold does not implement a subsystem.

WSR already has its CLI, workflows, and documentation. Do not rerun these generators over existing
content as a migration/update shortcut. Managed files require their generated marker to be
overwritten; created files must be absent or exactly match the scaffold. The planners also reject
symlink/nonregular destinations and conflicting dependencies. Do not add markers to bypass guards.
This `AGENTS.md` is deliberately user-owned, without a generated marker; edit it directly rather
than replacing it with `scaffold agents`.

## Release command reference

Use the wrapper for synchronized releases. Execute publication or Git pushing only within the
user's authorized scope. Keep private development packages private and preserve existing releases.

| Command | Actual behavior |
| --- | --- |
| `cargo xtask release init` | Generates guarded release configuration, adds a primary-package changelog hook, and invokes available `dist init --yes` when a publishable binary exists. Without one, only the configuration is generated. Mutates files; no dry-run option. WSR is already initialized and its existing unmarked configuration is protected. |
| `cargo xtask release plan [--tag TAG]` | Delegates to available `dist plan`, optionally with `--tag=TAG`. Requires a publishable binary and a dist version compatible with the distribution configuration; plans distribution artifacts, not crates.io uploads. |
| `cargo xtask release prepare [LEVEL]` | Delegates to available `cargo release LEVEL --workspace --no-publish --no-push`. Levels are `patch` (default), `minor`, and `major`. Requires cargo-release and git-cliff. |
| `cargo xtask release prepare [LEVEL] --execute` | Applies the workspace version/dependency updates, hooks/changelogs, release commit, and package tags after cargo-release's confirmation. Does not publish or push. Private packages can still receive version bumps and tags. |
| `cargo xtask release publish` | Registry dry-run via `cargo release publish --workspace --exclude xtask --exclude wsr-bench`. Requires cargo-release; checks unpublished versions in dependency order and skips versions already published. |
| `cargo xtask release publish --execute` | Uploads the selected unpublished versions after cargo-release's confirmation. Neither bumps versions nor pushes Git refs. |
| `cargo xtask release changelog --tag TAG` | Hidden hook helper: generates root `CHANGELOG.md` with git-cliff; skips generation when `DRY_RUN=true`. It does not stage the file. |

Preparation defaults to cargo-release's dry-run. The root `release.toml` hook runs changelog
generation and staging only when `DRY_RUN=false`; previews preserve both file contents and the
index. Use a clean isolated checkout for previewing preparation and inspect status afterward.

For an authorized release from a clean checkout:

```sh
cargo xtask ci
cargo xtask release prepare patch --execute
cargo xtask release publish
cargo xtask release publish --execute
git push origin main
# Replace VERSION with the prepared product version; push the binary tag separately.
git push origin wsr-vVERSION
```

Push other intended package tags individually after reviewing them. GitHub suppresses tag push
events for batches larger than three tags, so do not rely on `--follow-tags` to trigger binary
distribution. Existing tags must not be deleted/recreated to retrigger CI.

Inspect the generated commit and tags before uploading. The wrapper has no `--no-confirm` option;
execute mode retains cargo-release's interactive prompt. No release subcommand wraps Git pushing.
The publisher propagates registry errors and retry times without automatic sleeping. For a partial
upload, retry `release publish --execute` after the stated time; do not rerun `prepare`, which would
bump versions again. When all selected versions are published, cargo-release reports
`no packages selected` and exits nonzero. Confirm the preceding skip messages before treating
that as completion; other errors still require diagnosis. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Keep tooling and this guide aligned

The CLI contract is [tools/xtask/src/cli.rs](tools/xtask/src/cli.rs); behavior lives in `tasks.rs`,
`tools.rs`, `release.rs`, and `scaffold/`. When changing these commands, update this guide and the
affected contribution/migration documentation. Preserve the WSR overrides, especially release
publication exclusions and preparation's no-push behavior. Keep the installed cargo-dist version
aligned with `dist-workspace.toml` and its separately generated release workflow. The template's
generation hooks resolve versions for new projects; do not import them or unresolved placeholders
into this existing workspace. Preserve WSR's concrete requirements and documented MSRV.

Validate actual help and behavior rather than trusting inherited template descriptions. Test
mutating generators in temporary fixtures, preserve user-owned files, and distinguish local
verification from hosted CI results. Template development tooling does not enforce workload isolation.
