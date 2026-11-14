# Job cancellation timeout inspection (B-34)

## Scope and pinned basis

Job `cancel-timeout-minutes` is retained in `JobSummary.cancel_timeout_minutes` as source data
(D-033), next to job `timeout-minutes`. Inspection never applies either timeout or executes a job.

The [pinned schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json#L1791)
assigns `number-strategy-context`: `github`, `inputs`, `vars`, `needs`, `strategy`, `matrix`, and an
unrestricted number definition (no minimum or integer constraint). The
[pinned job converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/job.ts)
adds no timeout value check. This slice reuses the existing job/step timeout decoder; positivity
is a WSR inspection subset restriction, not a constraint from the pin.

| Source | Inspection category |
| --- | --- |
| Finite positive number, including fractional | Literal source text, not applied |
| Text containing `${{` | Unevaluated; syntax not validated; not applied |
| Zero or negative number | Unsupported, expected a positive number |
| Boolean, ordinary string, null, sequence, non-finite number | Invalid, expected a positive number |
| Combined with job `uses`, either key order | Invalid, cannot be combined with `uses` |

## Acceptance and evidence

After adding the type field with no decoder behavior, `cargo test --workspace --locked` passed.
`cargo test -p wsr-gha --test cancel_timeout --locked` then observed two failures and one passing
existing-behavior check (`uses`). Observed red text:

```text
jobs.a: unsupported field `cancel-timeout-minutes`; this field is not inspected yet at line 4 column 5
```

The new CLI golden, run with `cargo test -p wsr --test inspection inspect_reports_cancel_timeouts --locked`,
observed:

```text
error: wsr inspect: ci.yml: jobs.a: unsupported field `cancel-timeout-minutes`; this field is not inspected yet at line 6 column 5
```

Acceptance covers literals, fractional numbers, expression retention, all table categories, absence,
and both call-job key orders. The CLI golden checks literal and hostile expression output, escaping
of ESC and bidi text, a missing `executed` sentinel, unchanged source, and exactly one directory entry,
using the existing positive-controlled PATH execution traps. Existing B-27 tests remain required.

Intended B-34 contract change: the job Unsupported sample in `diagnostics.rs` changes from
`cancel-timeout-minutes` to `services`. No other existing assertions change.

Final validation: `cargo xtask ci` passed (format, locked workspace compilation and Clippy,
workspace tests and strict documentation). The cancellation-timeout category suite passed 3 tests,
CLI inspection passed 72, diagnostics passed 9, and call-job regressions passed 4.
`git diff --check` passed.
