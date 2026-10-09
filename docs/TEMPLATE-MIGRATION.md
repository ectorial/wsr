# Template migration and repository tooling

WSR initially adopted a rendered snapshot of [carlosferreyra/rust-template at
74d68f0](https://github.com/carlosferreyra/rust-template/tree/74d68f0daa4ab41c9c02a92b30b21372a10e2faa),
adopted on October 7, 2026. The WSR baseline was
`468444aa7d49fc08cbda3c08a19604e2b1c8e8b7`.

On October 8, repository tooling was synchronized with [template commit
62488bc](https://github.com/carlosferreyra/rust-template/tree/62488bc7d5cf0a34501282305dedb7d32068e235).
This is a reviewed tooling merge; WSR's product and design documents remain its own.

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
WSR-specific verification/release behavior, and update this record. Action references live in
`tools/xtask/assets/tooling.toml`; optional tool versions are selected at sync time. The checked-in
reusable workflows are deliberately hand-maintained. Match their checkout/setup references to
the registry when updating them. The cargo-dist-generated release workflow is a separate owner.

The October 8 merge adopts workspace dependency inheritance for xtask and CLI scaffolding,
using concrete generation-resolved requirements: cargo_metadata **0.23.1**, Clap **4.6.7**,
and toml_edit **0.25.16**. Private xtask now inherits the shared workspace version. Sync updates
stable Rust with rustfmt/Clippy and installs current stable crates.io releases with each release's
locked dependencies. Discovery accepts working local or global tools without exact-version checks.
Full CI scaffolds request unversioned cargo-deny/typos-cli through the install action. The retired
template-maintainer CI asset was removed; generation hooks and their placeholders are not imported
into WSR.

WSR retains its product Rust **1.85** baseline instead of adopting the freshly generated template's
**1.99.0** baseline. It also retains locked all-feature checks, warnings-denied documentation,
publication exclusions, preparation's `--no-push`, and the existing CI structure. Cargo-dist is a
documented exception to unversioned sync: an existing `dist-workspace.toml` selects the matching
upstream Git tag, currently **0.33.0**, preventing a downgrade to an older crates.io release.
Sync does not rewrite workspace dependency requirements, the product MSRV, or release CI.

October 8 verification passed: fresh template generation, WSR's standard `cargo xtask ci`,
product checks on Rust **1.85.0**, real release planning, and disposable-fixture scaffold checks
for a private crate, primary CLI, and full CI. The template's tool regression harness verified
two sync upgrades and local/global discovery; WSR-specific probes verified the distribution pin,
no-push preparation, private publication exclusions, and guarded changelog behavior without uploads.
Hashes confirmed all 80 protected product/document/history files were unchanged, and the preserved
stash list was identical. Hosted CI has not run for the synchronization edits.

## CI and releases

The existing CI entrypoint, reusable workflows, check names, manual dispatch, credential handling,
and release workflow are retained. Lint/tests invoke xtask. Checkout and Rust setup use the imported
registry references. Local and hosted checks preserve locked dependencies, all-feature linting,
and warnings-denied documentation. No scheduled audit was added during the initial migration.

The October 9 follow-up adds weekly Cargo/GitHub Actions Dependabot updates and individual
`cargo xtask ci --job format|lint|docs|test` operations. Hosted jobs reuse these operations without
repeating formatting or linting; aggregate local commands keep their existing checks. Private
xtask explicitly requires Rust 1.86 while product crates retain Rust 1.85. Dist discovery now rejects
executables that differ from the configured 0.33.0 version, retaining the Git-tag installer.
The release changelog hook skips both generation and staging during dry runs. The release guide
pushes the distribution tag separately instead of batching workspace tags with `--follow-tags`.
The generated release workflow remains unchanged; actionlint records five narrow cargo-dist 0.33.0
ShellCheck exceptions, to be reviewed on generator upgrades. `scripts/test-tooling.py` exercises
CI job isolation, configured dist selection/installation, and dry-run/execute hook behavior in
temporary fixtures without installs or uploads.

Optional tools are installed only on request:

```sh
cargo xtask tools sync test
cargo xtask tools sync coverage
cargo xtask tools sync release
```

Executables install under ignored `.xtask/tools`; coverage setup also installs Rustup's
`llvm-tools-preview` component. Do not treat these development helpers as workload isolation.

The migration initially retained cargo-dist **0.30.3**. The October 7 tooling refresh updated
the registry and `dist-workspace.toml` to **0.33.0** and regenerates the release workflow with
that version. Cargo-dist was installed from its matching upstream Git tag with `--locked`;
other tools initially used exact crates.io versions, superseded by the October 8 sync behavior
above. The refresh also updated typos to **1.51.1**, install-action to **2.87.26**,
and the reusable workflows' rust-cache pin to **2.9.2**, and added Rustup self-updates to those
workflows. Local Rustup **1.29.1** and stable Rust **1.99.0** were already current.
The release workflow uses supported `github-action-commits` overrides for checkout **7.0.1**,
upload-artifact **7.0.2**, download-artifact **8.0.2**, and rust-cache **2.9.2**, matching the registry.
Regenerate it with `.xtask/tools/bin/dist generate --mode ci` after updating these settings;
verify it with the same command plus `--check`.
Existing `release.toml`, `cliff.toml`, CHANGELOG, and target list are preserved. `release init`
remains a guarded generator and refuses the existing user-owned release configuration;
this repository is already initialized.

The refresh was verified locally with `cargo xtask tools sync all`, `cargo xtask doctor`,
`cargo xtask ci`, `cargo xtask release plan`, and the release workflow generation check.
All seven optional tools then resolved to their pinned project-local installations; cargo-generate
**0.25.0** was already available globally. Hosted CI was not run for those local edits.

```sh
cargo xtask release prepare
cargo xtask release plan
```

These require the available tools. Preparation defaults to cargo-release's dry run. With explicit
`--execute`, the imported wrapper still passes both `--no-publish` and WSR's added `--no-push`;
it may change versions, changelog, commits, and tags locally. Use a clean reviewable branch and
handle publication separately. No release command was executed during the migration.

After migration, WSR added a separate publisher:

```sh
cargo xtask release publish
cargo xtask release publish --execute
```

It delegates to cargo-release's workspace publish phase, defaults to a dry run, and skips
versions already on the registry. Registry errors, including retry times, are returned without
automatic waiting. Rerun the publisher to resume; preparation would bump versions again.
Publication does not push commits or tags. See [the release procedure](../CONTRIBUTING.md#publish-a-workspace-release).

The CLI and its fourteen supporting crates were published at version 0.0.3, and a fresh
registry installation was verified. Internal dependencies retain versioned declarations.
`wsr-bench` is an internal benchmark placeholder and is now private, alongside `xtask`;
its workspace version can still advance without an upload. Published scaffolds remain
scaffolds: registry availability does not establish runtime or security support.

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
