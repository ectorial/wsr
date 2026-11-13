# Dogfood acceptance iteration: `example-*.yml` golden inspection (backlog B-6)

Status: **delivered on merge of its PR**, branch `test/dogfood-inspection` from `de3e910` (Schedule A).
Implemented by Claude (claude-opus-5-5). Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

`crates/wsr/tests/dogfood.rs` reads the real `.github/workflows/example-python.yml` and
`example-shell.yml` and asserts the complete `wsr inspect` stdout (golden text in the test), empty
stderr and exit 0, for both the real path and a byte-identical copy beside a `wsr.toml` sentinel.
`PATH` is replaced by a directory whose `sh`, `bash`, `python`, `python3`, `git` and `pwsh` each
write a marker through `/usr/bin/touch`; the test asserts no marker, an unchanged sentinel directory
and unchanged real example files. No product code changes: B-3..B-5 delivered the behavior.

This is static translation-only acceptance. The native GitHub reference runs and the existing
native shell reference test are not WSR support; nothing here executes, binds, resolves or isolates.

## Evidence

- Expected red: none on current `main` (B-5 made both examples inspect). Red observed on the
  pre-B-5 build `85a1c30` in a temporary worktree: ``jobs.python.steps[1]: unknown field `with` ``.
- The marker command was checked to fire under the restricted `PATH`; a first draft used bare
  `touch`, which the restricted `PATH` could not find, so it would have passed vacuously.

Roadmap Gate 1a additionally requires the 500 KB warning (B-7).
