# Worker contracts and checkpoints

Java E2E implementation is **paused**. Read the current checkpoint before historical next actions.
Moving or indexing these documents does not resume the worker or create a schedule.

| Document | Owns |
| --- | --- |
| [Inspection checkpoint](inspection-progress.md) | Current autonomous inspection work, readiness evidence and next action |
| [Next inspection iteration](inspection-next.md) | Delivered static-dependency iteration (PR #22); the path is kept stable for external prompts, and current delivery evidence lives in the checkpoint |
| [Inspection backlog](backlog.md) | Ordered ready slices for Schedule A; Schedule B appends |
| [Decision requests](decision-requests.md) | Open owner decisions raised by the architecture review (Schedule B owned) |
| [Architecture review log](architecture-review.md) | Dated findings, classifications and dispositions from each Schedule B run (Schedule B owned) |
| [Typed-error iteration](inspection-typed-errors.md) | Backlog B-0: provisional typed inspection errors with category and route |
| [Run-name iteration](inspection-run-name.md) | Owner-approved literal/unevaluated run-name inspection and DX acceptance |
| [Manual-input iteration](inspection-dispatch-inputs.md) | Authorized manual trigger/input-definition inspection and acceptance gates |
| [Runner-label iteration](inspection-runner-labels.md) | Authorized literal runner-list slice, acceptance gates and token estimate |
| [Step identity iteration](inspection-step-identities.md) | Authorized explicit step-ID inspection slice and usage-aware continuity considerations |
| [Proposed beta dependencies](beta-dependencies.md) | Feature prerequisites and proposed beta gates; only inspection is currently authorized |
| [Java E2E contract](java-e2e.md) | Historical authorized scope, prerequisites, bounded TDD loop and acceptance gates |
| [Java progress](java-progress.md) | Current hold, pinned source identities, dated preparation evidence and remaining work |

Keep these paths stable for saved external prompts. A future schedule must be reconciled with
current owner instructions and [project decisions](../context/decisions.md); historical model,
branch and command records are not live configuration. Completed records belong in the
[archive](../archive/README.md), while current test evidence belongs in [coverage](../testing/coverage.md).
