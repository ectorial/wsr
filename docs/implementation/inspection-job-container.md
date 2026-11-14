# Job container iteration: job `container` as source text (backlog B-29)

Status: **delivered on merge of its PR**, branch `feat/b29-job-container` from `094c0e0` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, steady mode; `wsr-types` `Container` under
D-033); cross-reviewed by Codex gpt-6.1-sol (high), and all three findings are fixed. The P1 was a
password quoted by a YAML decoder error under a mismatched tag; the decoder detail is now withheld.
The P2 was that whitespace-only `image`/`username`/`password` were Invalid; the pinned check is
length only. The P3 was a table row that contradicted the null handling. Evidence lives in the
[checkpoint](inspection-progress.md).

## Scope

A job `container` is reported as written and rendered `Container (not pulled; not started):`. No image
is pulled, no registry is contacted, no credential is used, no container is created or started,
nothing is bound, exposed or mounted. The CLI never prints a password.

Pinned basis ([schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
[converter](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/container.ts)):
`container` has an expression context (github, inputs, vars, needs, strategy, matrix) that nested
values inherit, and is `string` or `container-mapping`. `credentials` adds secrets and env, and
`container-env` values are `string-runner-context`. `convertToJobContainer` skips all of its checks
when any token in the container is an expression. Without expressions, it requires `image`.

| Source | Category |
| --- | --- |
| string form, `${{` text | image, literal or unevaluated |
| empty string form | Unsupported (the runner treats an empty image as no container); whitespace is kept as written |
| null, boolean or number as the whole value or `options` | Unsupported (converted to a string, B-13) |
| boolean or number `image`, `username`, `password` | Unsupported (converted to a nonempty string) |
| sequence or other non-mapping shape | Invalid |
| mapping key with `${{` | Unsupported (valid in an expression context) |
| unknown or non-string key | Invalid |
| `image`, `username`, `password`: empty or null | Invalid (`non-empty-string` checks length only; whitespace is kept) |
| `options`: any string | literal or unevaluated |
| `env` | as step `env` (B-5): nonempty keys unique ignoring case, scalar values keeping their type |
| `ports`, `volumes`: sequence of nonempty strings; numbers and booleans keep their type | items literal or unevaluated; empty or null items Invalid |
| whole `env`, `ports`, `volumes` or `credentials` given as `${{` text | Unsupported |
| missing `image`, no expression in the container | Invalid (``missing field `image` ``) |
| missing `image` with an expression elsewhere | Unsupported (the converter defers the check) |
| `container` with `uses` | Invalid (unchanged, B-23; also when the scan budget is exhausted and `container` precedes `uses`) |
| YAML decoder error inside the container (e.g. a mismatched `!!int` tag) | Invalid, with the decoder's detail withheld so no value is quoted |

## Acceptance and expected red

- `crates/wsr-gha/tests/container.rs`: the string and mapping forms decode to the expected
  `Container`, and one category and route test covers each table row.
- A CLI golden with a hostile image (`$(touch executed)`, escape and bidi characters) and a literal
  password that is not printed. A PATH shim for `docker`, `podman` and `sh` proves no pull or start
  (an `executed` sentinel).

Expected red: ``jobs.a: unsupported field `container`; this field is not inspected yet``.
