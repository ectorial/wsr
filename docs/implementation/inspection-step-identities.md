# Next inspection iteration: explicit step identities

Status: **owner-authorized; implemented locally, PR integration pending**. The owner authorized
implementation after reviewing this plan; completion evidence is recorded in the worker checkpoint.
Baseline: PR #22 is merged on `main` as `c04243e`; static job dependencies are delivered.
The owner requested this next plan after reviewing the old/new dependency-inspection behavior.

## Product outcome

`wsr inspect <file>` accepts explicit `steps[].id`, preserves it in the source summary, displays
it with its step, and rejects malformed or duplicate explicit IDs before printing a summary.
Step identities are prerequisites for later step-output and expression-reference work. This
iteration does not resolve those references, evaluate expressions or execute workflows.

Current behavior rejects `id` as an unknown step field. Proposed behavior:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: compile
        name: Compile sources
        run: echo compile
      - id: upload
        uses: owner/action@v1
```

```text
Job: build
  Runner: ubuntu-latest
  Step 1 [compile]: run (Compile sources)
  Step 2 [upload]: uses owner/action@v1
```

The output above is the relevant excerpt, not a replacement for the existing workflow header.
Steps with no explicit ID retain their old output byte-for-byte. IDs do not change step order,
select steps, rename actions, or become generated execution identities. Job dependency output
and graph validation remain unchanged.

## Source rules and representation

Use the [modeling references](../../crates/wsr-gha/docs/frontend-modeling.md), especially shared
step metadata, source preservation and validation boundaries. Keep this change small:

- Add an optional explicit ID to the private step source struct and `StepSummary`.
- Use the existing strict string decoding pattern: missing is allowed; explicit null, numbers,
  booleans, sequences and mappings are rejected.
- Check the pinned official identifier rules: nonempty ASCII letter/underscore first; remaining
  characters ASCII letters/digits/underscore/hyphen; fewer than 100 characters; no `__` prefix.
  An expression is unsupported input, not an ID to normalize or evaluate.
- Reject repeated explicit IDs within one job. Preserve spelling and use the pinned reference's
  exact comparison; do not silently case-fold identifiers. The same ID may occur in different jobs.
- Retain unnamed steps as unnamed. Do not implement GitHub's synthetic-ID generation in this slice.
- Report the file and structural step/ID route; duplicate diagnostics should identify the first
  and repeated step indices. Keep existing one-based index conventions. Expanding source spans
  is a future feature.

Keep GHA rules in `wsr-gha`, summary data in `wsr-types::inspection`, and rendering in the host.
Do not introduce a generic symbol table, new crate, third-party parser dependency or metadata
refactor merely to add this field. Existing job-ID rules are outside scope: sharing validation
helpers must not silently change those older contracts.

## Red-green delivery sequence

1. Establish the current baseline; add one real CLI acceptance test for command/action steps with
   IDs. Expected red: current unknown `id` field. Implement decode, preservation and rendering.
2. Add invalid explicit-ID cases and observe their intended failures; implement strict shape,
   syntax, length/reserved-prefix and unevaluated-expression checks.
3. Add same-job duplicate-ID failure; implement per-job checks. Verify reuse across jobs, mixed
   identified/unidentified steps, and the pinned reference's case behavior without normalization.
4. Run existing dependency and no-ID output regressions, hostile-content/sentinel checks, full
   `cargo xtask ci`, Markdown link checks and `git diff --check`.
5. Deliver one complete feature PR from merged main. Merge only the tested head after required
   hosted checks pass and review findings are resolved. Provide captured CLI and parser-model
   before/after comparisons from the old and merged binaries, as in the previous iteration.

Do not merge an intermediate version that accepts IDs but silently drops them or permits duplicate
IDs. Each new test must exercise actual supported behavior, not create ignored/expected-failure
scaffolds. The feature's finish line is merged behavior with evidence, not elapsed time or LOC.

## Acceptance inventory

| Input | Required result |
| --- | --- |
| Valid IDs on command/action steps | Retained in source summaries and displayed with the correct step |
| No IDs | Existing output unchanged |
| Mixed identified/unidentified steps | Source order retained, no generated ID inserted |
| Same explicit ID twice in one job | Exit 1, empty stdout, diagnostic identifies the duplicate scope and steps |
| Same ID in different jobs | Accepted |
| Case variants | Pinned official behavior, original spelling retained |
| Empty/invalid/reserved/overlength ID | Explicit field diagnostic; test length boundaries of 99 and 100 |
| Wrong YAML type or expression | Rejected without coercion/evaluation or a partial stdout summary |
| Existing needs graphs | All current valid/invalid graph contracts preserved |
| Hostile script/action path and sentinels | No execution, acquisition or repository mutation |

Use `cargo test -p wsr --test cli --test inspection --locked` for the baseline and focused suite.
New descriptive test names must exist before using filters; zero matched tests are not evidence.
Run `cargo xtask ci` before committing. No environment refresh or hooks changes are prerequisites
unless a concrete baseline failure demonstrates one.

## Pinned reference and unresolved conflicts

- [Official identifier builder](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/id-builder.ts)
- [Official step conversion](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/steps.ts)
- [Rust shared step metadata](https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs)

If a pinned fixture conflicts with a rule above, document the exact case and revise the proposed
contract before implementing it. A full official-parser harness is not required to add one field;
do not introduce a runtime TS boundary. Runner selectors, triggers/inputs, expression ASTs,
output binding, scheduling and execution remain subsequent features.

## Usage-aware continuity — external to WSR

Keep a durable checkpoint before implementation and after each completed behavior: revision,
branch/PR, test result, owned uncommitted files and exact remaining action. Do not begin another
slice while this one is unfinished. Do not infer weekly allowance from conversation length.

On inspection, installed T3 Code nightly `0.1.8-nightly.20261106.4262` contains native Codex
reset-credit discovery/redemption and a `Use reset` confirmation flow. Its Codex backend invokes
the account reset API and refreshes limits. No threshold-based auto-redemption setting or agent
MCP redemption control was identified. This was a read-only implementation inspection, not live
credit availability verification or a successful redemption. No credit was consumed.

Prefer T3-owned usage management over repository scripts. Any later automatic policy needs live
same-account usage/credit data, an explicit threshold and credit budget, idempotent redemption and
verified refreshed limits. Missing data must not trigger repeated attempts or spending. Context
clearing/session restart is a different operation from redeeming a usage-reset credit. Do not
change the current schedule, redeem credits, buy credits, switch models or patch T3 as part of
this feature plan. If no supported reset control is available, checkpoint and report the blocker.
