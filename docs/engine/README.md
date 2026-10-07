# Planning and execution engine

## Current scaffold

[wsr-engine](../../crates/wsr-engine/src/lib.rs) and
[wsr-expr](../../crates/wsr-expr/src/lib.rs) contain documentation only.
Scheduling, expression evaluation, matrix expansion, and workflow execution are not implemented.

## Accepted responsibilities — planned

The provider-neutral core consumes compiled workflows and explicit inputs. It validates the plan,
evaluates canonical expressions at appropriate phases, manages dependencies/conditions/outputs,
and supervises jobs and steps. Dynamic outputs cannot all be evaluated before execution.

The job supervisor owns workspace, tool, service, output, cancellation, and timeout lifetimes.
Step environments receive separate grants and export allowed state through controlled channels.
A system step does not automatically widen other steps' permissions. Shared state can carry risk.
Backend selection happens before a side-effecting step; partial execution is not automatically
replayed in a more permissive backend.

Precise matrix/failure/retry/state semantics and the first supported subset still need specification
and fixtures. The same engine later runs inside Actions; wrapper scheduling/credential/reporting
ownership remains open. See [ARCHITECTURE.md](../../ARCHITECTURE.md) and [PLAN.md](../../PLAN.md).
