# Require-on iteration: a workflow without `on` (backlog B-25)

Status: **delivered on merge of its PR**, branch `feat/b25-require-on` from `d061843` (Schedule A).
Implemented by Claude (claude-opus-5-5, burn mode); cross-reviewed by Codex gpt-6.1-sol (high): one
P3 fixed (the CLI line check left flow-style roots as written). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

The pinned `workflow-root-strict` mapping marks `on` `required: true`
([schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)).
The `on` field of `Workflow` in `crates/wsr-gha/src/lib.rs` loses its `serde(default)`, so the derived
decoder reports ``missing field `on` `` (`Invalid`, no route; the root has no parser location). A
missing field is raised at the end of the root mapping, so any other decoding error in the file is
still reported first.

Fixtures that do not test triggers change through one shared helper per suite: `common::inspect` in
`crates/wsr-gha/tests/common/mod.rs` (parses the fixture and appends only when the root mapping lacks
`on`) and `with_on` in `crates/wsr/tests/inspection.rs` (a line check, since `wsr` has no YAML
dependency; flow-style roots are left as written). Both append `on: workflow_dispatch` after the content, leaving earlier line/column
locations unchanged. The CLI contract sentence that accepted a missing `on` is replaced.

## Acceptance and expected red

`a_workflow_without_on_is_invalid` and `inspect_rejects_a_workflow_without_on` (exit 1, empty
stdout). Expected red: a workflow without `on` exits 0.
