# Inspection iteration: owning-crate compiler tests (backlog B-2)

Status: **implemented, PR pending checks** on `test/gha-compiler-characterization` (Schedule A,
[backlog B-2](backlog.md), audit F6). Baseline `main`: `1545ab3`. Evidence lives in the
[checkpoint](inspection-progress.md).

## Outcome

`crates/wsr-gha/tests/compiler.rs` calls `wsr_gha::inspect` directly, so the GHA inspection
compiler's rules are tested in their owning crate rather than only through `wsr inspect` stderr.
It asserts summary structure, category and typed route, not host rendering:

- Graph: diamonds, repeated edges, forward references and `[]` accepted with declared order;
  missing target (`jobs.a.needs[2]`), self dependency (`jobs.a.needs`) and a three-job cycle
  (`jobs.c.needs[1]`, `a -> b -> c -> a`) rejected.
- Identifiers: invalid job IDs and an expression job ID (unsupported) are reported before an
  unknown dependency; reserved `__` and duplicate step IDs rejected at `steps[n].id`.
- Summary: a runner label list keeps order and repetitions; step IDs, names and run/uses kinds kept.
- Shapes: empty jobs, empty steps, empty `runs-on` list, run plus uses, mapping `needs`; a scalar
  step raised by the YAML decoder keeps no typed route (the B-1 gap).
- Hostile YAML: a 10^10-element alias expansion stops at serde_yaml's repetition limit; 10,000-deep
  nesting inside `needs` stops at its recursion limit; a deeply nested non-mapping root is rejected
  by shape. All fail as `Invalid` without panicking.

Expected red: none. This is characterization coverage and is reported as regression evidence,
not TDD red/green. No product code changed.
