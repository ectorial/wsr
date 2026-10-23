# Contributing to wsr

Thank you for your interest in contributing!

## Scope and choosing work

WSR is pre-alpha: operational CLI commands are placeholders. Start with the
[README](README.md), [roadmap](ROADMAP.md), and [validation cases](tests/README.md).
Bug fixes, tests, and documentation improvements are welcome. Open an issue before
starting a new execution feature or changing architecture so maintainers can confirm
scope and unresolved design choices. Never include credentials in issues or test fixtures;
report vulnerabilities through [SECURITY.md](SECURITY.md).

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
# Install stable Rust with rustup first. Python 3.11+ and Git must be on PATH.
# Install the components selected by rust-toolchain.toml.
rustup show
rustc --version
python3 --version
git --version
cargo xtask ci
```

`cargo xtask ci` checks formatting, locked workspace compilation and Clippy,
tests (including the offline corpus), and documentation. No optional Cargo tools,
credentials, or upstream source downloads are needed for this baseline. Use
`cargo fmt --all` to fix formatting. See [AGENTS.md](AGENTS.md) for focused commands.

Before opening a PR, run `cargo xtask ci` and `git diff --check`, explain the
behavior changed and validation results, and update affected documentation. Feature
work should follow the [TDD contract](docs/TESTING-PLAN.md#11-tdd-and-agent-implementation-contract).
Maintainers review scope, behavior, and CI before merging; a green build alone does
not prove workflow compatibility or security enforcement.

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
Optional tools are installed explicitly with `cargo xtask tools <group>` under `.xtask/tools`.
Each sync updates stable Rust, adds rustfmt/Clippy, and uses `cargo +stable install --locked` to
resolve current stable tool releases from crates.io. Working local tools take priority over
working tools on `PATH`; cargo-dist must match the configured version before use. Cargo-dist installs
from the upstream Git tag matching `dist-workspace.toml`, keeping release generation compatible.
Sync does not rewrite dependency requirements or the product MSRV. Refresh Rustup itself with
`rustup self update`.
Existing CI and documentation files are user-owned; merge updates rather than rerunning their
scaffolds over them. See [the migration record](docs/TEMPLATE-MIGRATION.md) for provenance,
release exceptions, and future update rules. Hosted jobs use `cargo xtask ci --job` with `format`,
`lint`, `docs`, or `test` so each check runs once. Ordinary `cargo xtask ci` still runs them all.
Run `python3 scripts/test-tooling.py` after building xtask to verify job isolation, dist compatibility,
and release-hook dry-run behavior without installing tools or publishing.
Run `python3 scripts/test-scaffolds.py` to verify generated test obligations in temporary
workspaces, including library/binary crates and both CLI entrypoint modes.

Weekly Dependabot updates cover Cargo dependencies and GitHub Actions. The cargo-dist-generated
release workflow stays byte-for-byte reproducible. `.github/actionlint.yaml` records five narrow
upstream ShellCheck exceptions for cargo-dist 0.33.0; review them when changing that version.

## Publish a workspace release

Run `cargo xtask ci` before preparing a release from a clean checkout:

```sh
cargo xtask release prepare patch --execute
cargo xtask release publish
cargo xtask release publish --execute
git push origin main
# Replace VERSION with the prepared product version.
git push origin wsr-vVERSION
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
The publisher does not push commits or tags; run the explicit Git commands after publication.
Push any other intended package tags individually after inspection. GitHub suppresses tag push
events when more than three tags are pushed together, so `--follow-tags` is unsuitable for triggering
the binary distribution workflow. Do not delete/recreate existing remote tags to retrigger CI.
The root changelog hook generates and stages files only in execute mode (`DRY_RUN=false`);
preparation previews preserve file contents and the index.

New crates use `cargo xtask add <semantic-name>` and capabilities use `cargo xtask init`.
Generated `TEST_OBLIGATION` examples intentionally fail; replace them with meaningful
behavioral assertions before completing implementation. See [AGENTS.md](AGENTS.md).

## Validation corpus

Python 3.11+ and Git are required for the corpus integration test included in ordinary Cargo tests.
Use `cargo xtask corpus` for the offline native-reference/snapshot baseline,
`cargo xtask corpus --list` for case inventory, and `cargo xtask corpus --prepare fastapi` (or `uv`) only when you
explicitly need pinned upstream Git objects. Normal tests do not fetch sources or execute upstream
workloads. WSR product inspection/execution/security cases remain planned. Follow
[the corpus guide](tests/README.md) and [TDD handoff contract](docs/TESTING-PLAN.md#11-tdd-and-agent-implementation-contract)
when implementing a feature from a validation case.
