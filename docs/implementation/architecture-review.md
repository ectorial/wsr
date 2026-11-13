# Architecture review log (Schedule B)

Owned by the Schedule B architecture review. Each run appends a section; earlier sections are history.
Findings are classified (a) bounded correction, (b) owner decision — see
[decision requests](decision-requests.md), (c) not acted on. Ready work goes to the [backlog](backlog.md).

## 2026-11-13 run 1

- Audited range: `a641361..fd574dc` (PRs #16–#27; first audit, last ~10 commits). No open Schedule A PR.
- Auditor: claude-opus-5-5 (effort high), burn mode. Second opinion on the highest-risk finding (F1):
  Codex gpt-6.1-sol (reasoning high). Implementer: claude-sonnet-5-5 (high). Cross-review: Codex gpt-6.1-sol.
- Verified clean: no legacy `WorkflowIR`/`WorkflowProvider` use outside `wsr-types/src/lib.rs`; inspection
  summaries derive no `Serialize`; no IO in `wsr-gha`/`wsr-types`; validation stays in `wsr-gha`; stdout
  empty on failure; deterministic ordering; exit codes 0/1/2 as documented (except F8).

| ID | Finding | Class | Disposition |
| --- | --- | --- | --- |
| F1 | Compiler failure contract is untyped `anyhow` strings; no invalid/unsupported category; negative tests string-coupled to host stderr (medium after second opinion) | (a) | Backlog B-0; reclassified from (b) — see note |
| F2 | Untrusted control bytes reach stderr unescaped (serde messages, `on:` keys) | (a) | Fixed in this PR |
| F3 | Zero-based serde routes vs one-based semantic routes for the same step; `jobs.b: runs-on[2]` vs `jobs.b.runs-on[2]` | (a) | Backlog B-1 |
| F4 | `wsr-types::inspection` labelled provider-neutral but GHA-shaped | (b) | DR-002 |
| F5 | Job IDs validated after `needs`; inconsistent quoting/double escaping; pseudo-routes; duplicate-job message lacks ID | (a) | Backlog B-1 |
| F6 | Graph/ID/shape rules tested only through the host binary, not in owning `wsr-gha` | (a) | Backlog B-2 |
| F7 | Unbounded read: non-regular files (`/dev/zero`, FIFO, symlinks) hang inspect | (a)+(b) | Regular-file check fixed in this PR; size cap DR-003 |
| F8 | Broken stdout pipe panics (exit 101) | (a) | Fixed in this PR |
| F9 | Stale checkpoints/status (run-name, dependency iteration, "all handlers are stubs", coverage count) | (a) | Fixed in this PR |
| F10 | Corpus INS-001..005 still `planned` though INS-004/005 behaviorally met | (c) | Catalog IDs optional; wire when convenient |
| F11 | Workflow without `on` accepted as an inspection fragment | (c) | Documented; revisit with B-0 categories |
| F12 | `wsr-gha` depends on unused placeholder `wsr-expr`; `serde_yaml 0.9.34+deprecated` | (c) | Revisit before Component packaging |
| G1 | Roadmap Gate 1 remainder depends on Gate 2 Component/WIT and open pin schema | (b) | DR-004 |

- F1 note: the Opus audit proposed an owner decision. The Codex second opinion found the evidence accurate but
  severity medium, and the fix bounded within D-002/D-031: frontend-modeling.md calls diagnostic-schema choices
  bounded implementation decisions, reserving stable protocol/WIT contracts for the owner. Triage accepted that;
  no DR-001 was filed, so decision requests start at DR-002.
- PR: see the architecture PR that adds this file (`arch/review-2026-11-13`).
- Cross-review (Codex gpt-6.1-sol): two P2s fixed before PR — the regular-file check now re-validates the
  opened handle and reads only through it (a path swapped to a FIFO after the check can at worst block `open`,
  never be parsed; non-blocking open would need a platform flag or new dependency, left as residual); backlog
  B-3..B-6 marked blocked on DR-002 instead of ready. Ready backlog depth: 3 (B-0..B-2).
- Next focus: verify B-0/B-1 land typed, non-string-coupled diagnostics;
  first permissions/env slices for untrusted-content escaping and no-binding claims.
