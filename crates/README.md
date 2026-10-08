# Current workspace inventory

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
| [wsr-gha](wsr-gha/src/lib.rs) | GitHub Actions frontend placeholder |
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
