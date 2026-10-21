# Native GitHub reference evidence

Verified October 21, 2026 at source revision
`93f910f15376a450350ac43a0ae9d35d45be3fc6` (implementation PR #9).
Both manual dispatches used `.github/workflows/example-python.yml`, job `python`,
`ubuntu-latest`, and CPython **3.13.16** as reported by setup-python.

| Input `mode` | Observed outcome | Evidence |
| --- | --- | --- |
| `pass` | Success; `test_addition` passed, `Ran 1 test`, `OK` | [Native pass run](https://github.com/ectorial/wsr/actions/runs/39575704381) |
| `fail` | Expected failure; setup succeeded, unittest step exited 1 with `AssertionError: 4 != 5 : controlled WSR example assertion`, `Ran 1 test`, `FAILED (failures=1)` | [Native failure run](https://github.com/ectorial/wsr/actions/runs/39575707814) |

These are independent **native Python** reference runs, not WSR execution or isolation evidence.
The failed conclusion is intentional and belongs only to the manually dispatched failure variant;
ordinary corpus validation asserts that this workload fails correctly and remains green.
The workflow and inputs are pinned by the recorded source revision, but the hosted runner image
and selected Python minor version can change. Rerun and record a new reference when relevant
example semantics or setup change. WSR execution cases EXE-001 and EXE-002 remain planned.

Local implementation verification also passed `cargo xtask ci`, the existing tooling/scaffold
regressions and ten corpus harness regression tests. Explicit preparation fetched both cataloged
FastAPI/uv commits and verified their workflow/license snapshots without executing upstream code.
