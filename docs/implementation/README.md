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
| [Diagnostic-route iteration](inspection-diagnostic-routes.md) | Backlog B-1: one-based route grammar and job-ID-first validation order |
| [Classification iteration](inspection-classification.md) | Backlog B-8: expression category by schema context, schema-valid values no longer Invalid, job-ID-first route safety |
| [Case-insensitive identifier iteration](inspection-case-ids.md) | Backlog B-21: job and dispatch input IDs repeated ignoring case are invalid |
| [Diagnostic-text iteration](inspection-diagnostic-text.md) | Backlog B-20: unsupported-field wording matches the category; background step keys are invalid |
| [Job-environment iteration](inspection-job-environment.md) | Backlog B-19: job `environment`, never selected; no protection rules evaluated |
| [Job-outputs iteration](inspection-job-outputs.md) | Backlog B-18: job `outputs` by name, never resolved |
| [Continue-on-error iteration](inspection-continue-timeout.md) | Backlog B-17: job/step `continue-on-error` and step `timeout-minutes`, never applied |
| [Concurrency iteration](inspection-concurrency.md) | Backlog B-16: workflow and job `concurrency`, never applied |
| [Trigger-event iteration](inspection-trigger-events.md) | Backlog B-15: bare `on` events in source order; event configuration unsupported |
| [Shell and action-reference iteration](inspection-shell-uses.md) | Backlog B-14: unknown shells and malformed `uses` references rejected as invalid |
| [Scalar-conversion iteration](inspection-scalar-conversion.md) | Backlog B-13: null, boolean and number conversions classified per field as Unsupported or Invalid |
| [Conditions iteration](inspection-conditions.md) | Backlog B-12: job and step `if` reported as written, never evaluated |
| [Env and defaults iteration](inspection-env-defaults.md) | Backlog B-11: workflow/job `env` and `defaults.run`, never bound, merged or applied |
| [Dogfood-guard iteration](inspection-dogfood-guard.md) | Backlog B-10: every `example-*.yml` must have a golden inspection case |
| [Format-escape iteration](inspection-format-escape.md) | Backlog B-9: Unicode format/bidi characters escaped in inspection stderr |
| [Size-warning iteration](inspection-size-warning.md) | Backlog B-7: GitHub 500 KB workflow-size warning (D-032) |
| [Dogfood acceptance iteration](inspection-dogfood.md) | Backlog B-6: golden `wsr inspect` output on the real `example-*.yml` with no-execution sentinels |
| [Step-input iteration](inspection-step-inputs.md) | Backlog B-5: step `env`, `with` and `working-directory`, never bound, passed or resolved |
| [Timeout and shell iteration](inspection-timeout-shell.md) | Backlog B-4: job `timeout-minutes` and step `shell`, never applied or selected |
| [Permissions iteration](inspection-permissions.md) | Backlog B-3: requested workflow/job `permissions`, never granted |
| [Compiler-test iteration](inspection-compiler-tests.md) | Backlog B-2: owning-crate characterization tests for `wsr_gha::inspect` |
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
