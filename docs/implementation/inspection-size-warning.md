# Size-warning iteration: GitHub 500 KB workflow-size warning (backlog B-7, D-032)

Status: **delivered on merge of its PR**, branch `feat/inspect-size-warning` from `07d1892` (Schedule A).
Implemented by Claude (claude-opus-5-5); cross-reviewed by Codex gpt-6.1-sol (high): no blockers;
P2 fixed (`eprintln!` panicked with exit 101 and no report when stderr was closed; red reproduced,
regression test added) and the 512,000-byte reading documented as WSR's interpretation. Evidence lives in the [checkpoint](inspection-progress.md).

## Scope

D-032: WSR imposes no workflow file-size limit. When the bytes read exceed GitHub's documented 500 KB
limit, read by WSR as 512,000 bytes (GitHub does not say whether KB means 1,000 or 1,024 bytes; the
backlog chose 1,024), a successful `wsr inspect` writes one warning line to stderr before the report
and still exits 0 with unchanged stdout:

```text
warning: wsr inspect: ci.yml: 512001 bytes exceeds GitHub's documented 500 KB (512000-byte) workflow file limit; GitHub would not run this workflow
```

The path is escaped like diagnostics. A failing inspection reports only its error. The warning is a
fallible write: a closed stderr is ignored and the report is still written. The warning is
about GitHub hosting; it is not a WSR limit or a compatibility claim.

## Acceptance and expected red

CLI test at 511,999, 512,000 and 512,001 bytes: exit 0, identical stdout, a warning only above the
limit. Expected red: no warning at 512,001 bytes. The closed-stdout test's 638,896-byte fixture now
expects exactly this warning instead of empty stderr (intended contract change).

With B-6 this completes roadmap Gate 1a.
