# Next inspection iteration: literal runner-label lists

Status: **owner-authorized; implementation complete locally, integration pending**. Baseline: PR #24 merged as `b22b339` on `main`;
explicit step identities are delivered. The owner instructed continuation; schedule settings remain unchanged.

## Outcome and scope

Accept a nonempty literal string list in `jobs.<id>.runs-on` through the real
`wsr inspect <file>` command. Preserve the existing scalar-string contract and its output.
This is the next small extension of literal runner requirements in the inspection milestone,
following the [frontend modeling guidance](../../crates/wsr-gha/docs/frontend-modeling.md#7-runner-syntax-is-independent-of-a-selected-backend).
It makes label-based workflows inspectable without selecting a runner backend.

Example input:

```yaml
jobs:
  build:
    runs-on: [self-hosted, linux, x64]
    steps:
      - id: compile
        run: echo compile
```

Before: exit 1, no stdout, a string-shape diagnostic at the runner field.
After: exit 0, empty stderr, including:

```text
Job: build
  Runner labels: ["self-hosted", "linux", "x64"]
  Step 1 [compile]: run
```

Keep the existing workflow header and execution-compatibility disclaimer. Use quoted/escaped
list elements so punctuation and control characters cannot obscure label boundaries.
A single-element list remains a list in the output; a scalar retains `Runner: ubuntu-latest`.
Preserve list order, exact spelling and repetitions; do not normalize, deduplicate, infer an OS,
resolve labels, discover machines, or claim that any runner satisfies the requirements.

Runner-group objects, matrices, expressions, reusable-workflow jobs and additional trigger
shapes remain explicitly unsupported. In particular, an expression in a scalar or any list
member must diagnose an unevaluated dynamic value, never become a literal resolved label.
Do not evaluate expressions or add a runtime TypeScript sidecar.

## Minimal model and implementation boundary

- In the private GHA source model, replace the string-only runner field with a strict
  scalar-or-sequence representation, retaining original form. Follow existing decoding patterns;
  YAML numbers and booleans must not coerce into strings.
- In `wsr-types::inspection`, represent the source requirement as a small enum with scalar-label
  and label-list variants. This replaces the current pre-alpha `JobSummary.runner: String`;
  it is source-summary evolution, not a canonical executable IR or stable serialization contract.
- Keep syntax/semantic checks in `wsr-gha`, summary data in `wsr-types`, and rendering in the host,
  as required by D-031. No new crate, dependency, generic selector framework or changes to legacy IR.
- Validate the entire input before rendering. Empty lists and blank members fail. Missing/null,
  booleans, numbers, mixed or nested lists and mappings fail explicitly with file/field context.
  Semantic list-member diagnostics use one-based routes such as `jobs.build.runs-on[2]`.
  Decoder errors retain available line/column context; exact source-span infrastructure is deferred.
- Mapping rejection must make the unsupported runner-selector form clear, without asserting that
  the source is invalid GHA. If that requires a specialized diagnostic, keep it local to this field.

## Acceptance and red-green sequence

1. Run `cargo test -p wsr --test cli --test inspection --locked` on merged main.
   Baseline should retain the existing 5 CLI and 17 inspection tests; inspect live counts.
2. Add a real CLI acceptance fixture with scalar and list jobs, identified/unidentified command
   and action steps, and a static needs edge. Observe the intended list-shape rejection.
   Implement strict decoding, summary preservation and deterministic output together.
3. Add invalid-shape/blank-list/member/expression cases, observing meaningful behavioral red
   where the current implementation accepts an invalid value. Already-rejected cases are
   regression evidence, not invented red evidence. Implement indexed semantic diagnostics.
4. Verify order, punctuation/escaping, exact case, repeated labels, a single-element list,
   byte-for-byte existing scalar output, and dependency/step-ID behavior. Include source/model
   assertions so preserving scalar-versus-list form is tested independently of display.
5. Retain malicious-run/action-path and repository sentinel assertions. Inspection must read
   only the supplied workflow, never execute scripts, fetch/open actions, provision tools,
   inspect host capabilities or mutate repository state. Use temporary directories for fixtures.
6. Review the complete diff; run focused tests, `cargo xtask ci`, Markdown link checks and
   `git diff --check`. Update CLI/status/testing documentation for verified behavior only.
7. Deliver one feature PR from current `origin/main`, using the existing checkout. Merge only
   the tested head after required hosted checks and relevant review findings are resolved.
   Capture old/new CLI output and source-model comparison against the merged binary.

Do not broaden this slice to unblock the full example workflows: their other unsupported fields
must still fail explicitly. Canonical `example-*.yml` acceptance remains a later milestone gate.
No hooks, environment refresh, dependency churn or unrelated repairs are prerequisites.

## Cost and continuity

Planning estimate for implementation through PR review: **8,000–16,000 additional tokens**,
with roughly **2,000–4,000** more reserved for hosted-check follow-up and merge verification.
This is a rough workload estimate, not measured billing or a conversion of weekly percentage
into tokens. Unexpected regressions or review findings can exceed it.

Before implementation, record live revision/branch/dirty state and any supported account-limit
observation. Do not infer reset availability from old cached values. No reset is authorized by
this plan, and no supported agent redemption control has been established. Keep reset management
in T3; do not patch it or add repository scripts. If interrupted, checkpoint exact owned files,
red/green evidence, PR head/check state and next action. Never restart completed work or spend
credits automatically. Stop after this iteration; the remaining inspection milestone is not complete.
