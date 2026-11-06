# Documentation layout and consolidation plan

Implemented November 6, 2026. This is a documentation maintenance record, not a product roadmap
or authorization to resume implementation.

## Old layout and problem

Project decisions, architecture, delivery gates and completion status lived in four long root
files. Test strategy, coverage, security and tooling history were loose uppercase files in docs/.
GHA contracts lived beneath crates/wsr-gha/docs/, separately from other provider guides.
Completed CLI and design-refactor plans appeared alongside current guides and paused worker work.
Agents had to discover these locations without a single context reading order.

## Plan and implemented disposition

1. Make docs/README.md the navigation hub and docs/context/README.md the agent reading guide.
2. Move authority documents together under docs/context/ without merging their distinct responsibilities.
3. Group testing, security and development guidance by purpose. Centralize the GHA design under
   docs/provider/gha/ beside the shared provider contract.
4. Separate completed records into docs/archive/. Retain paused Java contract/checkpoint paths
   for saved external prompts, with an implementation index that makes the hold explicit.
5. Repair repository Markdown links, retain small root/crate entry points, and verify content,
   references and whitespace. Preserve historical dates, evidence and completion limits.

No accepted decision, implementation status, worker authorization or runtime behavior changes.

## Canonical file mapping

| Old owner | New canonical owner |
| --- | --- |
| `PLAN.md` | [docs/context/decisions.md](../context/decisions.md) |
| `ARCHITECTURE.md` | [docs/context/architecture.md](../context/architecture.md) |
| `ROADMAP.md` | [docs/context/roadmap.md](../context/roadmap.md) |
| `CHECKLIST.md` | [docs/context/status.md](../context/status.md) |
| `docs/TESTING-PLAN.md` | [docs/testing/strategy.md](../testing/strategy.md) |
| `docs/TEST-COVERAGE.md` | [docs/testing/coverage.md](../testing/coverage.md) |
| `docs/TEMPLATE-MIGRATION.md` | [docs/development/template-migration.md](template-migration.md) |
| `docs/SECURITY-MODEL.md` | [docs/security/README.md](../security/README.md) |
| `docs/cli/SKELETON-PLAN.md` | [docs/archive/cli-skeleton.md](../archive/cli-skeleton.md) |
| `docs/implementation/design-docs-refactor.md` | [docs/archive/design-docs-refactor.md](../archive/design-docs-refactor.md) |
| `crates/wsr-gha/README.md` | [docs/provider/gha/README.md](../provider/gha/README.md) |
| `crates/wsr-gha/docs/ARCHITECTURE.md` | [docs/provider/gha/architecture.md](../provider/gha/architecture.md) |
| `crates/wsr-gha/docs/COMPATIBILITY.md` | [docs/provider/gha/compatibility.md](../provider/gha/compatibility.md) |
| `crates/wsr-gha/docs/STATE.md` | [docs/provider/gha/state.md](../provider/gha/state.md) |
| `crates/wsr-gha/docs/VALIDATION.md` | [docs/provider/gha/validation.md](../provider/gha/validation.md) |

## New navigation and retained local documentation

- docs/context/README.md: minimal reading order, authority, task routes and context maintenance.
- docs/testing/README.md: strategy versus observed coverage versus current test commands.
- docs/implementation/README.md: paused worker status and durable contract/checkpoint links.
- docs/archive/README.md: historical records, with no current execution authority.
- Root AGENTS.md: automatically discovered operational rules and link to the context guide.
- Root README, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT: public entry points and community policy.
- Root PLAN, ARCHITECTURE, ROADMAP and CHECKLIST: short compatibility pointers only. Old fragment
  bookmarks on these pointers must be updated to the canonical document; section headings were retained there.
- crates/wsr-gha/README.md: package entry point linking to centralized GHA contracts.
- crates/README.md: source/package inventory beside the workspace; changelogs stay with release tooling.
- tests/README.md and tests/reference-runs.md: commands, fixture conventions and native reference provenance beside test data.
- examples/ README files: explain their local inputs; docs/dx/ keeps simulated walkthroughs and adjacent YAML.
- docs/ subsystem guides: continue to own their separate contracts; they are indexed centrally rather than concatenated.
- tools/xtask/assets/: reusable scaffold templates remain unchanged; they describe generated projects, not this repository.

Only the root compatibility documents and package entry point retain the former paths. Other
moved files have one canonical copy; external consumers of those paths must use this mapping.
Saved Java worker paths remain valid, but their coverage references now use docs/testing/coverage.md.

## Maintenance rules

Update owners, not redirects. Link to a contract instead of copying it into handoffs or summaries.
Keep current evidence separate from planned acceptance and archived commands. Update incoming links
whenever moving an owner. A schedule must start with current context and explicit task authorization;
repository documents do not establish that an old external schedule is inactive.

## Verification — November 6, 2026

- Checked 350 relative Markdown links and heading fragments across repository documentation,
  excluding reusable scaffold templates and fenced examples: no missing destinations or headings.
- All section headings retained across the 15 canonical document moves; historical evidence,
  decisions and stop instructions remain in their owning documents.
- `cargo xtask ci --job docs` passed with Rustdoc warnings denied.
- `git diff --check` passed. No Rust source, workflow, manifest or template changed.
- Prototype stash identity remains `daee0885832df55a9a872037d3ce6741b9e6e72a`.

External websites, rendered Markdown layout and runtime behavior were not tested by these checks.
