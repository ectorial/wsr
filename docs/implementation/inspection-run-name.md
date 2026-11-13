# Next inspection iteration: unevaluated run-name inspection

Status: **delivered** in [PR #27](https://github.com/ectorial/wsr/pull/27), commit `fd574dc`
(owner-approved Schedule A backlog item 1). Baseline `main`: `1c565a4`, PR #26 merged. Evidence lives in the
[checkpoint](inspection-progress.md).

## Developer outcome

Both canonical example workflows declare `run-name` with an embedded inputs expression.
The current inspector rejects the field. Accept and report it as source metadata, distinguishing
literal text from potentially dynamic text without evaluating or claiming expression validity.
This is a bounded step toward the finite milestone's dynamic-source classification and real
example acceptance; it does not finish either gate.

Input:

```yaml
name: Example shell reference
run-name: "Shell reference — ${{ inputs.mode }}"
on: workflow_dispatch
jobs:
  shell:
    runs-on: ubuntu-latest
    steps:
      - run: echo hello
```

Before: exit 1, unknown field `run-name`, empty stdout.
After: exit 0, existing report plus a line following the workflow name:

```text
Run name: Shell reference — ${{ inputs.mode }} (unevaluated; expression syntax not validated)
```

A literal run name uses `Run name: Local reference (literal)`.
Omission adds no line and preserves existing output byte-for-byte. Escape control characters in
all displayed text. Preserve the decoded YAML scalar text in the source summary, not necessarily
its original YAML quoting/block-style lexeme. Never apply the input default to replace inputs.mode.

## Parser boundary

Add optional strict-string `run-name` decoding in `wsr-gha`, source metadata in
`wsr-types::inspection`, and rendering in the host under D-031. Missing is allowed; explicit null,
booleans, numbers, lists and mappings fail before stdout. Empty/whitespace-only values fail with
a field diagnostic, consistent with this inspection subset's required-text handling.

Use a small source-text enum, with literal and unevaluated-template variants retaining the entire
raw decoded string. Detection of a `${{` opening fence classifies potential dynamic source; it
is not an expression parser. Preserve text with multiple, multiline or unterminated fences as
unevaluated and state that syntax is not validated. Do not split on the first closing braces or
claim that malformed expression syntax is valid. A later expression AST slice must replace this
provisional classification with tested grammar/offset/context handling.

The [modeling guidance](../../crates/wsr-gha/docs/frontend-modeling.md#4-mixed-strings-need-a-sequence-of-segments)
proposes ordered template segments and AST parsing. This slice deliberately delivers only honest
source inspection, with no lowering/normalization/compatibility claim. Do not introduce a segment
parser, a generic expression framework or an executable IR merely to display one field. If strict
AST validation becomes necessary for the agreed output contract, stop and revise this scope rather
than silently build a partial grammar. Reuse of the source-text type in other fields requires later
field-specific authorization/tests; do not relax existing literal-only validation globally.

## Acceptance and delivery

1. Refresh current Git/PR/checkpoint state and run existing focused CLI tests. Preserve unrelated
   local work. Use the existing checkout and one branch from current origin/main.
2. Add one real CLI test with the example run-name and a hostile run/action sentinel fixture.
   Observe unknown-field red, then implement decode, source preservation and rendering together.
3. Add literal/omitted cases, strict wrong-shape and blank diagnostics, multiple/multiline/open
   fences and escaped-control cases. Dynamic source must remain visibly unevaluated, with no
   input lookup, defaults applied or expression validation claim. Observe intended new behavior
   failures before minimal implementation; existing rejections are regression evidence.
4. Add public parser-summary assertions for literal versus unevaluated raw text. Preserve all
   existing manual-input, runner, needs and step-ID assertions. Assert stable output/exit behavior,
   empty stdout on errors, source/config sentinels unchanged and no executed/acquired action.
5. Verify pinned official schema allows run-name strings/expressions, recording the exact source.
   Do not adopt upstream runtime event/context binding. Update CLI/status/testing evidence only
   for observed behavior. Run focused tests, `cargo xtask ci`, links and `git diff --check`.
6. Review and deliver one feature PR; merge only its verified head after required checks and no
   unresolved findings. End with a DX report: old/new CLI example, parser representation changes,
   regression evidence, remaining unsupported behavior and exact next action.

Do not edit the canonical examples to make them pass. Their permissions, job timeout, step shell,
environment, action inputs and working-directory remain unsupported after this slice. Full examples
and final milestone audit remain pending. No expression evaluation, execution, Component activation,
provider acquisition, new dependencies/crates or tooling changes. Java worker stays paused.

## Estimate and continuity

Rough estimate: **6k–12k additional tokens**, including implementation, tests/docs, PR and merge
follow-up. This is workload planning, not a measured account quota. Owner reported a manual usage
reset; no automatic reset or account action is needed. Save revision, owned files, red/green evidence,
PR head/checks and next action on interruption; never count an opened PR as completed delivery.
