# Testing WSR

## Rust tests drive implementation

Use Cargo's standard test layout; no custom test framework or case registry is required:

- Unit tests live beside the code in `src/*.rs`, inside `#[cfg(test)] mod tests`.
- Integration tests live in the owning package's `tests/*.rs`. For larger suites, a
  `tests/it.rs` entrypoint can declare modules under `tests/it/`; an arbitrary root
  `it/` directory is not automatically discovered by Cargo.
- Shared fixture data lives under root `tests/fixtures/`. Resolve it from
  `CARGO_MANIFEST_DIR` so tests work from any working directory.
- GHA translation tests belong in `crates/wsr-gha/tests/`; executable WSR tests
  belong in `crates/wsr/tests/`. Add tests to other owning crates as behaviors emerge.

For each behavior: write one meaningful failing test, observe the intended failure,
implement the smallest change, then refactor while keeping the tests green. Preserve
existing regression coverage. A case ID such as INS-001 is optional bookkeeping,
not an interface specification or permission to choose an unresolved design.
Do not prewrite an entire disabled suite or accept generic placeholder failures as success.

Current commands:

```sh
cargo test -p wsr-cli --locked
cargo test -p wsr --test cli --locked
cargo test -p wsr-gha --test workflow_examples --locked
cargo test --workspace --locked
cargo xtask ci
```

The workflow example tests validate manual-only triggers and execute the shell
example's exact script natively in pass/fail modes. They do not translate GHA to IR,
evaluate GHA expressions, or invoke WSR's executor.

## Manual workflows are the dogfood suite

The canonical examples live in `.github/workflows/example-*.yml`, each triggered only
by `workflow_dispatch`. Start with [the example guide](../examples/README.md).
Native GitHub dispatch runs those workflows directly. Future WSR tests must use the
same YAML files, rather than maintaining separate local copies.

Build coverage incrementally:

1. **Now:** Cargo unit/integration tests drive the currently implemented code. Native
   reference checks validate the examples independently. Manual GitHub runs provide
   hosted reference evidence; normal Cargo tests do not dispatch workflows.
2. **As GHA-to-IR translation is implemented:** add Rust integration tests calling
   the real translator on the canonical workflow files. Assert jobs, steps, inputs,
   trigger mapping and meaningful IR fields for the supported subset; include malformed
   or unsupported inputs. Translation tests do not execute workflow commands.
3. **As execution becomes available:** add executable integration tests invoking WSR
   on those same workflows with explicit manual-event inputs in disposable workspaces.
   Assert successful execution and controlled failure attribution/exit status. Compare
   meaningful outcomes with native GitHub runs. Qualify isolation separately.

Stages 2 and 3 are not implemented yet. Do not invent a translator API, stable CLI
invocation or expected IR schema here. Settle the narrow behavior when implementing
its test, following [the TDD contract](../docs/TESTING-PLAN.md#11-tdd-and-agent-implementation-contract).
A passing YAML/reference check is never evidence that WSR translated or ran a workflow.

## Supporting corpus checks

The existing catalog and upstream snapshots remain supporting validation material.
They are not the development entrypoint, a prerequisite for every contribution, or
an exhaustive specification. New Rust tests and manual examples need no catalog ID.

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

## Choosing the next behavior

Choose a small behavior from the agreed product direction and the example workflows.
Use an ordinary named Rust test as its acceptance check. Existing INS/EXE entries may
help identify edge cases, but they do not mandate development order. Resolve only the
interface or architecture decisions needed for that behavior; unrelated future runtime
choices need not block independent unit or translation tests.

Follow the [TDD handoff contract](../docs/TESTING-PLAN.md#11-tdd-and-agent-implementation-contract)
for exact commands, the intended red failure, scope and regression evidence.
