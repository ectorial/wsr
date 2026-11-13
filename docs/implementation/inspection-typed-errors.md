# Inspection iteration: provisional typed inspection errors (backlog B-0)

Status: **delivered** in [PR #29](https://github.com/ectorial/wsr/pull/29) as `d9712ec` (Schedule A, [backlog B-0](backlog.md), audit F1).
Baseline `main`: `3793749`. Evidence lives in the [checkpoint](inspection-progress.md).

## Developer outcome

`wsr_gha::inspect` returns `wsr_gha::Error` instead of `anyhow::Error`. The error is explicitly
provisional and non-stable (not a WIT, protocol or `wsr-types` contract). It carries:

- `category()`: `Category::Invalid` (invalid per the pinned GHA schema or semantically rejected by
  GitHub) or `Category::Unsupported` (valid GitHub Actions outside WSR's inspection subset).
  Unevaluated text accepted by a field (run-name) stays successful summary state.
- `route()`: a structured source route such as `jobs.b.if` or `on.push` when known at detection.

Classification happens where the failure is detected, never from message wording. Display stays
the existing message, so `wsr inspect` stderr, stdout and exit status are byte-identical; route
grammar and validation order change only in [B-1](backlog.md).

Unknown keys are classified by observing map keys during decoding: a key listed by the
[pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
for that level (workflow root, job, step; `on` events) is `Unsupported`; any other unknown key stays
conservatively `Invalid`. serde cannot carry typed payloads, so detection inside a deserializer
records the classification in a per-call side channel before returning the serde error, which keeps
parser line/column locations and duplicate-key evidence. `dispatch.rs` builds typed errors instead
of erasing them through `serde::de::Error::custom`.

## Acceptance

- `crates/wsr-gha/tests/diagnostics.rs` asserts category and route for: job `if:` (unsupported,
  `jobs.b.if`); unknown job key (invalid, `jobs.b.bogus`); dispatch unsupported event (unsupported,
  `on.push`); expression in a field with a schema expression context (unsupported, e.g.
  `jobs.b.steps[1].name`); plus a
  step-level schema key, unsupported runner-selector mapping and invalid shapes.
- Existing CLI tests unchanged; a 53-case stderr/exit corpus captured on `3793749` compares
  byte-identical after the change.
- Expected red: the type lands first with every error classified `Invalid` and no route, so the
  new tests fail on category/route assertions rather than on compilation.

Correction: B-0 classified an expression in every literal-only field as `Unsupported`, including workflow
`name` and `on`. Fields without a schema expression context are `Invalid`; see
[expression category by schema context](inspection-classification.md) (B-8).

Non-goals: route grammar unification and validation order (B-1), owning-crate graph tests (B-2),
new accepted fields (B-3..B-5, blocked on DR-002), expression evaluation, execution.
