# Contributing to wsr

Thank you for your interest in contributing!

## How to contribute

1. **Fork** the repository and clone your fork locally.
2. Create a branch for your change:
   ```bash
   git checkout -b feat/your-feature
   ```
3. Make your changes and commit following **Conventional Commits**:
   ```
   feat: add workflow parser
   fix: handle missing config file
   docs: update usage examples
   ```
4. Push to your fork and open a **Pull Request** against `main`.

## Commit conventions

This project uses [Conventional Commits](https://www.conventionalcommits.org). Your commit messages directly determine the changelog, so please be descriptive.

Common prefixes:
- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `refactor:` — code change that neither fixes a bug nor adds a feature
- `chore:` — tooling, dependencies, config
- `ci:` — CI/CD changes

## Development setup

```bash
git clone https://github.com/<your-fork>/wsr.git
cd wsr
cargo xtask check
cargo xtask test
cargo xtask build
```

## Preview the changelog before releasing

```bash
git cliff
```

## Questions?

Open an issue or reach out at eduferreyraok@gmail.com.

## Documentation and design changes

Use [PLAN.md](PLAN.md) for accepted decisions and open proposals, [CHECKLIST.md](CHECKLIST.md)
for implementation accounting, and [ROADMAP.md](ROADMAP.md) for delivery gates. The architecture
interview is not implementation-ready; documentation reconciliation does not resolve open choices.

Distinguish current scaffold, preserved prototype, accepted design, and open proposals. Do not mark
an execution feature complete because a type, flag, or documentation-only crate exists. When a change
affects the product story, update the relevant WSR guides and organization summaries together.
Keep the organization's profile/template identical and verify active relative links. Preserve
historical release notes and unpublished recovery material as history, not current support claims.

## Repository tooling

The private `tools/xtask` package is imported from rust-template. `cargo xtask ci` runs the
local equivalents of format, locked all-feature lint, tests, and warnings-denied documentation.
Optional tools are installed explicitly with `cargo xtask tools sync <group>` under `.xtask/tools`.
Each sync updates stable Rust, adds rustfmt/Clippy, and uses `cargo +stable install --locked` to
resolve current stable tool releases from crates.io. Working local tools take priority over
working tools on `PATH`; their versions are not enforced between syncs. Cargo-dist installs from
the upstream Git tag matching `dist-workspace.toml`, keeping release generation compatible.
Sync does not rewrite dependency requirements or the product MSRV. Refresh Rustup itself with
`rustup self update`.
Existing CI and documentation files are user-owned; merge updates rather than rerunning their
scaffolds over them. See [the migration record](docs/TEMPLATE-MIGRATION.md) for provenance,
release exceptions, and future update rules.

## Publish a workspace release

Run `cargo xtask ci` before preparing a release from a clean checkout:

```sh
cargo xtask release prepare patch --execute
cargo xtask release publish
cargo xtask release publish --execute
git push origin main --follow-tags
```

Preparation synchronizes product versions, generates changelogs, and creates a local commit
and package tags. It does not publish or push. `publish` defaults to a registry dry run;
`--execute` uploads unpublished versions in dependency order, with cargo-release's confirmation
prompt. The private `xtask` and `wsr-bench` packages are not uploaded.

If crates.io rejects an upload, the command exits with the registry error and retry time.
After that time, rerun `cargo xtask release publish --execute`: already-published versions
are skipped. Do not rerun `prepare` to resume an upload, since it would bump versions again.
When every selected version is already published, cargo-release reports `no packages selected`
and exits nonzero; after checking the preceding skipped-version messages, no upload remains.
The publisher does not push commits or tags; run the explicit Git command after publication.
