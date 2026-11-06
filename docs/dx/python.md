# Python on macOS: uv, pytest and packaging through WSR

> **Design walkthrough, not a working quick start.** Reviewed 2026-11-01. WSR is currently a
> CLI/types scaffold. Help/version work; `init`, `list`, `status`, `inspect`, `run`, and cache
> operations still return “not implemented yet.” Every WSR log below is **simulated**, including
> success, cache hits, diagnostics, and paths. No timings or compatibility results were measured.

Read this like a screen-recorded tutorial: first see the workflow you recognize, then watch what
WSR would do on a Mac. **A is recommended for this example; B is an alternative.** Recommendations
are walkthrough assumptions, not new accepted decisions. Change them as implementation evidence arrives.

The [decision record](../context/decisions.md), [GHA contract](../provider/gha/README.md), and
[implementation checklist](../context/status.md) remain authoritative. The provider adapter compiles
GHA syntax; the core plans and supervises execution. Bun is selected for compatible JavaScript
actions, but none of the actions below has been qualified under WSR. Inspection must reject an
unqualified action rather than pretend that Node compatibility or wider permissions solves it.


## Shared lifecycle

This walkthrough uses the [architecture](../context/architecture.md) and
[provisioning contract](../provisioning/README.md). Tool preflight resolves/verifies packages;
setup actions still control activation and effects at their original step positions. Normal
YAML edits and approved downloads produce progress. Actionable reports concern blocked execution.
Cached exact inputs support disconnected runs when workload services permit it; jobs execute again.

```mermaid
flowchart LR
    W[Workflow] --> P[Preflight and verified tools]
    P --> E[Schedule and execute]
    E --> O[Local outputs]
    O -.-> R[Optional reporting]
```

The profiles, metadata file layout, daemon, cache CLI and attestation schemas remain proposals.

The adjacent YAML requests ubuntu-latest, whose target is Linux x64. The simulated traces preserve
that target. On an ARM64 Mac, execution needs an explicitly qualified x64 mechanism or an explicitly
selected ARM64 workflow/profile adaptation. No emulation or backend is selected by this walkthrough.
Backend/platform selection and image/tool identities in simulated logs require separate decisions.

## 1. Meet the repository

“We have a Python library on a Mac. Let's format-check, lint, type-check, test, and build it
inside Linux, then inspect a separate publish path.” Assume `pyproject.toml` defines a build backend,
`uv.lock` is committed, `.python-version` selects Python 3.13, and the dev group includes pytest and
Ruff. Source lives under `src/weather_api/`, with tests under `tests/`.

The researched CLI spells formatting `uv format --check`; this guide does not assume `uv fmt` is
an alias. `uv check` currently performs type checking through ty, so Ruff linting remains separate.
Tool versions and tool acquisition must be pinned/prepared for reproducible, offline-capable runs.
See the [uv CLI reference](https://docs.astral.sh/uv/reference/cli/).

- **A — Recommended:** a locked project and exact qualified uv/tool identities.
- **B — Alternative:** resolve floating tool/dependency versions on each run.

Use A. The YAML uses `vars.UV_VERSION`: supply an exact version supporting the illustrated commands
as an explicit GHA-context input; WSR's variables-input UX is still open. Do not let an empty value
silently select a moving latest release. Tool identities for `uv format`/`uv check` may be managed
separately from the project lock and need explicit recording.

## 2. A Python workflow you can recognize

Reference input: [python.yml](python.yml). The inline workflow below matches that file.
Copy it to an example project's `.github/workflows/ci.yml` after supplying the project prerequisites
and reviewing action/runtime pins. Its location here does not register a workflow or prove WSR support:

```yaml
name: Python CI
on:
  workflow_dispatch: {}
permissions:
  contents: read
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
      - uses: astral-sh/setup-uv@v10
        with:
          version: ${{ vars.UV_VERSION }}
          python-version: '3.13'
          enable-cache: false
      - name: Prepare project
        run: uv sync --locked --all-extras --dev
      - name: Formatting
        run: uv format --check
      - name: Lint
        run: uv run --no-sync ruff check .
      - name: Type check
        run: uv check
      - name: Test
        run: uv run --no-sync pytest --junitxml=reports/pytest.xml
      - name: Build distributions
        run: uv build
```

Readable action tags are examples; replace them with reviewed commit SHAs in actual validation
fixtures. Record exact Python, uv, formatter/type-checker, action, and runner image identities.
`--no-sync` assumes the preceding sync succeeded and avoids repeating environment synchronization;
it is not a replacement for checking the lockfile. Build-backend dependencies may need separate
acquisition. The [uv GitHub Actions guide](https://docs.astral.sh/uv/guides/integration/github/) and
[setup-uv inputs](https://github.com/astral-sh/setup-uv) describe the upstream setup pattern.

- **A — Recommended for this walkthrough:** disable setup-uv's GitHub-service cache and use a
  separately designed local uv-cache integration.
- **B — Alternative:** qualify the action's cache service/protocol, including post-job behavior.

Use A (`enable-cache: false`). An existing workflow enabling remote caching must be diagnosed,
not silently rewritten. setup-uv running through Bun is a compatibility hypothesis to test; Python,
uv, Ruff, ty and package build scripts execute in the isolated system backend, not in Wasm.

## 3. Initialize and explore from macOS

“Let's find the workflows before starting a Linux workload.” Assume the future WSR executable is
already installed; there is no selected installer command in this guide.

```sh
wsr --version
wsr --help
wsr init
wsr list
wsr status
wsr cache list
```

```text
SIMULATED WSR OUTPUT — initialization and discovery
[init] repository root located; existing configuration and pins preserved
[init] provider=gha; destination=local; configuration=wsr.toml
[provider] pinned component: cache MISS -> stage -> verify identity/signature -> atomic install
[list] .github/workflows/ci.yml; .github/workflows/release.yml (if separately added)
[status] execution: not started; grant policy: owner-managed; runtime preparation: pending
[cache] shared macOS store: provider available; Linux toolchain artifacts not prepared
```

Exact TOML keys, pin files, host cache directory, and status fields are open. A provider cache hit
reuses the exact verified artifact across repositories without downloading or compiling a Rust crate.

- **A — Recommended:** initialize the provider now and prepare execution artifacts only when needed.
- **B — Alternative:** provision the whole execution environment during initialization.

We use A so browsing a repository does not require installing every possible toolchain.

## 4. Inspect, then explain this invocation

```sh
wsr inspect .github/workflows/ci.yml
wsr run .github/workflows/ci.yml --event workflow_dispatch --dry-run --verbose
```

For this example, `inspect` explains syntax/compatibility and `run --dry-run` explains invocation
inputs, grants, and missing execution prerequisites. That division is proposed UX; both commands
are placeholders today. Explicit event payload/context binding remains open; the simple manual
workflow below needs no custom dispatch inputs. Selecting an event does not fabricate credentials.

The host passes source and resolved metadata to the signed provider component. Compilation preserves
source locations and lowers expressions; it cannot run installers, package scripts, or action code.
The core then selects backends and authorizes requirements before workload side effects. The provider
itself uses Wasm; this does not turn the JVM, Python, or JavaScript application into a Wasm module.

- **A — Recommended:** reuse canonical compilation only when all relevant input identities match.
- **B — Alternative:** compile on every inspection/run.

We use A as a candidate optimization. Keys need source/metadata, provider/compiler identity,
configuration, target, and any policy inputs actually used. Runtime conditions and outputs are
still evaluated at their proper phases; cached IR is not cached authorization or cached success.

- **A — Recommended:** use an owner-managed narrow policy, with missing grants explained before running.
- **B — Alternative:** approve a bounded set of grants interactively for each invocation.

We use A. Repository configuration only requests capabilities. `--yes` never grants access.

## 5. Follow the cache from your Mac into Linux

```text
macOS: source checkout + WSR control state
       |
       +-- per-user immutable store: providers / actions / runtime artifacts / eligible derived objects
       +-- proposed ecosystem cache namespace: package/tool data, separate trust and retention rules
                    |
                    | controlled delivery of required objects
                    v
managed Linux environment
       +-- fresh job workspace and job-owned tool installations
       +-- per-step filesystem views, environment updates, private scratch, and processes
```

The shared per-user content store is accepted design. The ecosystem-cache integration and host/guest
bridge in this tutorial are **additional proposals**. They must not allow workloads to publish arbitrary
bytes as verified WSR artifacts. Credentials, mutable run state, and unredacted logs stay out of the
shared immutable store. Private package content needs its own access/trust policy.

- **A — Recommended:** transfer only required objects through a controlled host/guest bridge.
- **B — Alternative:** expose a restricted read-only host cache mount.

Use A. Linux artifacts may be stored on macOS, but platform identity remains Linux. We assume a
qualified Linux/ARM64 image on an Apple Silicon Mac for the transcript, not automatic equivalence to
GitHub's Ubuntu image. Choosing Linux/AMD64 with emulation is B for platform selection; A is matching
the host architecture when the project permits it. An explicit AMD64 requirement must not be rewritten.

- **A — Recommended:** create a run-owned Linux workspace from the selected repository source.
- **B — Alternative:** mount your writable host checkout into the workload.

Use A. For the baseline run, source is a committed revision: the qualified checkout action must
materialize that revision into the isolated workspace using the required Git/tool/context contract.
It cannot erase uncommitted changes in your personal checkout. No checkout action is silently skipped.
For the edit loop later, local working-tree overlay is a separate explicit proposal.

## 6. Run the full Python job

```sh
wsr run .github/workflows/ci.yml --event workflow_dispatch --verbose
```

All prerequisites below are assumed future implementations. Counts, paths, IDs and child-tool
messages are illustrative; this is not a captured uv/pytest transcript.

```text
SIMULATED VERBOSE RUN — Python / cold dependency cache
[wsr run=python-001] source=committed-revision event=workflow_dispatch provider=gha
[wsr input] vars.UV_VERSION=<explicit-qualified-version>; no implicit GitHub variables/secrets
[wsr provider] signed component HIT; protocol verified
[wsr compile] MISS -> resolve metadata -> lower -> validate; source locations retained
[wsr plan] job=verify target=linux/x86_64 image=<pinned-image>
[wsr plan] checkout/setup-uv: qualified Bun actions; project tools: isolated system processes
[wsr preflight] Linux uv/Python and formatter/type-checker tool identities verified/prepared
[wsr policy] sync/build dependency endpoints allowed as declared; registry-write denied
[wsr vm] Linux environment ready; host home and personal .venv not mounted
[wsr workspace] /workspace/python-001 created; private scratch separated from shared job files
[wsr step=checkout] qualified Bun + Git child materialize selected revision; exit=0
[wsr step=setup-uv] action=<resolved-sha> compatibility-runtime=Bun@<qualified-version>
[wsr runtime] expose previously prepared Linux uv/Python artifacts
[wsr state] validate PATH/UV_PYTHON updates; tool files retained by supervisor
[wsr cache] namespace=uv-packages key=<lock+target+python+tool+build-inputs>; MISS
[wsr cache] create job-local uv cache view; no writable host cache mounted
[wsr step=sync] exec uv sync --locked --all-extras --dev
[workload sync] Python selected; fresh .venv created; locked dependencies installed
[wsr step=sync] exit=0; .venv persists inside this job only
[wsr tools] activate/use preflight-admitted formatter/type-checker identities
[wsr step=format] exec uv format --check
[workload format] Formatting check passed
[wsr step=format] exit=0; source files unchanged
[wsr step=lint] exec uv run --no-sync ruff check .
[workload lint] All checks passed
[wsr step=types] exec uv check
[workload types] Type checking passed
[wsr step=test] exec uv run --no-sync pytest --junitxml=reports/pytest.xml
[workload test] collected 36 items
[workload test] 36 passed; report written to reports/pytest.xml
[wsr step=test] exit=0; test result was executed, not restored from cache
[wsr step=build] exec uv build
[workload build] built dist/weather_api-0.1.0.tar.gz
[workload build] built dist/weather_api-0.1.0-py3-none-any.whl
[wsr lifecycle] finish registered cleanup; setup action remote-cache path disabled explicitly
[wsr cache] stage eligible package data; exclude .venv, secrets, reports and mutable run state
[wsr result] verify=passed; dist/ identities recorded; PyPI upload=none
[wsr cleanup] terminate descendants; remove private scratch; bounded results retained
```

Package acquisition, formatter/type-checker provisioning, and build-backend resolution can require
network access. No “tests have no network” promise is inferred from a warm package cache; grants are
assigned to each step and runtime acquisition requirements must be explicit.

## 7. What is actually cached?

| Data | Example treatment |
| --- | --- |
| Provider/action/uv/Python artifacts | Identified immutable WSR objects, with platform and trust checks |
| uv package downloads and eligible built wheels | Proposed ecosystem namespace with package integrity and build-input rules |
| `.venv` | Fresh per run, shared only among this job's steps |
| `dist/` and pytest reports | Run outputs; not reusable dependency-cache entries or automatically uploaded artifacts |
| Tokens/private index credentials | Explicit step input; never reusable public cache content |

- **A — Recommended:** reuse package artifacts and rebuild the environment in Linux.
- **B — Alternative:** reuse a whole virtual environment after validating every compatibility input.

Use A. A macOS `.venv` or platform-specific wheel is not a Linux environment. An apparently portable
wheel still needs package integrity and Python compatibility checks. Editable project source and build
backend inputs affect reuse; matching `uv.lock` alone does not justify every cached build product.

For test dependencies, A is a single successful locked sync followed by no-sync commands; B is
allowing each command to synchronize independently. Prefer A here. For format/type tools, A is
recorded versions with explicit preparation; B is floating on-demand downloads. Prefer A. Do not
claim the project lock automatically pins tools launched by separate convenience commands.

## 8. Edit locally and observe a warm failure

For this edit loop, **A — Recommended** is an explicitly selected working-tree overlay after the
isolated checkout; **B — Alternative** is committed-revision-only execution. We assume A, with
inclusion/deletion rules and selection syntax still open. It must not silently change hosted checkout
semantics or expose ignored secrets. After explicitly selecting that proposed overlay mode, change
a test and rerun the workflow. The unchanged `wsr run` command alone does not select the overlay.

```text
SIMULATED WARM RUN — one test changed
[wsr source] explicit local overlay captured; personal checkout remains writable only by you
[wsr compile] HIT; workflow and action inputs unchanged
[wsr cache] provider/action/runtime HIT; uv package inputs compatible -> restore
[wsr workspace] fresh .venv materialized; no previous test success restored
[wsr step=test] exec uv run --no-sync pytest --junitxml=reports/pytest.xml
[workload test] FAILED tests/test_weather.py::test_freezing_point
[workload test] AssertionError: expected 0, got 32
[wsr step=test] exit=1; build skipped under ordinary success condition
[wsr result] reports retained as allowed; publish absent; no fallback or permission escalation
```

Changing `uv.lock`, Python/ABI, platform, build inputs or relevant tooling changes reuse eligibility.
A cache hit reduces preparation; pytest still runs. A proposed source-object store could transfer
only changed file content while creating fresh workspaces: A is digest-based materialization, B is
full copying. Prefer A after correctness tests, not filename/mtime-only reuse.

For repeated runs, A is a cleaned warm VM and B is a new VM per run. Prefer A only when isolation
and cleanup are proven. For Python 3.12/3.13 matrix coverage, A is bounded parallel jobs with separate
workspaces/cache compatibility checks, B is serial jobs. Prefer A when resources permit; no shared
mutable `.venv`. Locks and atomic publication coordinate simultaneous package acquisitions.

## 9. Build now; publish only through an explicit release path

- **A — Recommended:** CI produces distributions; a separate release workflow handles publication.
- **B — Alternative:** add a publishing job with an explicit release condition to the CI workflow.

Use A. Add a release workflow only when its artifact and credential contract is defined, then:

```sh
wsr inspect .github/workflows/release.yml
wsr run .github/workflows/release.yml --event workflow_dispatch --dry-run --verbose
```

Its workload would build/review distributions and invoke `uv publish` for the explicitly selected
index. That is a real upload, not part of this tutorial's local run. `uv build` produces wheel/sdist;
use the [uv publishing guide](https://docs.astral.sh/uv/guides/package/) to configure the destination.
A smoke test against the built wheel in a separate clean environment is recommended before release;
testing editable source alone does not establish that the wheel includes the required files.

For authentication, **A — Recommended locally** is an explicitly supplied scoped credential to a
chosen test/private index; **B — Alternative** is trusted publishing in an actually supported hosted
identity environment. We assume A for a future local release. A Mac running WSR does not acquire
GitHub OIDC identity because YAML says `id-token: write`. Hosted identity integration remains open.

```text
SIMULATED RELEASE PREFLIGHT — no publication
[wsr release] selected distribution identities and destination required
[wsr identity] GitHub-hosted OIDC unavailable in this local example
[wsr policy] no supplied credential or upload grant -> BLOCKED before publish
[wsr result] CI artifacts remain available; no retry with broader access
```

Do not store tokens in `wsr.toml`, shell examples, logs or shared package caches. A publishing grant
does not change package versioning or prove that an interrupted upload is safe to replay.

## 10. Check the cache, recover, and try offline work

```sh
wsr status
wsr cache list
wsr cache verify
wsr cache purge --help
```

```text
SIMULATED WSR OUTPUT — proposed maintenance presentation
[status] last workload run: finished, outcome=failed (edit-loop example); descendants stopped
[status] private scratch removed; failure evidence retained according to policy
[cache] provider/action/runtime objects retained by repository pins
[cache] ecosystem entries: separate namespace; target and dependency identities recorded
[verify] integrity check completed; this does not certify action behavior or build correctness
```

- **A — Recommended:** garbage-collect unreferenced objects while explaining retention roots.
- **B — Alternative:** explicitly delete a broader set and reacquire the same pinned identities later.

Use A for routine cleanup. `cache purge` exists only as a stub; its deletion scope and confirmation
contract are undecided. This guide does not invent `cache gc`, `repair`, `logs`, or `retry` commands.

D-027 requires corrupt objects to be quarantined and the same pinned identity reacquired when
authorized. If acquisition is unavailable or denied, report the blocked requirement. Exact repair
commands remain open. No repair silently changes a pinned version.

For offline work, A is preflight rejection of statically known missing objects; B is discovering
misses during execution. Prefer A. A cached provider alone does not make the job offline-capable:
toolchains, packages, build plugins, action dependencies, and any dynamic downloads must also be
available. No WSR offline flag is specified yet. Publishing is outside the offline path.

## 11. Keep this walkthrough honest as WSR grows

| Boundary | Evidence needed before replacing simulated output |
| --- | --- |
| Source/provider | Real compilation, provenance, action metadata resolution, and safe checkout semantics |
| Bun compatibility | Qualified action revisions and APIs, pre/main/post handling, command channels and descendants |
| macOS/Linux | Enforced isolation, path mapping, source capture, tool identity, cleanup and resource limits |
| Cache | Correct invalidation, trust namespaces, concurrency/atomicity, integrity and pin-preserving recovery |
| State | Workspace/tool persistence, environment/PATH/output timing, failure writes and secret/private-state separation |
| Release | Explicit destination, artifact identity, authorization, credentials, and publishing error behavior |

When implementing a boundary, update A/B choices with the decision and reason, link the owning
contract and behavior test, then replace only the verified portion of the transcript with captured,
redacted output. Keep command/output examples aligned. Mark a recommendation superseded when needed;
do not preserve it for compatibility with a pre-alpha tutorial. Never turn synthetic logs into evidence.
