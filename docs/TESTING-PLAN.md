# WSR validation suite implementation plan

Status: **foundation implemented; WSR inspection/execution/security cases remain planned**.

The implemented slice includes validated case catalogs, owned Python pass/failure references,
committed FastAPI/uv workflow and license snapshots, explicit pinned bare-source preparation,
and a Cargo integration test for the offline foundation. See [the corpus guide](../tests/README.md)
for actual commands and [native reference evidence](../tests/reference-runs.md) for observed runs.
Later phases below remain proposals, not implemented capabilities.

This plan defines how tests will guide WSR development. It does not approve unresolved product
architecture or change the design status in [PLAN.md](../PLAN.md). [ROADMAP.md](../ROADMAP.md)
owns delivery gates; [CHECKLIST.md](../CHECKLIST.md) records implemented behavior.

## Development entrypoint: Rust tests and manual workflows

Use ordinary Cargo unit tests in source files and integration tests in each owning
crate's `tests/` directory. [Testing WSR](../tests/README.md) gives the implemented
commands and layout. There is no mandatory case-ID or catalog-registration step.
The INS/EXE inventory below is supporting planning material, not a prescribed task
queue or a prerequisite for starting TDD.

The dogfood inputs are the actual manual-only `.github/workflows/example-*.yml` files.
As the GHA translator is implemented, use these same files in Rust tests asserting the
real translation to IR without executing commands. Once execution is implemented,
use them in WSR CLI/engine tests and compare meaningful outcomes against manual native
GitHub runs. Do not substitute a YAML parse test or a simulation for either product path.
Unresolved future runtime decisions do not block independent unit/translation work;
settle the narrow interface needed for each behavior before writing its acceptance test.

## 1. Goal and assumptions

Success means a developer can change WSR, run the relevant validation, and identify which documented
workflow behaviors still work, which regress, and which have never been implemented.

Assumptions for this proposal:

- Keep this work in the WSR repository; do not create separate example repositories initially.
- Use small owned example projects for routine development and pinned upstream repositories for
  external validation. No required symlinks, submodules, or full vendored repositories.
- Examples target Linux jobs first. Test execution from Linux and macOS hosts as backend support
  becomes available; do not silently translate native Windows/macOS jobs into Linux.
- Start with inspection, then component isolation, system isolation, and useful workflow execution,
  following the existing roadmap. This plan does not select a runtime or isolation backend.
- Keep existing CLI and tooling tests. Replace placeholder assertions only as the corresponding
  command becomes functional; retain coverage for the other placeholder paths.
- Never fetch repositories or run external workloads as an implicit side effect of ordinary tests.
- Develop each behavior through red-green-refactor, using the agent handoff contract in section 11.
  The inventory is a backlog of behaviors, not a requirement to write all future tests upfront.

## 2. Validation layers

| Layer | Inputs | Assertions | When required |
| --- | --- | --- | --- |
| Unit and CLI contract | Small values and temporary repositories | Parsing, diagnostics, exit codes, selection, isolated module behavior | Every PR |
| Workflow inspection | Committed YAML, scripts, event fixtures, expected semantic results | Discovery, source locations, dependencies, compatibility, expressions and matrix planning as implemented; no workload execution | Every PR after activation |
| Owned execution examples | Tiny committed projects copied to disposable directories | Actual outputs, failure attribution, job/step states and artifacts | Every relevant PR on a qualified backend after activation |
| Security and lifecycle | Bounded adversarial fixtures and synthetic credentials | Grants/denials, fail-closed behavior, cleanup, host/control-state protection | Every relevant backend/policy PR; full suite before release |
| External compatibility | Exact upstream commits and explicit invocation context | Selected real workflow behavior and declared unsupported requirements | Manual first; scheduled after stabilization; selected release gate |
| GitHub reference | Owned manually dispatched example workflows | Compare selected semantic outcomes against GitHub's execution | Manual and before promoting a compatibility claim |

An intentionally failing workload can produce a passing WSR test: the assertion is that WSR reports
its failure correctly. Conversely, a workload passing does not prove that WSR enforced isolation.

## 3. Repository layout and planned additions

```text
.github/workflows/
  example-shell.yml                 # Manual shell pass/failure reference
  example-python.yml                # Manual Python pass/failure reference
  example-job-outputs.yml            # Future example, added with that feature
  validate-wsr.yml                   # Future host-driven suite orchestration
examples/
  README.md
  python-basic/
    README.md
    pyproject.toml
    tests/test_example.py
  job-outputs/                       # Added when needed
  service-postgres/                  # Deferred until services are supported
tests/                              # Root test data
  README.md
  cases.toml                         # Small catalog of named cases
  fixtures/
    inspection/
    security/
    events/
  expected/                          # Reviewed semantic results/diagnostics
  ecosystem/
    cases.toml                       # Upstream revisions, selections, provenance
crates/wsr-gha/tests/
  workflow_examples.rs               # Manual-trigger guard/native shell reference
  translation.rs                     # Future real GHA-to-IR tests using example YAML
crates/wsr/tests/
  cli.rs                             # Existing executable contract
  corpus.rs                          # Implemented foundation/native-reference bridge
  inspection.rs                      # Future CLI inspection integration tests
  examples.rs                        # Future owned execution tests
  ecosystem.rs                       # Explicit opt-in external cases
  support/mod.rs                     # Shared helpers only when actually shared
```

Root `tests/` contains data and documentation. Cargo discovers integration test drivers in a
package's `tests/`, such as `crates/wsr/tests/`; root data is not automatically executed.
Resolve test data from `CARGO_MANIFEST_DIR`, not the caller's working directory.
Use a single root data directory; provider/backend unit tests can consume the same fixtures.

Put fetched sources, isolated workspaces and local results under ignored `target/wsr-tests/`.
Use immutable cached source snapshots plus a separate fresh writable copy per execution.
A developer may manually supply a local repository snapshot later; required cases always use the
recorded commit and must not mutate that developer's checkout.

## 4. Supporting case catalog and result contract

The existing small catalog supports native references and preserved inputs; it does not
replace Rust tests or require every new behavior/workflow to be registered. The implemented loader is
Python 3.11+ using standard-library TOML parsing, called by xtask and a Rust integration test.
The current schema covers IDs, status, kind, input, expectations and native mode; pinned source
metadata lives in the ecosystem catalog. Add the remaining fields below when their executable
lanes need them, rather than inventing unresolved runtime configuration now. Each eventual case records:

- Stable ID and purpose: the exact behavior being tested.
- Input project/workflow, event payload, selected job/matrix entry and required dependencies.
- Layer and capability requirements; host/job platform and backend requirements where applicable.
- Exact assertions: expected outcome, diagnostic code/source location, relevant job/step states,
  outputs/artifact contents, and absence of forbidden side effects.
- Trusted test grants separately from repository-controlled capability requests.
- Timeout, resource budget, tool/runtime requirements and allowed network requirements.
- Status: `planned`, `active`, or `required`, with the feature gate that promotes the case.
  `active` identifies the current test-first implementation slice; it is not passing coverage.
- For external material: repository URL, full commit SHA, workflow path, source license/notices,
  action references, dependency limitations and every modification made by the test.

Validate unknown fields, duplicate IDs, missing inputs and invalid expectations. A catalog entry
is inventory, not automatically a passing test. Each required case must have executable assertions;
report the IDs exercised and fail if a selected required case has no implementation or disappears.

Results distinguish passed, failed, planned/not run and infrastructure failure. Required cases
cannot succeed by skipping, accepting any nonzero exit code, or matching the current generic
"not implemented yet" message. Missing prerequisites in a selected required lane fail that lane.
The ordinary suite may exclude opt-in lanes, but must report that those lanes were not exercised.
Do not use ignored tests as evidence of coverage.

An active case must be selected explicitly and its real failure reported during development.
Its expected product failure never becomes a passing assertion or an allowed-failure CI gate.
Existing required cases remain enforced while the active slice is developed. Promote the active
case to required, and include its lane in the applicable gate, before calling the feature complete.
This permits a green supported baseline without pretending the planned backlog already works.

Assert semantics rather than entire unstable log streams. Normalize only incidental fields such
as temporary paths and timestamps; retain meaningful source locations, grants and state changes.
Expected results require review and must not be rewritten automatically when a test fails.

## 5. Initial case inventory

Add these incrementally with their corresponding phase, rather than creating empty directories
or a large collection of disabled test bodies upfront.

| IDs | Case | Initial validation and later promotion |
| --- | --- | --- |
| INS-001 | Valid manual-dispatch workflow | Discover job/steps and preserve source traceability |
| INS-002 | Invalid YAML and invalid workflow structure | Specific parse/validation diagnostics with source locations |
| INS-003 | Unknown action/platform/required feature | Explicit unsupported/unknown classification; block unsupported execution |
| INS-004 | Dependency cycle or missing dependency | Reject invalid plan |
| INS-005 | Inspection side-effect sentinel | Repository scripts do not execute; source and host sentinel remain unchanged |
| INS-006 | Two-value matrix and event/input conditions | Correct expansion/binding as those semantics become available |
| EXE-001 | Tiny Python project | One passing test; selected Python and dependency identities reported |
| EXE-002 | Same project with controlled assertion failure | Failed test maps to failed step/job and failing command exit |
| EXE-003 | Job output consumed by dependent job | Correct output propagation, ordering and failure/skip behavior |
| EXE-004 | Artifact produced and retrieved | Exact expected contents and bounded destination paths |
| SEC-001 | Minimal component with explicit grants | Permitted operation works; absent grant is denied |
| SEC-002 | System filesystem/network access | In-grant access works; outside access is blocked |
| SEC-003 | Policy and synthetic credential probes | Repository input cannot grant itself access or read unprovided host credentials |
| SEC-004 | Traversal/symlink/archive and shared-state probes | No escape or control-state modification across relevant host boundaries |
| LIFE-001 | Timeout/cancellation with a child process | Run ends within budget; descendants and temporary resources are cleaned up |
| BACKEND-001 | Required enforcement unavailable | Reject before workload side effects; no permissive fallback |
| EXE-005 | Tiny app plus PostgreSQL service | Connectivity, readiness, job scope and teardown; later milestone |

Security cases use bounded disposable resources and synthetic secrets. The test supervisor must
observe denials and cleanup from outside the workload boundary; do not trust workload-reported
success as proof of enforcement. Use positive controls so a missing test setup cannot look like a
successful denial. Never execute escape probes uncontained on a personal host.

## 6. Example workflow contract

Each owned project explains what it teaches, prerequisites, expected success/failure, current WSR
support, and how to invoke it once an actual product command exists. Do not invent stable CLI
flags or TOML configuration before those contracts are decided.

Reference workflows in `.github/workflows/example-*.yml` use `workflow_dispatch` only initially.
They execute the project's commands directly on GitHub. Local WSR tests consume those same workflow
files and projects. There is one canonical workflow per example; avoid maintained duplicate YAML.
Nested projects may contain only the additional files genuinely needed for repository discovery.

Use typed manual inputs for deterministic success/failure variations. Give intentionally failing
reference runs clear names and document their expected failed conclusion; do not hide failures with
`continue-on-error`. Keep examples free of publishing, deployment and real-secret requirements.
Pin action commits and relevant dependencies, and record image/runtime drift that pins cannot remove.

`workflow_dispatch` permits native GitHub execution; it does not make a workflow local-only.
Repository discovery rules must allow the WSR harness to select an example explicitly without
accidentally executing normal release workflows.

The orchestration workflow builds WSR and invokes the host-side test harness. Example payloads
must not invoke WSR's full test suite or recursively dispatch validation. Once engine reuse inside
Actions is supported, add a separate path that runs examples through WSR there. Keep the native
GitHub reference as an independent comparison.

Compare selected outcomes, not byte-identical logs: success/failure, skip propagation, matrix
membership, outputs and artifact contents. Security policy is WSR-specific and needs its own oracle.
A stored GitHub reference records source revision, inputs and run URL; it is not timeless proof.

## 7. External repository onboarding

Start with FastAPI for external inspection and uv for a later inspection/compatibility challenge.
The exact commits and runnable selections must be inspected and pinned during onboarding; do not
assume the default branch or entire workflow can execute. Defer Chromium until a specific relevant
workflow and affordable objective are identified. No external support claim is made by this plan.

For each upstream case:

1. Select a single valuable assertion and inspect its transitive workflow/action/script dependencies.
2. Record the exact source revision, workflow, event, job/matrix selection, tool identities and
   prerequisites. Pin referenced material where possible and disclose mutable external services.
3. Fetch only through an explicit preparation step, verify the requested commit, then create a
   fresh disposable working copy. Preserve relative paths and local actions/reusable workflows.
4. First assert inspection results, including unsupported parts. Isolated execution is a separate
   promotion with separate expectations; successful parsing does not establish runtime support.
5. When execution is viable, select a credential-free Linux case. Resolve prerequisite jobs or
   supply documented test inputs; do not silently bypass conditions or fake successful dependencies.
6. Run a deterministic pass/fail pair. For failure injection, record the exact patch and apply it
   only to the disposable copy. Report patched/derived cases separately from unmodified workflows.
7. Preserve upstream attribution and applicable license notices for copied workflow snapshots.
8. Review revision updates as changes to the corpus. Keep a pinned baseline; optionally test newer
   upstream revisions in an advisory lane without replacing the baseline automatically.

A network/setup failure is infrastructure failure, not WSR incompatibility or a successful test.
When WSR fully supports a selected case, a new failure is a regression until diagnosed.

## 8. Implementation sequence and acceptance gates

### Phase 0 — Supporting corpus foundation (implemented)

Deliver `tests/README.md`, the minimal catalog schema/loader, INS-001 through INS-005 behavioral
specifications, the first active inspection fixture, the Python example, and its manual native
GitHub workflow. Create remaining executable fixtures as their slices become active.
Keep unimplemented product cases marked planned; do not add fake green execution tests.
Use existing Rust dependencies/test helpers where sufficient; no new crate is required.
Use the section 11 handoff template for concrete behaviors; catalog IDs are optional.

Gate: catalog validation rejects malformed/duplicate/missing entries; existing tests remain intact;
the Python example's pass/fail variants work directly in a declared environment. Before treating
GitHub as a reference, execute both variants manually and record their actual run evidence.
A workflow file alone is not reference evidence.

### Phase 1 — Inspection becomes the first product gate

Drive GHA-to-IR translation through ordinary Rust tests on the canonical manual workflow files.
Assert the supported jobs/steps/trigger mapping and IR semantics without executing commands.
Add inspection CLI tests when that path is delivered; the INS inventory suggests boundary cases,
but is not a required sequence. Test discovery, provider failures, diagnostics and source mappings;
use deterministic local provider artifacts for normal tests rather than implicit downloads.
Provider bootstrap tests separately cover signature/digest rejection, pins, atomic installation,
offline cache hits and missing pinned artifacts using controlled sources.

Gate: required inspection cases pass offline without executing repository commands. Product stubs
fail these assertions. Provider/backend implementation decisions remain owned by PLAN.md.

### Phase 2 — Qualify execution boundaries

After the corresponding runtime/backend decisions, activate SEC-001 for components, then SEC-002
through SEC-004, LIFE-001 and BACKEND-001 for system execution. Qualify each host/backend separately.
Add a trusted outer test supervisor and bounded execution/cleanup checks before adversarial runs.

Gate: positive controls work, unauthorized probes fail, cancellation cleans up, and missing
enforcement blocks execution. A backend without this evidence is not a supported secure backend.

### Phase 3 — Make the Python example useful

Activate EXE-001 and EXE-002 through the complete provider/planner/executor path. This includes
support for every selected setup action, or a clearly labeled smaller shell-only workflow until
those actions are supported. Do not replace unsupported actions silently in the canonical example.
Implement the required matrix/input semantics and activate INS-006 as appropriate.

Gate: native GitHub reference and WSR agree on selected workload outcomes; WSR identifies the
failing step and test output, returns the correct exit status, and reports environment/grants.
Run on Linux and on macOS-hosted Linux once those backends are qualified.

### Phase 4 — Grow composition through examples

Activate EXE-003 and EXE-004 with job/expression/artifact support; add EXE-005 only with services.
For each feature add both normal behavior and a meaningful failure case, then require the case
in its relevant validation lane. Maintain WSR's ordinary CI as an independent test runner.

Gate: documented dependency, failure, skip, output and cleanup semantics hold across the supported
backends. Update CHECKLIST.md only with links to working tests and observed evidence.

### Phase 5 — External compatibility and release evidence

Onboard pinned FastAPI inspection, then selected execution when supported. Add uv inspection as
another independent compiler input; choose an executable uv case only after inspecting its needs.
Introduce explicit preparation and execution commands only when the first external case needs them;
keep them in existing xtask rather than creating another development CLI.

Gate: clean preparation reproduces the pinned case; pass/fail evidence is retained; required
external cases run in a scheduled lane and before releases that claim their compatibility.
Do not mark a whole repository supported because one job passes.

## 9. Developer and CI policy

Existing `cargo xtask test` and `cargo xtask ci` retain their roles. Ordinary Cargo tests include
fast deterministic unit/CLI/inspection cases as activated. Heavy/backend/external tests require
explicit selection and preparation; ordinary tests do not install tools or fetch external repos.
The foundation now provides `cargo xtask corpus`, `--list`, `--case ID` and `--prepare REPOSITORY`.
These commands validate native references/snapshot integrity or prepare sources; they do not run
WSR workflows. Exact execution-lane syntax remains deferred.

- Every PR: existing standard checks plus activated fast semantic cases.
- Backend/engine/policy PR: relevant required execution, lifecycle and security lanes on qualified
  Linux/macOS hosts. Once activated, missing prerequisites cannot convert these lanes to green.
- Scheduled/manual: full qualified backend matrix and pinned external corpus.
- Release: standard CI, applicable security/execution lanes, and the pinned compatibility cases
  supporting that release's published claims, all for the release candidate revision.
- Manual native references: run before promoting an example and whenever its asserted semantics
  change. Store the reference revision and run link with the evidence.

Use explicit lane selection as well as affected-path rules: shared engine/policy changes must not
escape validation just because example files were unchanged. Preserve current CI and add lanes
incrementally. Scheduled execution is configured during implementation, not by this proposal.

Persist case ID, WSR revision, input digest/upstream SHA, host/job platform, backend/runtime/provider
identities, effective grants, duration and outcome. Retain relevant logs/artifacts on failures with
synthetic secrets redacted. A small versioned JSON result format is sufficient; no dashboard needed.

## 10. Completion and claim rules

A feature is complete only when its behavior tests run, pass and are enforced by the appropriate
CI gate. Standard Cargo discovery is sufficient; catalog registration is optional. Planned fixtures, ignored tests, documentation and source compilation are not completion.
Unsupported behavior must receive a precise diagnostic and must never become an implicit success.

The corpus foundation is supporting tooling. Product progress is measured by real Rust
behavior tests, then translation and execution of the canonical manual workflows. The first compelling execution
demonstration is the tiny Python project's controlled failure, actionable diagnosis and repaired
success, using the same example workflow locally and as a native GitHub reference. Pinned external
cases then challenge assumptions that owned examples might miss.

## 11. TDD and agent implementation contract

### Ready slices

Each task is a small observable behavior with a descriptive Rust test name, not an entire subsystem.
A catalog case ID is optional; a normal unit or integration test is sufficient. A maintainer
or planning agent prepares the following handoff before assigning implementation:

```text
Behavior and test name (optional catalog ID):
Accepted decision/specification links:
Public interface and expected outcome:
Input fixture/event and relevant negative or boundary case:
Existing test/validation paths, or the first test to add:
Exact focused command and required environment/preparation:
Expected red failure and why it demonstrates the missing behavior:
Existing required regression commands:
Permitted scope and explicit non-goals:
Unresolved prerequisites, or "none":
Completion evidence and CI lane:
```

Fill commands from actual implemented tooling; a placeholder command is not an executable handoff.
A ready slice needs enough settled semantics to distinguish correct behavior from an arbitrary
implementation. If interface, policy or backend decisions are unresolved, report the specific
decision needed rather than inventing it to satisfy a test. Existing architecture gates remain
in force; tests cannot authorize an otherwise undecided design.

Tests and validation should be reusable task inputs. A test may be authored in a preparatory
change and handed to another agent with its observed red result. A pending test may remain on a
work branch, but must not break required main-branch CI. Either land the vertical slice together,
or keep the active case in an explicitly selected development lane until implementation lands.
Do not introduce ignored tests or expected-failure wrappers to simulate this distinction.

### Red-green-refactor loop

1. Read the handoff, relevant decisions, fixture and existing assertions. Run the focused baseline
   before changing production code. Existing tests are a starting specification, not proof that
   every requirement is covered.
2. Select one active behavior. Reuse its existing acceptance test or add one meaningful test
   through the public CLI/API. Observe failure for the intended missing behavior. Compilation,
   missing downloads, unavailable tools and harness errors are not sufficient behavioral red
   evidence; repair setup and rerun until the test reaches the intended product path.
3. Implement the smallest general behavior that passes. Do not special-case case IDs, fixture
   paths, repository names or expected output files in production code. Use deterministic doubles
   only at external boundaries; acceptance tests must exercise the real behavior they claim.
4. Run the focused test and existing relevant required cases. If green, refactor only within the
   authorized scope, then rerun. Add the next test only after this cycle is complete; do not write
   a whole subsystem's speculative test suite before implementing anything.
5. Before completion, run the specified regression lane and standard repository checks. Keep the
   new Rust test in the default Cargo suite or the relevant required execution lane. If the test
   also has a catalog entry, promote that entry to required. Record commands and outcomes.

If a supplied test already passes, verify that it reaches the intended path and covers the
acceptance behavior. Report that result; do not manufacture a red run or add unnecessary product
code. Identify an uncovered requirement only if the task calls for one. Distinguish new behavior
driven by TDD from already-working behavior confirmed by validation.

### Protect the acceptance contract

Agents must not delete/ignore required cases, relax assertions, broaden normalization, refresh
expected snapshots, downgrade case status or weaken CI selection merely to make a change green.
When an expectation is demonstrably wrong, report the conflicting specification and proposed
correction for review. An intentional behavior change needs an explicit specification update
and reviewed expectation changes; do not treat generated current output as the oracle.

Behavioral tests should survive internal refactoring. Test outcomes, state transitions and public
diagnostics rather than private method names, internal object layouts or call counts. Add focused
unit tests for complex rules where helpful, while retaining end-to-end assertions for product
claims. Add a second input or a negative case when needed to distinguish real behavior from a
hard-coded answer; do not require redundant tests merely to increase test counts.

When an external workflow exposes a bug, reproduce it in a small owned regression fixture first
where possible, observe red, fix it, then rerun both the fixture and the external case. Preserve
the original external case as evidence that the reduction still represents a real integration.

### Completion report

The implementing agent reports case IDs, specification links, the focused red command/result,
the green command/result, regression results, changed scope and any unexercised lanes. Include
WSR/input revisions and backend identities where relevant. Keep this evidence in the task/PR
record; durable compatibility evidence follows section 9. Do not commit transient logs as source.

The handoff and completion record must let another agent reproduce validation without chat
history. Passing the supplied tests is necessary; fulfilling the stated behavior and preserving
the existing contract are also required. Tests cannot establish untested compatibility or security.
