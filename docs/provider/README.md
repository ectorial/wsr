# Workflow providers

```mermaid
flowchart LR
    S[Provider source] --> C[Compile and diagnose]
    C --> M[Canonical model and provenance]
    M --> P[Core planner]
```

## Current scaffold

[wsr-gha](../../crates/wsr-gha/src/lib.rs) provides a temporary built-in static inspection compiler
under D-031. It returns temporary GHA-derived source summaries (D-033), not executable canonical IR.
[Supported inputs and diagnostics](../cli/README.md#static-workflow-inspection) are bounded explicitly.
The existing [WorkflowProvider trait](../../crates/wsr-types/src/lib.rs) remains legacy; signed
Component provider execution and full GHA lowering are not implemented.

## Accepted compilation boundary — planned

A host-side Rust abstraction delegates to signed WebAssembly Component provider plugins through
versioned WIT. Plugins receive explicit inputs, parse provider defaults and expressions, preserve
source traceability, classify compatibility, and lower to a provider-neutral canonical model.
They terminate at compilation; planning/execution does not call back into opaque provider expressions.

One source workflow maps to one canonical workflow; individual constructs can lower to several
nodes. Compatibility categories are native, faithfully emulated, fallback-required, unsupported,
and unknown. Core/extension admission and serialized IR stability remain open. No universal or
lossless-source-mapping claim is made.

Planned `wsr init` creates root `wsr.toml` with GitHub Actions as the primary provider.
That configuration can declare multiple providers and their defaults; an explicit invocation or
unambiguous workflow path can select another provider. A conflict or remaining ambiguity is an
error. A provider compiles source syntax; hosted execution administration belongs to a separate
integration adapter. See [configuration](../config/README.md) and D-023 in [PLAN.md](../context/decisions.md).

GitHub Actions is first. Its [GHA provider design](gha/README.md) owns action
compatibility, Bun qualification, lifecycle lowering, and GHA state-channel semantics. Bun is selected
for compatible JavaScript actions in the isolated system backend; provider compilation does not invoke it.
The packaging of runtime compatibility helpers and canonical extension admission remain open.

Other provider frontends are a design extension path with no approved
release schedule. Cross-provider source translation is outside scope. Broader grants do not
implement absent semantics or APIs. See [PLAN.md](../context/decisions.md) and [ROADMAP.md](../context/roadmap.md).
