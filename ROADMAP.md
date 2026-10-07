# wsr delivery roadmap

This is a staged direction, not a release/version schedule. [PLAN.md](PLAN.md) owns decisions;
[CHECKLIST.md](CHECKLIST.md) owns implementation status. Creating a crate or writing a design does
not complete a runtime milestone. All execution milestones below remain planned.

## 0. Reconcile evidence and decisions

Preserve and inspect the July 29 prototype in an isolated recovery checkout. Record what compiles,
which tests run, what can be retained, and which assumptions conflict with the accepted design.
Resolve the first compatibility subset, security backend, wildcard scope, platform architectures,
and component host contract before calling a slice implementation-ready.

**Gate:** one coherent specification and recovery report; original stash retained. Documentation
can be synchronized before every design choice is resolved, provided open choices stay explicit.

## 1. Explain a workflow without executing it

Connect provider discovery/pinning and the compilation boundary to deterministic inspection,
source diagnostics, a minimal canonical model, and plan validation. Classify unknown or unsupported
behavior instead of silently discarding it. Define evaluation phases for dynamic expressions.

**Gate:** representative fixtures produce expected diagnostics and plans; invalid plans are rejected;
inspection executes no repository-controlled commands. The current `inspect` handler is still a stub.

## 2. Prove component security

Choose a runtime/WASI baseline and a minimal versioned WIT contract. Execute one useful component
with explicit grants, input identity, bounded resources, and structured results.

**Gate:** permitted behavior works and adversarial attempts at ungranted access fail. Runtime and
host assumptions are documented; signatures are not treated as proof of safe behavior.

## 3. Prove isolated Linux system execution

Select a backend for Linux jobs from macOS and Linux machines. Specify workspace/tool/service state,
filesystem/network/process restrictions, secrets, resource limits, and cleanup. Implement default
policy and the explicitly authorized broad-access exception within its eventual approved scope.

**Gate:** adversarial probes and timeout/cancellation cleanup pass; unavailable enforcement blocks
execution; workflow edits cannot grant permissions to themselves. Native macOS/Windows jobs are rejected.

## 4. Deliver useful local CI

Combine compilation, planning, both execution paths, job supervision, and results for a small tested
workflow. Expand dependencies, expressions, outputs, matrices, and actions only through named fixtures.
Keep immutable content separate from secrets, mutable run state, and unredacted logs.

**Gate:** users can inspect, run, and diagnose the supported subset; failure/skip/cancellation semantics
are tested; effective grants and environment identity are visible. No full compatibility claim.

## 5. Reuse the engine inside GitHub Actions

Define a wrapper contract for job/pipeline selection, outer scheduling, inputs, cancellation,
credentials, reporting, and recursion prevention. Qualify the available execution boundary without
assuming nested virtualization works everywhere.

**Gate:** the same engine executes the supported subset locally and in Actions; environment differences
and security exceptions are reported; broad access does not expand GitHub token permissions.

## Later possibilities

Independent CI with a GitHub App, durable scheduling, workers, and Checks reporting comes after
the local/Actions path. Remote/team cache, catalogs, more providers, native macOS/Windows jobs,
and deployment workflows require separate evidence and decisions. No repositories or version
numbers are reserved for them by this roadmap.

Performance comparisons follow working execution and a reproducible workload/environment definition.
A content digest identifies an input; it does not make external dependencies deterministic.
