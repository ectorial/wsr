# Inspection iteration: diagnostic text matches the category (backlog B-20, audit F29, F31, F37)

Status: **delivered** in the Schedule B architecture PR `arch/review-2026-11-13-run4`
(baseline `main` `7ecb135`). Evidence lives in the [review log](architecture-review.md#2026-11-13-run-4).

## Behavior

B-0 classified unknown keys by the pinned schema but kept serde's text, so an `Unsupported` key read
``unknown field `strategy`, expected one of …`` as if it were unrecognized. Presentation now follows the
category. Categories and routes are unchanged.

| Case | Category | Text |
| --- | --- | --- |
| key in the level's pinned-schema list (workflow `description` until B-33, job `strategy`, step `parallel`, …) | Unsupported | ``unsupported field `{key}`; this field is not inspected yet`` |
| any other key at a derived struct level | Invalid | serde text, including ``expected one of …`` |
| step `background`, `wait`, `wait-all`, `cancel` | Invalid (was Unsupported) | serde text |
| invalid key under `on` | Invalid | `unknown event '{x}'`, as scalar `on: x` |
| invalid key in `on.workflow_dispatch` or an input body | Invalid | ``unknown field `{x}` `` shown at the enclosing mapping |
| `environment: [a]` | Invalid | `expected a string or mapping`, as `concurrency` |

The text deliberately does not say "valid GitHub Actions": schema membership recognizes the key, not
its value. The parser location suffix, enclosing-mapping label and route are unchanged
(``jobs.b: unsupported field `strategy`; this field is not inspected yet at line 4 column 5``).
Handwritten nested decoders (`defaults`, `concurrency`, `environment`) already named their own keys and
are unchanged. This is an intentional presentation change from the B-0 behavior documented in
[typed errors](inspection-typed-errors.md), which preserved serde text.

### Step background keys (F37)

The pinned reader (actions/languageservices `4043eda`, `template-reader.ts` L29-30, L221) rejects `background`,
`wait`, `wait-all` and `cancel` step keys unless the `allowBackgroundSteps` feature flag is on, and the
default is false. They were `Unsupported`; they are now `Invalid`. `parallel` is accepted by the reader and
stays `Unsupported`. The follow-up in [the B-17 checkpoint](inspection-progress.md) is resolved.

### Dead parameter (F31)

`dispatch.rs` `fields` lost its `schema` parameter: every caller passed `&[]`, so its `Unsupported` branch
was unreachable.

## Acceptance and expected red

- `crates/wsr-gha/tests/diagnostics.rs`: exact message, category and route for root `description`, job
  `strategy`, step `parallel`; `bogus` unchanged; `background`, `wait`, `wait-all`, `cancel` Invalid; the
  duplicate-key assertion kept and the `cancel-timeout-minutes` text updated; dispatch input `description`
  still inspects.
- `triggers.rs` (`on` and dispatch key text), `environment.rs` (sequence message) and one exact CLI stderr
  test for job `strategy` with an `executed` sentinel; the dispatch CLI expectations were updated to the new
  text. Dogfood goldens are byte-identical.
- Expected red (observed): ``unknown field `description`, expected one of …`` for the unsupported rows;
  `background` got Unsupported; `on.bogus: unsupported field or event`; `environment: expected a string`.

## Limitations

- The location suffix still follows the parser and may be absent: custom value diagnostics point at the start of
  the enclosing mapping, unknown-field diagnostics at the key, and other decoding and syntax errors keep the
  parser's location (`name: foo\njobs: []\n` reports line 2 column 7, the sequence value; the
  [CLI guide](../cli/README.md) says so).
- `unknown field` text inside `on` bodies prints the raw key; the CLI escapes control characters at output.
