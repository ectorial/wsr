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

## Shell dogfood workflow — October 25, 2026 (America/Denver)

Verified at source revision `2813537dd140f5e6d5f8d9e29ae07676e5f616b5`
(implementation PR #14), using `.github/workflows/example-shell.yml`, job `shell`,
`ubuntu-latest`, and `/usr/bin/sh -e`. Both runs were triggered manually.

| Input `mode` | Observed outcome | Evidence |
| --- | --- | --- |
| `pass` | Success; the step printed `WSR shell example passed` | [Native shell pass](https://github.com/ectorial/wsr/actions/runs/40261645085) |
| `fail` | Expected failure; the step printed `controlled WSR shell failure` and exited 1 | [Native shell failure](https://github.com/ectorial/wsr/actions/runs/40261647553) |

The Cargo integration test `workflow_examples` reads the script from this same YAML
and validates both native outcomes locally. Its manual-trigger guard also rejects
additional automatic triggers. The fail-mode GitHub conclusion is intentional;
ordinary Cargo tests assert that failure and stay green. These runs and tests do
not implement or establish GHA-to-IR translation, WSR execution, or isolation.
