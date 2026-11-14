# Workflow description iteration (backlog B-33)

Scope: retain the root `description` as written and render it as `Description: <escaped text>` on
the line after the workflow name. Pinned basis: `workflow-root-strict` maps `description` to
`workflow-description`, a plain `string` with no expression context
([schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json));
the converter (`model/convert.ts`) does not read it. `WorkflowSummary.description` is a D-033
temporary GHA-derived field.

| Source | Category |
| --- | --- |
| string, including empty or whitespace | Inspected as written |
| null | Absent, like `name` |
| boolean or number | Unsupported; the pinned reader converts it to a string (B-13) |
| `${{` text | Invalid; no expression context (B-8) |
| list or mapping | Invalid |

Acceptance: `crates/wsr-gha/tests/description.rs` (retention and categories) and a CLI golden
with hostile text, the `executed` sentinel and the positive-controlled PATH trap, plus the exact
`${{` diagnostic.

Contract change: `crates/wsr-gha/tests/diagnostics.rs` used root `description` as its
recognized-but-uninspected sample in two tests; job `snapshot` replaces it. The root level now has
no recognized-but-uninspected key.

Expected red: ``unsupported field `description`; this field is not inspected yet at line 2 column 1``.
