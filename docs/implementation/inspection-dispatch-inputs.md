# Next inspection iteration: manual dispatch input definitions

Status: **delivered**. PR #26 merged as `1c565a4` after all four required checks passed
on `8ac3356`, with no review findings. Merged CLI fixture verified; local main clean. Baseline: PR #25 merged as `91fd3d5`;
literal runner-label lists are delivered. Owner authorized implementation; schedule settings remain unchanged.

## Outcome

Inspect the manual-trigger mapping and input declarations used by both real
`.github/workflows/example-*.yml` files. Report declared inputs, without selecting an event,
binding values, applying defaults or evaluating expressions. Preserve the existing scalar
`on: workflow_dispatch` behavior and output, and continue accepting fragments without `on`.

A representative excerpt:

```yaml
on:
  workflow_dispatch:
    inputs:
      mode:
        description: Choose the reference outcome
        type: choice
        options: [pass, fail]
        default: pass
        required: true
```

Before: exit 1 because `on` expects a scalar string.
Proposed successful output includes:

```text
Trigger: workflow_dispatch (not evaluated)
  Input: mode
    Type: choice
    Description: Choose the reference outcome
    Required: true
    Default: "pass" (not applied)
    Options: ["pass", "fail"]
```

Keep the existing workflow header/disclaimer and job/runner/step/needs output unchanged.
Order input identifiers deterministically; preserve option order and repetitions. Quote/escape
user-controlled values. Render only explicitly declared metadata: omitted required/default/type
values must not be invented or applied. State absence of a declared type if needed for clarity.

## Bounded syntax and model

Use the existing [trigger modeling guidance](../../crates/wsr-gha/docs/frontend-modeling.md#8-trigger-shape-and-trigger-configuration-are-distinct).
At implementation start, verify supported shapes and validation rules against the pinned official
workflow schema/converter already referenced there, and record the exact reference used.
Do not infer GitHub rules solely from the examples or this output sketch.

- Accept `on: {workflow_dispatch: null}`, an empty dispatch body, and a dispatch body with an
  optional inputs mapping, in addition to the existing scalar form. Preserve absent versus present
  declaration state. Trigger sequences and other event names/configurations remain unsupported.
- Decode input identifiers as unique strings; reject duplicates rather than overwrite them.
  Preserve spelling. Do not import job/step identifier restrictions without upstream evidence.
- Model declared input type as a closed enum for string, boolean, number, choice and environment.
  Preserve omission separately. Metadata covers description, required, default and options.
  Default values need a small scalar enum; preserve YAML types rather than stringify them.
- Validate supported field shapes strictly, including genuine booleans for required, scalar
  defaults, string descriptions/options, and mapping structure. Unknown metadata and unsupported
  events must identify the unsupported source construct rather than disappear.
- Resolve choice/default/type cross-field rules against the pinned schema before writing tests.
  Reject malformed declarations with file and structural field routes. If upstream rules conflict
  with the proposed subset, record the exact conflict before implementing a substitute contract.
- Expressions in declaration metadata/defaults/options are explicitly unevaluated and rejected in
  this static slice, retaining the current fail-before-stdout behavior. They are not resolved values.
  General dynamic-value preservation and expression AST parsing remain later work.

Keep private GHA source decoding/validation in `wsr-gha`, provider-neutral inspection summaries
in `wsr-types`, and file IO/rendering in the CLI per D-031. Extend the current trigger summary
minimally rather than adopt legacy executable IR, add a generic event engine or introduce crates.
No event binding, expression evaluation, required-input enforcement at invocation, environment
lookup, secret access, action fetching or workflow execution. No runtime TypeScript sidecar.

## Acceptance and delivery

1. Refresh live Git/PR/checkpoint state; baseline focused CLI/inspection tests. Preserve unrelated
   work; one feature PR at a time in the existing checkout, based on current origin/main.
2. Add one real CLI test using a dispatch mapping with the choice input from the examples.
   Observe the current string-shape failure, then minimally implement parsing, summary and output.
3. Add empty/null dispatch bodies and missing/explicit metadata cases. Add invalid shapes,
   duplicates, unsupported events/fields and dynamic-value diagnostics incrementally; observe
   meaningful red before each new accepted behavior or validation rule. Existing rejections count
   as regressions, not behavioral red. Verify pinned type/default/choice rules through real CLI tests.
4. Cover all five declared types and omitted type, deterministic input order, original scalar types,
   escaped text, option order, no partial output on failure, and unchanged scalar-trigger/no-trigger
   behavior. Add public parser-summary assertions for declaration presence and typed defaults.
5. Retain runner/needs/step-ID regressions, malicious scripts and source/config sentinels. No command
   runs or repository mutation. Use temporary files and ordinary Cargo integration tests.
6. Update CLI/status/testing evidence only for working behavior. Run focused tests, `cargo xtask ci`,
   Markdown target checks and `git diff --check`; review actual diff, then commit/push/create PR.
7. Merge only the verified head after required checks pass and no unresolved review findings.
   Report captured old/new CLI and parser behavior from the merged implementation.

Use extracted representative workflow fragments for this slice; do not edit the canonical examples
or claim they now inspect end-to-end. They still contain unsupported run-name, permissions,
timeout, shell/environment, action inputs and working-directory fields. Full example integration
and the finite milestone audit remain pending. Do not resume the Java worker or expand to execution.

## Cost and checkpoint

Rough estimate: **14k–24k additional tokens** including implementation, verification, PR and merge
follow-up. Typed defaults and cross-field validation make this larger than runner lists. This is
not measured billing or a conversion of remaining weekly allowance; unexpected findings may exceed it.
No reset or model change is authorized by this plan. Use supported T3 controls for account management;
do not automate redemption or patch tooling. Checkpoint revision, owned files, exact red/green evidence,
PR head/checks and next action before interruption. No full milestone completion is claimed.

## Implemented reference boundary and evidence

Pinned official reference: actions/languageservices `4043eda158e16579cc5fb1b0b07a4bce2a76f0b5`:
[workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
and [dispatch converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/workflow-dispatch.ts).
Schema descriptions supply input-identifier syntax; no step-ID length/reserved-prefix rules
were imported. Choice needs nonempty options; nonchoice rejects options. Boolean versus string
family default checks follow the converter. The converter does not check choice-default membership.
For number inputs it does not copy/check defaults; WSR retains schema-permitted scalar defaults,
without asserting stronger numeric validation. Omitted type stays absent instead of copying the
converter's injected string type. YAML coercions are not adopted: WSR keeps strict source kinds.
No new architecture or runtime compatibility is implied by these source-preservation differences.

Observed behavioral red/green and 25 CLI inspection tests plus two declaration-summary tests
are recorded in the checkpoint. The old fixture failed with `expected a string`, exit 1 and empty
stdout. The new fixture reports the choice declaration and ordered steps, exit 0. No reset consumed;
owner reported a manual account reset before implementation. No live usage amount was inferred.
