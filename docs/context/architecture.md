# wsr architecture

## Status and ownership

This document separates the current scaffold from the target design. Accepted decisions and
unresolved choices live in [PLAN.md](decisions.md); this overview does not approve additional choices.
The implementation checklist is [CHECKLIST.md](status.md).

## Current scaffold

- [Cargo.toml](../../Cargo.toml) defines sixteen product crates plus private `tools/xtask` repository automation.
- [wsr-cli](../../crates/wsr-cli/src/lib.rs) defines typed arguments and help.
- [The binary](../../crates/wsr/src/bin/wsr.rs) forwards OS-string arguments to
  [the application entrypoint](../../crates/wsr/src/lib.rs), which owns parsing, diagnostics,
  exit codes, and [command dispatch](../../crates/wsr/src/commands/mod.rs).
- Eight handler modules cover eleven operational leaf paths. Static inspection reads an explicit
  workflow file through the temporary built-in frontend (D-031); the other ten paths remain stubs.
- Parser/executable contract tests cover help/version, arguments, placeholders, and unchanged
  repository sentinels. They do not establish workflow execution or isolation.
- [wsr-types](../../crates/wsr-types/src/lib.rs) contains early serializable types, a provider trait,
  and errors. These do not establish a complete workflow compatibility model or stable protocol.
- `wsr-gha` implements the built-in static source summary; the remaining twelve library entry files
  contain documentation only. There is no component
  runtime, scheduler, expression evaluator, action resolver, or sandbox enforcement implementation.

Current source names such as `Vault` and `Workshop` are legacy scaffold identifiers. They do not
select production runtimes. Existing dependency edges and crate boundaries are inventory, not
requirements for the redesign. The [crate maintenance dispositions](../../crates/README.md#maintenance-dispositions--november-6-2026)
identify planned pivots, dormant placeholders and archive candidates. Published package descriptions
and dependencies can differ from this checkout; cleanup does not rewrite existing releases.

## Preserved prototype

Local stash `daee0885832df55a9a872037d3ce6741b9e6e72a`, labeled
`codex: preserve local work before syncing origin/main 2026-07-29`, preserves implementation
absent from this checkout. It includes 25 tracked-file changes and 49 formerly untracked files
in third parent `3a40ea7e0d14217e408bd853acc3fedb5fd61313`.

The audit identified inspection/planning, workflow IR, snapshots/evidence, a development command
backend, webhook verification/deduplication, invocation state, and Checks payload projection.
It counted 29 Rust test bodies; these were inspected, not executed. The command backend is not a
verified boundary for malicious workloads. GitHub adapter code is not a deployed CI service.

Inspect recovery in isolation before selecting reusable work. Preserve the original stash;
do not apply it blindly over the current workspace or infer current support from prototype code.

## Target architecture

The local engine executes compiled workflows independently of GitHub's control plane. Verified
cached inputs enable disconnected execution when the workload needs no external service. Approved
cache misses use routine acquisition. Changed YAML triggers resolution, not a warning.

```mermaid
flowchart LR
    I[Workflow inputs] --> P[Compile and validate]
    P --> T[Provision tools]
    T --> Q[Schedule jobs]
    Q --> E[Dispatch and execute]
    E --> R[Local results]
    R -.-> C[Optional reporting]
```

This is a responsibility map. Dynamic conditions/outputs are evaluated at their proper phases,
and readiness/capacity is rechecked at dispatch. Policy applies to acquisition and execution;
observability spans all phases. Exact APIs and serialization are still open.

| Owner | Contract and handoff | Guide |
| --- | --- | --- |
| Configuration/bootstrap | Explicit invocation, defaults and verified pins | [Configuration](../config/README.md) |
| Provider compiler | Source semantics to canonical model, diagnostics and provenance | [Providers](../provider/README.md) |
| Planner/validator | Valid structure, dynamic binding, platform/capability/tool requirements | [Planning](../planning/README.md) |
| Provisioner | Verified dependency closure and private environment readiness | [Provisioning](../provisioning/README.md) |
| Content store | Immutable verified objects, leases and atomic publication | [Cache](../cache/README.md) |
| Trusted policy | Effective acquisition and execution grants | [Policy](../policy/README.md) |
| Scheduler/dispatcher | Eligible job selection, global admission and supervisor launch | [Engine](../engine/README.md) |
| Job supervisor | Step lifecycle, state/outputs, services, cancellation and cleanup | [Engine](../engine/README.md) |
| Backend | Enforced platform/capabilities and bounded process/component execution | [Backends](../sandbox/README.md) |
| Observability | Repository-scoped logs/resources and authorized device correlation | [Observability](../observability/README.md) |
| Evidence | Local result/input/artifact identities and optional signing | [Evidence](../evidence/README.md) |
| Connector | Optional external publication, status and retry | [Integrations](../sync/README.md) |

## Workflow and environment compatibility

Signed provider components compile into the provider-neutral model. They terminate at compilation.
The core evaluates translated expressions and handles job/step dependencies and outcomes. GHA-specific
lowering/state semantics live in [wsr-gha](../provider/gha/README.md). Bun is selected for qualified
JavaScript actions, inside an enforced system boundary. QuickJS, automatic action recompilation and
WASIX are unselected. A component mapping needs an explicitly compatible artifact.

Runner profiles describe platform/ABI, defaults and available tools. Package catalogs describe exact
sources, verification, dependency closure and activation. D-025 preflight preserves setup-action
timing and effects. Complete discovery of arbitrary scripts is impossible through YAML parsing alone;
supported dynamic requirements and produced-tool policy need explicit contracts.

GHA labels are compatibility requirements, not interchangeable host labels. Initial execution scope
remains Linux jobs from macOS/Linux hosts. CPU architecture coverage, Linux environment substrate,
WASI/runtime and system outer isolation remain open. Avoiding Docker as a provisioning dependency is
the product direction; workloads explicitly requesting container semantics still need a supported
container path or a diagnostic. Linux system execution from macOS requires a compatible Linux boundary.

## Concurrency and state

D-028/D-032 admit jobs across repositories against one memory budget (default and minimum 5 GB,
matching `ubuntu-slim`; per-job 5 GB ceiling, not a reservation). Policy checks remain. Ready queue, blocked dependencies, provisioning and running
work have distinct wait reasons. Background service/foreground ownership, queue fairness and crash
recovery remain proposals to specify. No crate layout or daemon API is selected by these diagrams.

Jobs retain private mutable workspace/credentials/logs. Authorized state passes between steps through
the supervisor. Step outputs feed job output expressions and dependent jobs at defined phases.
Shared cache bytes do not grant cross-repository access or execution authority.

## Trust and result boundaries

Trusted policy authorizes workflow requests and backends enforce denials. Missing enforcement blocks
startup. The explicit broad-access profile requires owner selection; its scope and syntax remain open.
No failed step is replayed automatically with wider permissions. See [security](../security/README.md).

Local results survive connector outages. Optional attestations bind signer claims to identities under
an explicit verification policy. Signatures/containment do not establish truthful arbitrary tests.
No execution or enforcement path is implemented by this documentation refactor.
