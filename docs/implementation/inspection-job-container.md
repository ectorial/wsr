# Job container iteration: job `container` as source text (backlog B-29)

Status: **in progress** on branch `feat/b29-job-container` from `094c0e0` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, steady mode; `wsr-types` `Container` under
D-033); cross-review by Codex gpt-6.1-sol (high). Evidence lives in the [checkpoint](inspection-progress.md).

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
| empty or blank string form | Unsupported (the runner treats an empty image as no container) |
| null, boolean or number (whole value, `image`, `options`, `username`, `password`) | Unsupported (converted to a string, B-13) |
| sequence or other non-mapping shape | Invalid |
| mapping key with `${{` | Unsupported (valid in an expression context) |
| unknown or non-string key | Invalid |
| `image`, `username`, `password`: empty, blank or null | Invalid (`non-empty-string`) |
| `options`: any string | literal or unevaluated |
| `env` | as step `env` (B-5): nonempty keys unique ignoring case, scalar values keeping their type |
| `ports`, `volumes`: sequence of nonempty strings; numbers and booleans keep their type | items literal or unevaluated; empty or null items Invalid |
| whole `env`, `ports`, `volumes` or `credentials` given as `${{` text | Unsupported |
| missing `image`, no expression in the container | Invalid (``missing field `image` ``) |
| missing `image` with an expression elsewhere | Unsupported (the converter defers the check) |
| `container` with `uses` | Invalid (unchanged, B-23) |

## Acceptance and expected red

- `crates/wsr-gha/tests/container.rs`: the string and mapping forms decode to the expected
  `Container`, and one category and route test covers each table row.
- A CLI golden with a hostile image (`$(touch executed)`, escape and bidi characters) and a literal
  password that is not printed. A PATH shim for `docker`, `podman` and `sh` proves no pull or start
  (an `executed` sentinel).

Expected red: ``jobs.a: unsupported field `container`; this field is not inspected yet``.
