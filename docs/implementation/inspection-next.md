# Next inspection iteration: static job dependencies

Status: **delivered** via [PR #22](https://github.com/ectorial/wsr/pull/22), commit `c04243e`.
Requested after the frontend modeling review; the owner subsequently authorized implementation.
This document does not change the saved schedule. The first static
inspection slice was merged as PR #21, commit `f70ebf4`; the older preparation wording in the
[checkpoint](inspection-progress.md) is historical and must not cause another merge attempt.

## Outcome and why this slice comes next

`wsr inspect <file>` reports static job dependencies and rejects invalid dependency graphs before
printing a successful summary. This delivers useful workflow understanding and establishes graph
validation for eventual planning, without pretending to implement scheduling or execution.

The accepted inspection milestone already includes this behavior. The
[modeling catalog](../../crates/wsr-gha/docs/frontend-modeling.md) supplies patterns; it does not
require adopting every proposed type or upstream library in this iteration. Step identifiers,
runner-label lists and broader frontend forms remain subsequent slices. No full GHA-schema rewrite
or speculative universal IR is necessary to add this behavior.

## Contract

Keep the existing CLI syntax, supported source subset, deterministic job sorting, step ordering,
exit codes and no-execution behavior. Add these source forms to ordinary jobs:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo build
  test:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - run: echo test
  package:
    needs: [build, test]
    runs-on: ubuntu-latest
    steps:
      - run: echo package
```

For a job with dependencies, add a line after its runner line:

```text
Job: package
  Runner: ubuntu-latest
  Needs: build, test
```

Jobs without dependencies retain their existing output. Preserve declared dependency order; do
not present the lexicographic job summary as a topological execution order. Disconnected valid
jobs remain valid. A successful graph check means only that the static relationships are valid.

Missing `needs` means no edges. Accept a literal identifier or a sequence of literal identifiers;
an empty sequence means no edges. Reject null, scalar numbers/booleans, mappings, empty identifiers,
non-string list entries and expressions explicitly. Match identifier/reference behavior against
the pinned official reference before implementation; do not silently case-fold or normalize IDs.
Retain repeated references in the source summary; graph validation must not count duplicate edges
as separate unfinished dependencies. This is not permission to change unrelated identifier rules.

Failure leaves stdout empty, uses exit 1, and identifies the file and structural source location.
Use a route such as `jobs.test.needs[1]` for the first list entry and `jobs.test.needs` for scalar
form. Source line/column support is not expanded by this slice. Messages must distinguish:

- A dependency naming a job absent from the workflow.
- A direct self-dependency.
- A cycle involving multiple jobs; identify an actual cycle or its relevant edges, not every
  downstream job blocked by that cycle.
- A valid GHA construct that remains outside the supported static subset.

Validate the entire graph before rendering any workflow output. Diagnose unknown endpoints before
cycle analysis so malformed references are not misreported as cycles. Do not infer skipped-job,
condition, matrix, failure, cancellation or reusable-workflow semantics from static edges.

## Minimal model and ownership

| Owner | Proposed change |
| --- | --- |
| `wsr-gha` source model | Add strict scalar-or-list decoding for job `needs`; retain enough source-form information to locate diagnostics. |
| `wsr-gha` semantic validation | Resolve references within this source workflow and validate the directed graph without IO. Use a simple bounded traversal; no scheduler or new crate. |
| `wsr-types::inspection::JobSummary` | Add static dependency identifiers as source-summary data. This remains a pre-alpha inspection result, not executable IR or a stable serialized format. |
| `wsr` inspect handler | Render dependencies after complete validation; no GHA-specific graph logic in the host. |

Do not mutate/drop source edges merely to make an invalid graph pass. Share an immutable validated
graph/result with future planning only when that feature needs an API; avoid a generic graph framework
now. Respect the current no-dependency-adoption approach: upstream source guides the implementation,
and official behavior guides fixtures. No zizmor dependency or TypeScript runtime is added here.

## Red-green-refactor sequence within one feature PR

1. Establish the baseline with the current executable/inspection tests. Add one CLI test for the
   example above. Its intended red is the current unknown `needs` field, not a build/setup failure.
   Implement decoding, summary data and rendering; verify scalar/list forms and unchanged no-edge
   output. This intermediate green does not complete the feature until graph validation lands.
2. Add a missing-dependency assertion, observe red, implement resolution and a field-route diagnostic.
   Cover forward references and disconnected jobs so validation does not assume source/job order.
3. Add self-cycle and multi-job-cycle assertions through separate red/green cycles. Cover a valid
   diamond, repeated edges, and a downstream job blocked by a cycle. Invalid graphs produce no
   partial stdout. Preserve the assertions when simplifying the implementation.
4. Add malformed/dynamic `needs` cases and hostile-content/sentinel checks. Ensure scripts, hooks,
   local action paths, repository configuration and network acquisition remain untouched.
5. Review the complete diff and update only affected support/evidence/checkpoint documentation.
   Run full standard CI and deliver the complete feature through one PR; do not merge an intermediate
   implementation that accepts unresolved or cyclic graphs as supported inspection.

The feature is finished only after local acceptance/regressions, required hosted checks, review
findings and merge are complete. The user has authorized the existing PR/merge workflow; this
planning document does not expand it to releases, publication or changes outside WSR.

## Acceptance evidence

| Case | Observable expectation |
| --- | --- |
| No edges | Existing successful output remains byte-for-byte unchanged. |
| Scalar/list/empty list | Correct static edges; source list order retained; empty list renders no dependency line. |
| Forward reference, chain, diamond, disconnected nodes | Successful deterministic inspection regardless of declaration order. |
| Repeated reference | Source references retained; no false cycle or extra readiness count. |
| Unknown target | Exit 1, file/field diagnostic naming the missing target, empty stdout. |
| Self-cycle and longer cycle | Exit 1 with precise cycle evidence; blocked downstream nodes are not labelled cycle members. |
| Malformed types and expressions | Explicit structural/unsupported diagnostic; nothing silently coerced or evaluated. |
| Malicious scripts and references | Existing bytes/file inventory unchanged; no execution or acquisition. |
| Regression | Existing parser/help/remaining-placeholder contracts and current inspection fixtures still pass. |

Use Rust integration tests against the real binary, with focused compiler tests only where needed
for a meaningful graph boundary. Put new committed inspection fixtures under the existing shared
test-data conventions, not a parallel test framework. Import selected official fixtures only with
pinned provenance and required notices. Reference comparison here covers static graph behavior;
it does not claim equivalence of all GHA parsing, normalization or execution.

Commands supported by the current checkout:

```sh
cargo test -p wsr --test cli --test inspection --locked
cargo test -p wsr --test inspection --locked inspect_static_dependencies
cargo xtask ci
git diff --check
```

The named `inspect_static_dependencies` test is planned and must first be added; do not count a
zero-match filtered run as a passing acceptance test. Additional behavioral test names should
describe their individual negative/graph cases. `cargo xtask ci` includes the earlier standard
checks, so do not redundantly run `xtask check` and `xtask test` immediately before it.

## Source guidance and limits

- [Zizmor normal-job needs model](https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs): representation reference; its deserialization alone does not validate the graph.
- [Official needs schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json): accepted source forms and contextual rules.
- [Official jobs converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/jobs.ts): dependency validation behavior and diagnostics to inspect, not an algorithm to copy without testing.
- [Official reader fixtures](https://github.com/actions/languageservices/tree/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/testdata/reader): pinned negative and boundary-case references.

If exact reference behavior conflicts with a proposed contract here, report the fixture and amend
the plan before coding that conflict. Do not add a development TS sidecar merely to make this one
iteration run; a reusable official-oracle harness is a separately bounded supporting feature.

## After this iteration

Continue toward richer static inspection through named slices: step identities, runner selector
forms, trigger/input preservation, source spans, and parsed unevaluated expressions. Order those
by actual prerequisites and user-visible value. Canonical shell/Python examples require several
of those features and must not be called supported by ignoring their additional fields.

A full GHA frontend remains a sequence of verified behaviors, not an 8–15k LOC target to fill.
The execution engine, runtime selection, permission enforcement, provider distribution and beta
features beyond inspection remain outside this iteration.
