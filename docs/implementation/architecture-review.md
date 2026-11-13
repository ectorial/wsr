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

## 2026-11-13 run 2

- Audited range: `fd574dc..e3a8373` (PRs #28–#37; #37 B-7 merged during the audit). Auditor: claude-opus-5-5
  (effort high), burn mode. Second opinion on the highest-risk finding (F13): Codex gpt-6.1-sol (reasoning
  high), which confirmed F13 field-by-field, lowered severity to medium diagnostic correctness/low operational
  impact, and narrowed F14 (whitespace `shell` stays Invalid). Implementer: claude-sonnet-5-5 (high).
  Cross-review: Codex gpt-6.1-sol.
- Verified clean: no legacy `WorkflowIR`/`WorkflowProvider` use outside `wsr-types/src/lib.rs`; D-033 label
  applied; no IO in `wsr-gha`/`wsr-types`; `anyhow` removed from `wsr-gha`; exit codes 0/1/2 and empty stdout
  on failure; deterministic ordering; stdout escaping of all new fields; no grant/binding/execution claims;
  dogfood golden test byte-exact with PATH/sentinel/no-mutation checks.

| ID | Finding | Class | Disposition |
| --- | --- | --- | --- |
| F13 | `${{` in fields with no schema expression context classified Unsupported instead of Invalid; diagnostics.rs asserted the wrong category | (a) | Fixed in this PR (B-8) |
| F14 | Subset rules report schema-valid `timeout-minutes: 0`, empty `working-directory`/`run` as Invalid | (a) | Fixed in this PR (B-8); whitespace `shell` stays Invalid (runner rejects it) |
| F15 | stderr escapes control characters only; bidi/format characters (U+202E) reach stderr raw | (a) | Backlog B-9 |
| F16 | Job body decoded before job-ID validation lets a job ID forge a route (`jobs.a.steps[9]`) | (a) | Fixed in this PR (B-8) |
| F17 | Numbers re-rendered (`1e3` → `1000.0`), not source spelling | (c) | Nothing bound; dogfood unaffected |
| F18 | Stale status/CLI claims (dogfood pending; size decision open) | (a) | Size lines fixed by #37; remainder fixed in this PR |
| F19 | B-7 reads 500 KB as 512,000 bytes; 500,001–512,000 gets no warning if GitHub means 500,000 | (c) | Interpretation documented in CLI contract; revisit if GitHub states the unit |
| F20 | Dogfood test hard-codes two file names; a new `example-*.yml` gets no golden coverage | (a) | Backlog B-10 |
| F12 | carried: placeholder `wsr-expr` dependency, deprecated serde_yaml | (c) | Revisit before Component packaging |

- No decision requests filed. Gate 1a is claimed met by #37; with F13/F14 corrected its "explicit
  unsupported/dynamic classification" deliverable now matches the pinned schema.
- PR: this PR (`arch/review-2026-11-13-run2`). Ready backlog depth after this PR: 4 (B-9..B-12).
- Next focus: B-11/B-12 new-field slices follow the B-8 category rule (schema expression context decides
  Invalid vs Unsupported); Gate 1b remains blocked on pin schema, canonical-IR boundary and Component host contract.

## 2026-11-13 run 3

- Audited range: `e3a8373..6f58e96` (PRs #38–#42; #38 is run 2's architecture PR). No open Schedule A PR.
  Auditor: claude-opus-5-5 (effort high), burn mode. Second opinion on F21: Codex gpt-6.1-sol (reasoning high),
  which confirmed the conversion, class (a), medium diagnostic correctness/low operational impact; it corrected
  the proposal (step `shell` stays Invalid, and B-11's `defaults.run.shell` moves Unsupported→Invalid because
  the runner rejects unknown bare shells; `defaults.run.working-directory` null stays Invalid; step `id: true`
  becomes Unsupported) and found F28. Implementer: claude-sonnet-5-5 (high). Cross-review: Codex gpt-6.1-sol
  (reasoning high): no code findings; P2 (B-17 conflated boolean and numeric fields) and P3 (slice record
  misstated step `env`/`with` numbers as Unsupported) fixed before PR.
- Verified clean: no legacy `WorkflowIR`/`WorkflowProvider` outside `wsr-types/src/lib.rs`; no IO in
  `wsr-gha`/`wsr-types`; D-033 label on `RunDefaults` and the condition field; ownership; sorted `env` keys; empty
  stdout on failure; exit codes; B-8 categories for the B-11/B-12 contexts match the pinned schema (20+ probes);
  Cf/bidi escaping; F16 holds; no bind/apply/evaluate claims; dogfood golden byte-exact.

| ID | Finding | Class | Disposition |
| --- | --- | --- | --- |
| F21 | numbers/booleans in string fields Invalid though pinned reader converts them | (a) | Fixed in this PR (B-13) |
| F22 | null in empty-allowed string fields / env / with values Invalid | (a) | Fixed in this PR (B-13) |
| F23 | `Condition:` line omits "expression syntax not validated" though any text is accepted | (a) | Fixed in this PR |
| F24 | CLI guide says success has empty stderr, contradicting the 500 KB warning | (a) | Fixed in this PR |
| F25 | literal backslash sequence in a key renders like an escaped control char in the route prefix | (c) | Documented choice; revisit with machine-readable diagnostics |
| F26 | B-10 red proof placed an untracked fixture in the real .github/workflows | (c) | Nothing left behind; red proofs use temp copies only |
| F27 | pinned schema only linked, not vendored | (c) | Optional SHA-256 note later |
| F28 | quoted `shell: "true"` and `uses: "5"` accepted though the runner rejects them | (a) | Backlog B-14 |
| F12 | carried: placeholder `wsr-expr` dependency, deprecated serde_yaml | (c) | Revisit before Component packaging |

- No decision requests filed.
- PR: this PR (`arch/review-2026-11-13-run3`). Ready backlog depth after this PR: 6 (B-14..B-19).
- Next focus: B-15/B-16 unblock the repo's own non-dogfood workflows; keep the B-8/B-13 category rules for every
  new field; Gate 1b remains blocked on pin schema, canonical-IR boundary and Component host contract.
