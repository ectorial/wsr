# wsr architecture

## Status and ownership

This document separates the current scaffold from the target design. Accepted decisions and
unresolved choices live in [PLAN.md](PLAN.md); this overview does not approve additional choices.
The implementation checklist is [CHECKLIST.md](CHECKLIST.md).

## Current scaffold

- [Cargo.toml](Cargo.toml) defines sixteen product crates plus private `tools/xtask` repository automation.
- [wsr-cli](crates/wsr-cli/src/lib.rs) parses arguments.
- [The binary](crates/wsr/src/bin/wsr.rs) passes parsed arguments to
  [command dispatch](crates/wsr/src/commands/mod.rs).
- All eight command handlers immediately report “not yet implemented.”
- [wsr-types](crates/wsr-types/src/lib.rs) contains early serializable types, a provider trait,
  and errors. These do not establish a complete workflow compatibility model or stable protocol.
- The remaining thirteen library entry files contain documentation only. There is no component
  runtime, scheduler, expression evaluator, action resolver, or sandbox enforcement implementation.

Current source names such as `Vault` and `Workshop` are legacy scaffold identifiers. They do not
select production runtimes. Existing dependency edges and crate boundaries are inventory, not
requirements for the redesign. Package/help descriptions may still reflect the legacy scaffold.

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

## Target architecture — accepted design, not implemented

| Boundary | Intended responsibility |
| --- | --- |
| CLI and integrations | Supply explicit inputs and present plans/results; local use needs no hosted controller |
| Provider host | Discover, pin, verify, and instantiate signed provider components through versioned WIT |
| Provider compiler | Parse source/defaults/expressions, retain provenance, classify compatibility, lower to canonical IR |
| Planner and policy | Bind available inputs, analyze dependencies/platforms/capabilities, authorize grants, reject unsupported plans |
| Execution core | Evaluate canonical expressions at their defined phases; manage jobs, steps, outputs, cancellation, and failures |
| Job supervisor | Own workspace/tool/service lifetimes and explicit state transfer between isolated steps |
| Component backend | Expose only authorized typed host interfaces and enforce resource/lifecycle limits |
| System backend | Execute Linux tools with enforceable isolation and explicit grants; fail closed when requirements cannot be met |
| Local content store | Share immutable verified content across repositories; keep secrets and mutable run state separate |
| Results and evidence | Identify inputs, tools, policy, environment, effective grants, outcomes, and artifacts |

The intended flow is source and explicit event inputs → provider compilation → canonical model
and compatibility diagnostics → planning/policy → job supervision and selected backends → results.
Provider components terminate at compilation: execution has no opaque provider-expression callbacks.
Dynamic values are evaluated at the appropriate phase rather than all being known before execution.

## Execution and trust

The first deployment runs Linux jobs from macOS and Linux machines. Linux system execution on
macOS needs a VM-backed Linux environment. The Linux backend, outer boundary, CPU architectures,
images, concrete Component Model runtime, and WASI version are still open choices.

Deny-by-default is the normal policy. A workload requests access; owner or administrator policy
authorizes it; the chosen backend enforces grants and denials. The system assumes malicious code
on a single-user machine. See [the security model](docs/SECURITY-MODEL.md).

An explicit broad-access compatibility profile is accepted as a feature direction. Its wildcard
scope, syntax, and activation granularity are not settled. It must never activate automatically
when a permission check or backend fails, and must not claim the default profile's narrower protection.

A job is a state/lifecycle boundary, while step environments have separate grants. Shared workspace,
tools, services, and outputs need explicit contracts. Selecting a system backend for one step does
not grant every step the same authority. No partially executed step is automatically replayed in a
more permissive backend. Shared files may carry malicious state between steps.

## Compatibility and integration

Compatibility is feature-specific: native, faithfully emulated, fallback-required, unsupported,
or unknown. Provider compilation may lower one source construct into multiple canonical nodes;
it must preserve source traceability and diagnose unrepresentable semantics. Cross-provider source
translation is outside the design. No action is assumed convertible from JavaScript to Wasm.

Reusing the engine inside GitHub Actions follows useful local execution. Wrapper scheduling,
cancellation, credentials, and recursion prevention remain open. Local and CI runs share engine
semantics, not a guarantee of identical environments or outcomes.

A GitHub App, remote scheduler, and worker fleet are later possibilities. The organization profile
generator is unrelated to them. No separate action catalog, registry, or service repository is
required for the first local milestone.
