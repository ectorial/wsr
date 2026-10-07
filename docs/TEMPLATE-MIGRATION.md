# Template migration and repository tooling

WSR uses a rendered snapshot of [carlosferreyra/rust-template at
74d68f0](https://github.com/carlosferreyra/rust-template/tree/74d68f0daa4ab41c9c02a92b30b21372a10e2faa),
adopted on October 7, 2026. The WSR baseline was
`468444aa7d49fc08cbda3c08a19604e2b1c8e8b7`.

The migration retains sixteen product crates and their existing Rust sources, version 0.0.2,
license, release history, and product documentation. It adds private `tools/xtask` automation,
the Cargo alias, a stable Rust toolchain with rustfmt/Clippy, resolver 3, and product Rust 1.85
declarations. Product default members exclude xtask so ordinary Cargo commands retain their
original scope. No workflow engine, provider, or sandbox is implemented by this migration.

## Develop

```sh
cargo xtask check
cargo xtask test
cargo xtask build
cargo xtask ci
cargo xtask doctor
```

`check` runs formatting, locked all-feature/all-target compilation and warnings-denied Clippy.
`test` also runs locked workspace tests; `build` then builds the release workspace. `ci` adds
locked documentation with Rustdoc warnings denied while retaining caller-supplied Rustdoc flags.
The current product scaffold has no behavioral tests; passing these checks is build verification.

Tooling inherits the template's workspace lint policy. Product crates do not yet inherit that
policy: the assessment identified 75 additional warnings, principally missing API documentation.
Enabling it is a separate focused change. Product sources and CLI help remain the current scaffold.

## Grow and update deliberately

Start behavior as a module, then extract a project-prefixed crate when it owns a coherent
responsibility and a useful testable seam. Existing placeholder crates do not dictate future
architecture. [PLAN.md](../PLAN.md) still owns those decisions.

```sh
cargo xtask scaffold crate <semantic-name> --dry-run
```

New scaffolds support planning and ownership checks. Existing CLI sources, CI, contribution
guides, and crate inventory remain WSR-owned; their scaffolds intentionally refuse conflicting
content. Do not add generated markers just to bypass that protection or replace these documents
with starter text. The template's README, PLAN, CONTEXT, and maintainer CI are not WSR's design.

Generated projects are snapshots. There is no automatic template synchronization. For updates,
review changes against the pinned revision, merge selected tooling/assets in a branch, retain
WSR-specific verification/release behavior, and update this record. Tool/action versions live in
`tools/xtask/assets/tooling.toml`; the checked-in reusable workflows are deliberately hand-maintained,
not output of the template-maintainer workflow renderer. Match their checkout/setup references to
the registry when updating them. The cargo-dist-generated release workflow is a separate owner.

## CI and releases

The existing CI entrypoint, reusable workflows, check names, manual dispatch, credential handling,
and release workflow are retained. Lint/tests invoke xtask. Checkout and Rust setup use the imported
registry references. Local and hosted checks preserve locked dependencies, all-feature linting,
and warnings-denied documentation. No scheduled audit or automatic dependency updater is added.

Optional tools are installed only on request:

```sh
cargo xtask tools sync test
cargo xtask tools sync coverage
cargo xtask tools sync release
```

Executables install under ignored `.xtask/tools`; coverage setup also installs Rustup's
`llvm-tools-preview` component. Do not treat these development helpers as workload isolation.

The registry intentionally retains cargo-dist **0.30.3**, matching `dist-workspace.toml` and the
generated release workflow, instead of the template's 0.33.0. Existing `release.toml`, `cliff.toml`,
CHANGELOG, and target list are preserved. `release init` remains a guarded generator and refuses
the existing user-owned release configuration; this repository is already initialized.

```sh
cargo xtask release prepare
cargo xtask release plan
```

These require the pinned tools. Preparation defaults to cargo-release's dry run. With explicit
`--execute`, the imported wrapper still passes both `--no-publish` and WSR's added `--no-push`;
it may change versions, changelog, commits, and tags locally. Use a clean reviewable branch and
handle publication separately. No release command was executed during the migration.

Existing product publishability is retained, with versioned internal dependency declarations.
This does not approve publishing every placeholder crate or establish a tested crates.io release.

## Preservation and recovery

Before migration, all original Markdown documents and LICENSE were archived and checked by SHA-256.
A committed-tree archive and verified Git bundle preserve the baseline history and original stash.
Recovery locations and verification results are recorded outside the repository in the migration
backup's `PROGRESS.md`; backups are not committed as product files.

The original stash remains `daee0885832df55a9a872037d3ce6741b9e6e72a`. Its prototype code and
older `crates/xtask` tooling were not applied. Restore only selected product work in an isolated
recovery checkout after reconciling it with [the architecture](../ARCHITECTURE.md). Keep old root
manifests, automation, and documents from overwriting the template migration or current decisions.

WSR keeps technical decisions/status in its existing documents; the organization's `.github`
repository continues to summarize that direction. Template adoption changes development tooling,
not the accepted runtime design or its implementation status.
