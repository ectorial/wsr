# Shell and action-reference iteration: semantic validation (backlog B-14)

Status: **delivered on merge of its PR**, branch `feat/b14-shell-uses-validation` from `51bb95d`
(Schedule A). Implemented by Claude (claude-opus-5-5, orchestrator, burn mode; mechanical slice, no
`wsr-types` change). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

Literal `shell` text (run steps, workflow and job `defaults.run`) that is not a built-in shell
(`bash`, `pwsh`, `python`, `sh`, `cmd`, `powershell`, compared after trimming, case-sensitive) and
contains no `{0}` placeholder is `Invalid`: the runner's script handler rejects it (the B-8
semantic-rejection allowance). The backlog rule is applied as written, so `bash -e` without `{0}` is
also `Invalid`. Job `defaults.run.shell` expressions stay unevaluated and unchecked.

Literal step `uses` text that is not `./…`, `docker://<image>` or `owner/repo[/path]@ref` (exactly
one `@`, a nonempty ref, at least two nonempty path segments) is `Invalid`. The check runs after the
exactly-one-of-`run`/`uses` rule. Nothing is resolved, fetched or opened. Diagnostics escape the
text once with `escape_debug`. See the [CLI contract](../cli/README.md#job-timeouts-and-step-shells).

## Acceptance and expected red

`crates/wsr-gha/tests/shell_uses.rs`: category/route for `'true'`, `'5'`, `perl`, `bash -e` and `Bash`
at all three shell locations; built-ins and `{0}` strings accepted at all three; ten malformed and six
well-formed references. One CLI test pins three exact escaped diagnostics with a no-execution
sentinel. Dogfood goldens unchanged. Expected red: quoted `'true'` shell and `'5'` uses inspect Ok.
