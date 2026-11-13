# Owner decision requests

Appended by the Schedule B architecture review. Each request is open until the owner records a decision
in [decisions.md](../context/decisions.md). Do not implement a request's options before resolution. DR-001 was not filed
(audit F1 was reclassified as a bounded correction, backlog B-0).

## DR-002 Naming/ownership of GHA-shaped inspection summaries (audit F4, 2026-11-13) — resolved by D-033 (option 1)

- Question: `wsr-types::inspection` is documented as provider-neutral but holds GHA concepts (`ManualTrigger`,
  `DispatchInputType::Environment`, `run_name`, `${{` fence); host hard-codes `workflow_dispatch`.
- Options: (1) relabel as temporary GHA-derived source summaries (docs/rustdoc only); (2) move trigger/input/run-name
  types into `wsr-gha`; (3) generalize now.
- Recommendation: (1) now; settle the neutral model with the canonical-IR stability and source/translation-stage
  queue items.
- Blocks: further trigger/expression slices that add GHA-only fields to `wsr-types`.

## DR-003 Workflow file size limit (audit F7, 2026-11-13) — resolved by D-032 (no size cap; GitHub 500 KB warning)

- Question: should `wsr inspect` cap input size (regular files are now required)?
- Options: (1) fixed cap (e.g. 1 MiB) with a clear diagnostic; (2) cap with override flag; (3) no cap.
- Recommendation: (1); GitHub workflow files are small and inputs are untrusted.
- Blocks: nothing in the backlog; hardening only.

## DR-004 Roadmap Gate 1 ordering (audit G1, 2026-11-13) — resolved by D-034 (option 1)

- Question: Gate 1's remaining items (provider discovery/pinning, minimal canonical model, plan validation,
  evaluation phases) require the D-010 pin schema, the canonical-IR boundary and the Component host, which are
  Gate 2 runtime/WIT choices. Split Gate 1?
- Options: (1) split into "D-031 static inspection" and "Component compilation" gates; (2) keep and reorder
  dependencies; (3) leave as is.
- Recommendation: (1), so the inspection milestone can be completed and verified independently.
- Blocks: declaring Gate 1 progress beyond static inspection; any scheduling of canonical-model work.
