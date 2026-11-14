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
| matrix mapping or whole expression | Source YAML or Unevaluated; `matrix.mapping` with inherited context |
| other matrix shape | Invalid; mapping required |
| matrix variable sequence, including empty | Source YAML; loose values are `sequence` |
| whole variable sequence expression | Unevaluated within matrix source; inherited context |
| other variable value | Invalid; sequence required |
| include/exclude sequences of mappings or expression items | Source YAML; `matrix-filter` and `matrix-filter-item` with inherited context |
| whole include/exclude expression | Unevaluated within matrix source; inherited context |
| other include/exclude shape/item | Invalid; sequence/mapping required |
| empty or non-string variable/filter-item key | Invalid; `non-empty-string` loose key type; whitespace accepted |
| arbitrary nested variable sequence items/filter values | Source YAML; `sequence` items and filter loose values are `any`; no matching validation |
| strategy with uses, including beyond scan budget | Inspected; `workflow-job` explicitly lists strategy and converter retains it; no FACTORY_ONLY/presence prohibition added |
| decoder error | Invalid, detail withheld so arbitrary matrix text cannot leak through a mismatched YAML tag |

## Acceptance and evidence

Expected and observed behavioral red: ``jobs.a: unsupported field `strategy`; this field is not inspected yet at line 4 column 5``.
`cargo test -p wsr-gha --test strategy` exercises source preservation, categories and one unexpanded job.
CLI golden covers hostile command text, escape and bidi text, an `executed` sentinel and unchanged input.
Existing unsupported-field tests now use still-unsupported `services`; its call-job prohibition stays Invalid.
The repository workflow pin now requires release.yml success and its exact strategy expression report.
This proves inspection only, not execution compatibility. Cross-review is performed by the orchestrator.

Validation: five strategy tests and 67 CLI inspection tests pass. `cargo xtask ci` and
`git diff --check` pass; this is local verification, not hosted CI.
