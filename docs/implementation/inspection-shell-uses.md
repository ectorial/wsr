# Shell and action-reference iteration: semantic validation (backlog B-14)

Status: **delivered on merge of its PR**, branch `feat/b14-shell-uses-validation` from `51bb95d`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; mechanical slice, no
`wsr-types` change); cross-reviewed by Codex gpt-6.1-sol (high): no blockers; four P2s fixed (runner
path normalization, `.\` local paths and `$/` Unsupported, case-insensitive built-ins, first-space
split instead of trimming). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

Literal `shell` text (run steps, workflow and job `defaults.run`) that is not a built-in shell
(`bash`, `pwsh`, `python`, `sh`, `cmd`, `powershell`) and contains no `{0}` placeholder is
`Invalid`: the runner's script handler rejects it (the B-8 semantic-rejection allowance). As in the
runner, the text is split at its first space into a command, matched case-insensitively, and
arguments, so `PowerShell` and `bash ` are accepted while ` bash` (empty command) is not. The backlog
rule is applied as written, so `bash -e` without `{0}` is also `Invalid`. Job `defaults.run.shell` expressions stay unevaluated and unchecked.

Literal step `uses` text that is not `./…` or `.\…`, `docker://<image>` or `owner/repo[/path]@ref`
(exactly one `@`, a nonempty ref, at least two path segments after splitting on `/` or `\` and
dropping empty ones, as the runner does) is `Invalid`. A `$/…` self-repository reference, which the
pinned parser recognizes, is `Unsupported`. The check runs after the
exactly-one-of-`run`/`uses` rule. Nothing is resolved, fetched or opened. Diagnostics escape the
text once with `escape_debug`. See the [CLI contract](../cli/README.md#job-timeouts-and-step-shells).

## Acceptance and expected red

`crates/wsr-gha/tests/shell_uses.rs`: category/route for `'true'`, `'5'`, `perl`, `bash -e` and `Bash`
and ` bash` at all three shell locations; built-ins (any case, trailing space) and `{0}` strings
accepted at all three; nine malformed and nine well-formed references; `$/action` Unsupported. One CLI test pins three exact escaped diagnostics with a no-execution
sentinel. Dogfood goldens unchanged. Expected red: quoted `'true'` shell and `'5'` uses inspect Ok.
