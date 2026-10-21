# WSR validation corpus

This is a validation **foundation**, not a working WSR workflow runner. Python 3.11+ and Git
are prerequisites. The owned Python example has no third-party dependencies.

## Run the supported baseline

```sh
cargo xtask corpus
cargo test -p wsr --test corpus --locked
cargo xtask ci
```

`corpus` validates the catalog and the SHA-256 digests of committed upstream snapshots, then runs
our owned Python example in fresh temporary directories for both modes. The failing workload must
exit 1 and identify the controlled assertion; zero tests, import errors and arbitrary failures are
not accepted. The Rust integration test runs the harness regression tests and baseline as part of
ordinary `cargo test`/xtask CI, including hosted CI. No network or upstream execution is involved.
The product CLI placeholder tests remain unchanged. Repository tooling tests are separate.

```sh
cargo xtask corpus --list
cargo xtask corpus --case REF-PY-FAIL
cargo xtask corpus --case INS-001
```

The last command intentionally exits nonzero: INS-001 is **planned/not run**, not a passing WSR
inspection test. `--list` is inventory only. Missing Python, invalid catalogs, missing required
baseline cases and changed snapshots fail validation. Runtime, isolation and external compatibility
are not exercised. Default reports explicitly list unexercised cases.

Local results are written to ignored `target/wsr-tests/results.json`. They are transient validation
results, not release compatibility evidence. Native GitHub reference evidence belongs in
[reference-runs.md](reference-runs.md).

## External sources: FastAPI and uv

[ecosystem/cases.toml](ecosystem/cases.toml) records full commit SHAs, upstream paths, purpose and
snapshot digests. The files under `ecosystem/fastapi/` and `ecosystem/uv/` are unmodified upstream
workflow/license snapshots. FastAPI is MIT; uv is available under MIT or Apache-2.0. Retain the
included licenses. Upstream names identify test sources, not endorsements or compatibility claims.

Their current purpose is to preserve real compiler inputs. INS cases remain pending until WSR has
an agreed inspection result/diagnostic contract. The snapshots do not contain the full source,
local actions, reusable workflows, scripts or dependencies needed for execution.

Explicitly prepare pinned source objects when needed:

```sh
cargo xtask corpus --prepare fastapi
cargo xtask corpus --prepare uv
```

Preparation fetches one pinned commit into a staged bare Git repository, verifies its identity and
compares upstream workflow/license bytes against committed digests before installation under
`target/wsr-tests/sources/`. It does not checkout files, run Git hooks, initialize submodules or
execute upstream code. A pre-existing destination is rejected; remove that exact disposable cache
explicitly if preparing again. A failed preparation does not install a partial source repository.
Git network/authentication errors are infrastructure failures, not compatibility results.

To update a pin, review the new workflow and license files at the exact commit, replace the
unmodified snapshots, update their hashes and commit SHA together, and run the suite plus explicit
preparation. Never regenerate expected behavior from current WSR output. Job/expression/platform
semantics and dependencies require review before promoting an external case to execution.

## First ready implementation slice

The next product slice is **INS-001**: inspect the minimal manual-dispatch workflow without
execution. Its behavioral specification is in [cases.toml](cases.toml). The fixture already exists;
the diagnostic/output API and provider-host prerequisites remain architecture decisions in
[PLAN.md](../PLAN.md). This task is not ready for autonomous product implementation until those
prerequisites and its executable assertions are settled.

Agents can already implement additional **foundation** cases from this example handoff:

```text
Behavior and case ID: reject an invalid catalog input (add a focused regression)
Specification: docs/TESTING-PLAN.md sections 4 and 11
Public interface: scripts/corpus.py exits nonzero with an actionable validation error
Input: temporary malformed catalog created by the test; never edit required baseline data
Existing tests: scripts/test-corpus.py
Focused command: python3 scripts/test-corpus.py CatalogTests.<new_test_name>
Expected red: validator accepts the invalid input; assertion fails
Regression commands: python3 scripts/test-corpus.py; cargo xtask ci
Scope: catalog validation only; no WSR runtime, downloads or architecture choices
Unresolved prerequisites: none after a concrete invalid-input rule is selected from the specification
Completion: observed intended red, green, regression results and CI evidence in task/PR
```

Replace the illustrative test name with an actual focused command before handing off a concrete
task. Add one behavior/test, observe red, implement, then move to the next. Do not write speculative
acceptance tests for unsettled interfaces or weaken existing expectations to satisfy a feature.
Follow the complete [TDD handoff contract](../docs/TESTING-PLAN.md#11-tdd-and-agent-implementation-contract).
