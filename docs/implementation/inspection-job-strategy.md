# Job strategy iteration (B-30)

Status: **delivered on merge of its PR**.
Implemented by Codex gpt-6.1-sol (reasoningEffort medium), steady mode; cross-review by Claude.

## Scope

Job strategies are retained as source values on runner and reusable-workflow call jobs.
The D-033 temporary `Strategy` type is added to `JobSummary`. No expression is evaluated,
no job is multiplied, and no include/exclude matching is performed. Expansion remains core-owned
under D-012. The CLI labels the declaration `Strategy (not expanded)`.
Matrix mappings are retained as deterministic normalized YAML text, preserving scalar types,
nested mappings, sequences, order and include/exclude data, rather than YAML presentation details
(comments, quoting choices or aliases). Terminal output escapes this text, including controls and bidi.

## Categories and pinned basis

[Schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
and [job converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/job.ts)
are pinned to `4043eda158e16579cc5fb1b0b07a4bce2a76f0b5`.
The converter retains strategy tokens without extra checks; there is no separate strategy/matrix
converter in that directory. Nested definitions inherit the strategy expression context.

| Source | Category and basis |
| --- | --- |
| strategy mapping, including empty | Inspected; `strategy.mapping` has three optional properties |
| whole strategy expression | Unevaluated; strategy has github/inputs/vars/needs context |
| other strategy scalar or sequence | Invalid; mapping required, B-13 conversion cannot produce one |
| unknown strategy key or non-string key | Invalid; closed properties and string mapping keys |
| expression mapping key | Unsupported; inherited expression context allows dynamic keys, which are not inspected |
| fail-fast boolean | Literal; `boolean` |
| fail-fast expression | Unevaluated; inherited context |
| other fail-fast value, including quoted boolean/null/number | Invalid; reader string fallback does not create a boolean |
| max-parallel number, including zero and negative | Literal; `number` has no minimum |
| max-parallel expression | Unevaluated; inherited context |
| other max-parallel value, including quoted number/null/bool | Invalid; reader string fallback does not create a number |
| matrix mapping or whole expression | Normalized YAML or Unevaluated; `matrix.mapping` with inherited context |
| other matrix shape | Invalid; mapping required |
| matrix variable sequence, including empty | Normalized YAML; loose values are `sequence` |
| whole variable sequence expression | Unevaluated within matrix source; inherited context |
| other variable value | Invalid; sequence required |
| include/exclude sequences of mappings or expression items | Normalized YAML; `matrix-filter` and `matrix-filter-item` with inherited context |
| whole include/exclude expression | Unevaluated within matrix source; inherited context |
| other include/exclude shape/item | Invalid; sequence/mapping required |
| empty or null variable/filter-item key | Invalid; `non-empty-string` loose key type; whitespace accepted |
| boolean or number variable/filter-item key | Unsupported; the reader converts it to a string ("Convert to StringToken"), as for step `env` keys |
| arbitrary nested variable sequence items/filter values | Normalized YAML; `sequence` items and filter loose values are `any`; no matching validation |
| strategy with uses, including beyond scan budget | Inspected; `workflow-job` explicitly lists strategy and converter retains it; no FACTORY_ONLY/presence prohibition added |
| decoder error | Invalid, detail withheld so arbitrary matrix text cannot leak through a mismatched YAML tag | This includes serde_yaml's alias repetition limit, a WSR resource limit rather than a pinned-parser rejection |
| matrices costing more than twice the workflow source size together (alias amplification; B-32) | Unsupported, not rendered; a string costs its decoded length, any other node 1, so a literal matrix of any size fits (D-032) |

## Acceptance and evidence

Expected and observed behavioral red: ``jobs.a: unsupported field `strategy`; this field is not inspected yet at line 4 column 5``.
`cargo test -p wsr-gha --test strategy` exercises source preservation, categories and one unexpanded job.
CLI golden covers hostile command text, escape and bidi text, an `executed` sentinel and unchanged input.
Existing unsupported-field tests now use still-unsupported `services`; its call-job prohibition stays Invalid.
The repository workflow pin now requires release.yml success and its exact strategy expression report.
This proves inspection only, not execution compatibility. Cross-reviewed by Claude (claude-opus-5-5, medium). Two P2s and three P3s were found; four are fixed
and one is open:
- P2, fixed: alias amplification turned a 400 KB workflow into 444 MB of stdout. The matrix is now
  size-bounded before rendering, and a `wsr-gha` test and a CLI test cover it.
- P2, fixed: boolean or number matrix and filter-item keys were Invalid; they are now Unsupported.
- P3, fixed: the label read "YAML source"; it is now `Matrix (normalized YAML)`.
- P3, fixed: the repetition limit was undocumented; the table now records it.
- P3, open: the hostile golden's ESC is pre-escaped by the YAML emitter (U+202E still exercises the
  escape path).

Residual: aliases are still materialized by the decoder before the bound applies. This is the same
pre-existing memory follow-up recorded for step and call `with` values in B-23.

Validation: five strategy tests and 67 CLI inspection tests pass. `cargo xtask ci` and
`git diff --check` pass; this is local verification, not hosted CI.

## B-31 / audit F42 (2026-11-14)

Matrix mappings now reject keys duplicated ignoring case, using B-21's uppercase normalization,
including axes, include/exclude items and mappings nested in arbitrary matrix values. Failures are
`Invalid` at the repeated key's route; sequence indexes remain one-based. Distinct keys still pass.
The alias/size guard is unchanged and runs before this recursive duplicate check.

Observed red: `cargo test -q -p wsr-gha --test strategy matrix_keys_are_unique_ignoring_case`
failed with `accepted duplicate matrix keys: {a: [1], A: [2]}` (0 passed, 1 failed).
The CLI regression failed with `assertion left == right failed`, `left: Some(0)`, `right: Some(1)`.
Green: all 7 strategy tests and 69 CLI inspection tests pass, including the repeated-key route,
empty stdout and exit 1. B-28/B-30 goldens now share the repository-workflow positive-controlled
PATH traps, retaining their full output and executed-sentinel assertions.

Final regression: `cargo fmt --all` followed by `cargo xtask ci` passed (locked all-target/all-feature
workspace compilation, Clippy with warnings denied, workspace tests and doctests, strict rustdoc).
CLI probes were written outside the repository: `services` remains Unsupported; call-job strategy
inspects successfully and renders "not expanded" without loading the reference. The B-33..B-36
expected diagnostics in the backlog were captured from live CLI probes.

Cross-review follow-up: like the pinned reader (template-reader.ts L180-195), keys containing `${{`
are not compared, though their values still are; boolean or number keys are `Unsupported` at any
depth (`non-string key is not inspected yet`), not only at the matrix and include/exclude level.
Observed red: the `{a: [{'${{ x }}': 1, '${{ X }}': 2}]}` probe was rejected as a duplicate.
