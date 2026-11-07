# Workflow inspection worker checkpoint

## Current checkpoint — explicit step-identity iteration

PR #22 is merged as `c04243e`; the static dependency iteration is complete. Older pending-merge
wording below is historical. The owner authorized the [step-identity iteration](inspection-step-identities.md).
Current branch: `feat/inspect-step-identities`, based on `c04243e`.

Implemented: optional strict-string step IDs in the source model and `StepSummary`; rendering
as `Step N [id]: kind`; ASCII syntax/length/reserved-prefix validation; exact per-job duplicate
checks with first/repeated routes. Case and cross-job reuse are preserved. No-ID output and all
job dependency checks remain unchanged. No generic symbol table, synthetic IDs, output binding,
new dependency, hooks, toolchain or schedule changes. The existing proposed-plan edits are owned
work and belong in this feature PR.

Evidence: baseline 13 inspection + 5 CLI tests passed. Observed red/green for ID display,
invalid IDs and duplicate IDs. The completed suite has 17 inspection tests, including malformed
input, expressions, 99/100 length boundaries, case/scope behavior and non-execution sentinels.
`cargo xtask ci` passed. Next: review/push the complete feature PR, satisfy hosted checks, and
merge its verified head. Do not begin another feature before integration.

A rough token-cost planning range was communicated before implementation; it is not a verified
usage-budget conversion. No reset credit was consumed or automatic redemption enabled. Keep
account usage management outside the WSR product and do not infer a live remaining allowance
from this checkpoint. Preserve exact owned work/test/PR state on interruption.

Baseline/new binary comparison confirmed the fixture changes from unknown-field `id` (exit 1)
to ordered `Step 1 [compile]`, `Step 2 [upload]`, and ID-free `Step 3` output (exit 0).
After merge, confirm this evidence against the merged binary.
Record the merged revision alongside the next feature, not through a docs-only follow-up.
Runner selectors, richer triggers/inputs, expressions and canonical dogfood integration remain
pending; do not mark the inspection milestone complete or resume the paused Java worker.

## Bootstrap — November 6, 2026

Owner authorized a one-off run of the saved workflow-inspection schedule, readiness setup and
a first-beta dependency map. The recurring task remains scoped to inspection; the Java worker
remains paused. See [proposed beta dependencies](beta-dependencies.md).

Baseline: `main` and `origin/main` were both `04b4861`; checkout clean, no open feature PR.
`cargo xtask ci` passed (format, locked all-target/all-feature compilation and Clippy, workspace
tests including the offline corpus, and strict Rustdoc). This proves development readiness,
not workflow execution support. `cargo xtask doctor` found rustfmt, Clippy and all seven
optional tools ready. Rust was 1.99.1, Python 3.15.2; GitHub authentication was usable.
No custom `core.hooksPath` was configured. No hooks, toolchain, global configuration,
dependencies or variable names were changed. Optional tools need no refresh for inspection.

## Next action

Continue in the existing local checkout on `feat/workflow-inspection`. The uncommitted new
`beta-dependencies.md` and this checkpoint were created by this bootstrap, not unrelated user
work; include/update them alongside the first feature PR. No implementation or behavioral red
has been observed yet. Read the saved task's acceptance scope and current provider decisions
before coding. Resolve whether the first useful inspection slice fits the accepted compilation
boundary without inventing a native-provider bypass. If it does, deliver that bounded CLI
behavior through red/green and CI. If not, report the exact missing decision and alternatives.

The native manual-run control is used after bootstrap; its start acknowledgement is not a
completion claim. Keep the saved cadence, enabled state and prompt unchanged.

## First manual run — prerequisite decision

Inspected `04b4861` on `feat/workflow-inspection`; no open PR. Only the bootstrap documentation
is modified. `wsr inspect` is still a placeholder, `wsr-gha` is documentation-only, and the
legacy provider trait is explicitly not the accepted plugin contract. No WIT files or component
host implementation exist in the checkout. No product code, tests, dependencies or hooks changed;
no behavioral red or delivered feature is claimed.

D-007/D-009 require the host provider abstraction to delegate through versioned WIT to a signed
Component. D-010 requires verified, pinned activation; its pin schema remains open. The scheduled
scope explicitly forbids bypassing that architecture. Native parsing directly linked into the CLI
would introduce precisely that bypass, even if labelled inspection-only.

Owner decision required before the first successful workflow-inspection slice:

1. Authorize an explicitly temporary, built-in GHA compiler for local inspection only, with a
   provider-neutral result and no runtime compatibility claim. Record this scoped exception to
   D-009/D-010; later Component packaging must reuse the compiler and acceptance fixtures.
2. Keep the current boundary mandatory from the first slice, and authorize prerequisite work on
   the minimal versioned WIT/result/diagnostic contract, component host, trust and pinned activation
   before delivering inspection. Runtime/trust choices must be settled explicitly, not guessed.

Recommendation: option 1 for early inspection feedback, provided the owner accepts the temporary
exception; option 2 if exercising the production plugin boundary from day one is more important.
Neither option is selected by this checkpoint. Do not repeat environment preparation or substitute
documentation work on subsequent runs while this decision is pending. The schedule stays unchanged.

## Owner resolution and first implemented slice — November 6, 2026

Owner selected option 1. D-031 records the temporary built-in inspection exception; D-009 remains
mandatory for the target plugin architecture. No further approval is needed for this bounded path.
The beta dependency map remains a proposal outside the inspection milestone.

Implemented: the real CLI reads explicit source, invokes the pure built-in `wsr-gha` compiler,
and renders provider-neutral source summaries. Jobs are sorted; steps preserve source order;
runner requirements and static action references remain unresolved. Unsupported fields, expressions,
invalid shapes and duplicate identifiers fail explicitly before any stdout report. See the exact
[CLI subset](../cli/README.md#static-workflow-inspection).

Evidence: existing CLI tests passed before product changes. The first inspection acceptance test
failed on the intended placeholder diagnostic, then passed. Additional observed red/green cycles
covered dynamic fields, invalid/empty input, and static manual-trigger inspection. All four new
CLI acceptance tests, the five existing executable contract tests, and full `cargo xtask ci` pass.
No hooks, global environment, product versions, registry releases or legacy type declarations changed.
No workload execution or runtime enforcement is claimed.

Delivery: preparing the first feature PR from baseline `04b4861`; hosted checks and merge remain
pending. Next run must finish this PR before another slice. After merge, add step identifiers and
runner label lists with acceptance tests, then static `needs` graph validation, broader unsupported/
dynamic diagnostics and canonical example-workflow integration. Do not mark the entire milestone
complete or pause its schedule yet. Compiler/model-only changes do not count as delivered features.
