# Matrix literal size iteration (backlog B-32, audit F43)

Scope: the B-30 strategy guard in `crates/wsr-gha/src/strategy.rs` charged 8 bytes per number or
container against a fixed 512,000-byte budget, so literal matrices were rejected as alias
amplification. D-032 sets no size cap (only the GitHub 500 KB warning). The guard now bounds
alias amplification only: all matrices in one workflow share a budget of charge units equal to
twice the source size. A string is charged its decoded length (at least 1) and every other node,
including null and a tag, 1. Decoded string text is at most 1.5 times its source (`\L`/`\P`
escapes) and every node needs source bytes, so a literal matrix costs at most twice its source and
always fits; sharing the budget also bounds one large anchor reused by many jobs. The budget bounds
charge units, not rendered bytes (normalized YAML adds indentation and quoting). Matrices are still
never expanded.

Acceptance:

- `wsr-gha` and CLI: a 128,110-byte workflow whose matrix holds a nested sequence of 64,000 zeros
  inspects with exit 0; a matrix holding one 512,001-character literal inspects with exit 0 and
  the existing size warning on stderr only. One job, no expansion.
- The existing alias-amplified fixtures still fail as Unsupported at `jobs.a.strategy.matrix`
  without quoting the matrix, and one anchor reused across many jobs fails too.
- Review P2 (Codex): aliased null or empty-string items were free, so `a: &a [1000 nulls]` with
  100 `*a` references inspected (907 KB of normalized YAML from 5.4 KB). Every node now costs at
  least 1; red observed, then green.
- Contract change: the diagnostic now reads `matrix expands beyond twice the workflow source size
  through YAML aliases; not inspected yet` instead of naming 512,000 bytes.

Expected red: exit 1 `matrix expands beyond 512,000 bytes through YAML aliases; not inspected
yet` on the literal fixtures.
