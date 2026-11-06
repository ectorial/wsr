# Current workspace inventory

```mermaid
flowchart LR
    C[Current crate inventory] --> D[Design responsibilities]
    D --> T[Future behavior and tests]
```

These are existing crates, not approved boundaries for the redesign. Three library entry files
contain Rust declarations; the other thirteen contain documentation only. Dependencies in manifests
do not establish working subsystems. See [architecture](../ARCHITECTURE.md) and [decisions](../PLAN.md).

| Crate | Current contents |
| --- | --- |
| [wsr](wsr/src/lib.rs) | Thin executable, process entrypoint, eight stub handler modules, and executable contract tests |
| [wsr-bench](wsr-bench/src/lib.rs) | benchmark placeholder; no measured results |
| [wsr-cache](wsr-cache/src/lib.rs) | content-store placeholder |
| [wsr-cli](wsr-cli/src/lib.rs) | Typed clap argument definitions and parser contract tests |
| [wsr-client](wsr-client/src/lib.rs) | artifact-fetching placeholder |
| [wsr-engine](wsr-engine/src/lib.rs) | planning and execution engine placeholder |
| [wsr-expr](wsr-expr/src/lib.rs) | expression evaluator placeholder |
| [wsr-fs](wsr-fs/src/lib.rs) | filesystem helper placeholder |
| [wsr-gha](wsr-gha/README.md) | GitHub Actions frontend placeholder; crate-local compatibility design |
| [wsr-git](wsr-git/src/lib.rs) | hook integration placeholder |
| [wsr-resolver](wsr-resolver/src/lib.rs) | action/artifact resolution placeholder |
| [wsr-sandbox](wsr-sandbox/src/lib.rs) | component execution placeholder |
| [wsr-shell](wsr-shell/src/lib.rs) | system command execution placeholder |
| [wsr-tracing](wsr-tracing/src/lib.rs) | logging/reporting placeholder |
| [wsr-types](wsr-types/src/lib.rs) | early serializable types, legacy provider trait, and errors |
| [wsr-wasix](wsr-wasix/src/lib.rs) | legacy compatibility-backend placeholder; WASIX is not selected |

The binary forwards OS-string arguments to `wsr::main`, which parses the `wsr-cli` schema and
dispatches to eight unimplemented handler modules covering eleven operational leaf paths.
Each leaf writes `error: wsr <command>: not implemented yet` to stderr and exits with code 1.
`wsr-types` exposes early/legacy models, not a complete stable workflow or plugin contract.

The target design uses signed Component Model provider plugins through WIT, a provider-neutral
execution core, job supervision, and component/system backends. Future crate boundaries follow
validated ownership and dependencies. Do not add a crate/provider by following obsolete registry
instructions or assuming that the current trait is the accepted plugin ABI.

Repository automation lives outside this product inventory in private `tools/xtask`. It is not
a runtime subsystem. See [the template migration record](../docs/TEMPLATE-MIGRATION.md).

## Maintenance dispositions — November 6, 2026

These dispositions govern cleanup, not new product interfaces or completion claims. Retain means
preserve working behavior or a planned responsibility; it does not approve the existing crate split.
Pivot means align future behavior with the owning design, not implement it during sanitization.
Dormant and archive candidate mean preserve in place without treating the placeholder as a required
future subsystem. No crate has been deleted, removed from the workspace, or excluded from releases.

| Crate | Disposition and next gate |
| --- | --- |
| `wsr`, `wsr-cli` | Retain implemented parser/dispatch contracts and their tests; final operational CLI remains open. |
| `wsr-types` | Retain legacy API pending migration; inspect published types/serialization before replacing declarations. Specify accepted types through bounded behavior and tests. |
| `wsr-gha`, `wsr-expr`, `wsr-engine` | Retain planned compilation, evaluation and execution responsibilities; justify interfaces and crate separation through implementation. |
| `wsr-cache` | Pivot toward the [verified immutable content store](../docs/cache/README.md), not a Wasm-only cache. |
| `wsr-client`, `wsr-resolver` | Pivot toward approved acquisition and exact identity resolution under [provisioning](../docs/provisioning/README.md); resolution must not implicitly select legacy execution tiers. Consolidation requires implementation evidence. |
| `wsr-sandbox`, `wsr-shell` | Conditional retain/pivot under the [execution design](../docs/sandbox/README.md); names do not select a runtime, WASI version or enforcement mechanism. |
| `wsr-wasix` | Archive candidate: WASIX is unselected and the crate has no implementation. Preserve its name/history; do not repurpose it as an unrelated backend. Future retirement requires engine dependency and release review. |
| `wsr-fs` | Dormant/consolidation candidate: no implementation currently justifies a separate filesystem abstraction. Prefer owner-local modules until reuse warrants extraction; review Git integration dependencies first. |
| `wsr-git` | Deferred planned integration; hook implementation is not a prerequisite for useful local execution. |
| `wsr-tracing` | Pivot toward scoped diagnostics/logging under [observability](../docs/observability/README.md), not a catch-all for evidence or external reporting. |
| `wsr-bench` | Dormant private scaffold; meaningful benchmarks require measurable implemented behavior. |

## Publication and migration constraints

The crates.io API was checked on November 6, 2026. All fifteen publishable WSR crates above have
an unyanked `0.0.3` release identifying this repository. `wsr` also has `0.0.1` and `0.0.2` releases.
The [wsr-bench API](https://crates.io/api/v1/crates/wsr-bench) returned HTTP 404; its manifest is
private. Local xtask is private too; the registry's `xtask` package identifies a different repository.
This is a dated snapshot, not ownership or external consumer verification.

Published artifacts differ from the current checkout. The
[published wsr dependencies](https://crates.io/api/v1/crates/wsr/0.0.3/dependencies) include engine,
Git, tracing and types crates, whereas the current executable depends on the CLI crate.
[Published engine](https://crates.io/api/v1/crates/wsr-engine/0.0.3/dependencies) depends on
`wsr-wasix`; [published Git integration](https://crates.io/api/v1/crates/wsr-git/0.0.3/dependencies)
depends on `wsr-fs`. These are historical package dependencies, not evidence of working subsystems.

Before retiring a package or replacing public declarations:

1. Refresh registry versions, ownership and dependent-package evidence; inspect the published source
   and serialized interfaces separately from the checkout. Downloads alone do not establish usage.
2. Review future consumer dependencies and synchronized workspace release participation together.
   Do not silently set a published dependency private or leave registry/path requirements inconsistent.
3. Approve a compatible migration or explicit retirement path. Preserve existing releases and tags;
   sanitization does not authorize yanking, publication or replacing a package with unrelated behavior.
4. Verify through [the release runbook](../CONTRIBUTING.md#publish-a-workspace-release) before any
   separately authorized release. Checkout description corrections do not update existing registry metadata.
