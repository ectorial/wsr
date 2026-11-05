# TypeScript and JavaScript on macOS: Bun workflows through WSR

> **Design walkthrough, not a working quick start.** Reviewed 2026-11-01. WSR is currently a
> CLI/types scaffold. Help/version work; `init`, `list`, `status`, `inspect`, `run`, and cache
> operations still return “not implemented yet.” Every WSR log below is **simulated**, including
> success, cache hits, diagnostics, and paths. No timings or compatibility results were measured.

Read this like a screen-recorded tutorial: first see the workflow you recognize, then watch what
WSR would do on a Mac. **A is recommended for this example; B is an alternative.** Recommendations
are walkthrough assumptions, not new accepted decisions. Change them as implementation evidence arrives.

The [decision record](../../PLAN.md), [GHA contract](../../crates/wsr-gha/README.md), and
[implementation checklist](../../CHECKLIST.md) remain authoritative. The provider adapter compiles
GHA syntax; the core plans and supervises execution. Bun is selected for compatible JavaScript
actions, but none of the actions below has been qualified under WSR. Inspection must reject an
unqualified action rather than pretend that Node compatibility or wider permissions solves it.


## Shared lifecycle

This walkthrough uses the [architecture](../../ARCHITECTURE.md) and
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

“We have a TypeScript package on a Mac. Let's install from its lockfile, type-check, test, bundle,
and preview the publish contents in Linux.” Assume `package.json`, committed `bun.lock`, `tsconfig.json`,
`src/index.ts`, and `tests/`. The package includes TypeScript as a dev dependency, has a real registry
name/version, and explicitly selects published files. This is a Bun-tested package, not a claim of
compatibility with every Node runtime or native addon.

Example `package.json` fragment (merge with the project's real metadata/dependencies):

```json
{
  "type": "module",
  "files": ["dist"],
  "exports": "./dist/index.js",
  "scripts": {
    "typecheck": "tsc --noEmit",
    "test": "bun test",
    "build": "bun build ./src/index.ts --outdir ./dist --target=node"
  }
}
```

`bun run test` runs the package script, which invokes Bun's test runner here. `bun build` is the
bundler, while `bun run build` invokes the configured script. A Node-targeted bundle still needs
Node-runtime testing if Node support is advertised. Type checking is separate from bundling.
See the [Bun bundler](https://bun.sh/docs/bundler) and
[Bun CI guide](https://bun.sh/guides/runtime/cicd).

- **A — Recommended:** exact qualified Bun version plus committed lockfile and frozen installation.
- **B — Alternative:** floating Bun/dependency resolution for each run.

Use A. Supply `vars.BUN_VERSION` explicitly as a GHA-context input; its local binding UX is open.
An absent variable must be diagnosed instead of silently choosing the action's default version.

## 2. The GHA workflow

Reference input: [node.yml](node.yml). The inline workflow below matches that file.
Copy it to an example project's `.github/workflows/ci.yml` after supplying the project prerequisites
and reviewing action/runtime pins. Its location here does not register a workflow or prove WSR support:

```yaml
name: TypeScript CI
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
      - uses: oven-sh/setup-bun@v2
        with:
          bun-version: ${{ vars.BUN_VERSION }}
      - name: Locked install
        run: bun install --frozen-lockfile
      - name: Type check
        run: bun run typecheck
      - name: Test
        run: bun run test
      - name: Bundle
        run: bun run build
      - name: Preview package
        run: bun publish --dry-run
```

Action tags are readable examples, not qualified WSR artifacts. Pin reviewed action SHAs and record
runtime/image identities before using this as a validation fixture. The
[setup-bun action](https://github.com/oven-sh/setup-bun) installs the project runtime; its own action
execution under WSR is a separate compatibility test.

There are **two Bun roles**: the provider-selected compatibility runtime hosting qualified GHA
JavaScript actions, and the project runtime installed by setup-bun. They may need different versions.
Deduplicate only when exact artifact and compatibility identities match. Bun never executes the
provider compiler; the provider remains a signed Wasm component.

The frozen install command follows [Bun's install contract](https://bun.sh/docs/pm/cli/install).
Dependency lifecycle scripts, native addons, and downloads remain workload behavior under policy.
No `actions/cache` is implied by this YAML.

- **A — Recommended for this walkthrough:** proposed local package-download caching, with fresh installs.
- **B — Alternative:** add and qualify a GHA cache action/service for package-manager state.

Use A. WSR's immutable artifact store does not already implement B.

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

## 6. Run it and distinguish the two runtimes

```sh
wsr run .github/workflows/ci.yml --event workflow_dispatch --verbose
```

This is the proposed successful path after compatibility and isolation gates pass. IDs, versions
shown as placeholders, counts and messages are synthetic; no timing claim is made.

```text
SIMULATED VERBOSE RUN — TypeScript / cold package cache
[wsr run=node-001] source=committed-revision event=workflow_dispatch provider=gha
[wsr input] vars.BUN_VERSION=<explicit-project-version>; credentials absent
[wsr provider] signed component HIT; metadata compilation MISS -> canonical plan validated
[wsr plan] checkout/setup-bun qualified for action-host Bun@<compat-version>
[wsr plan] project Bun@<project-version> selected separately; target=linux/x86_64
[wsr preflight] known tool/script requirements validated; unsupported requirements block admission
[wsr runtime] project Bun Linux artifact MISS; approved acquisition/identity verification
[wsr policy] package-download access allowed for install; registry-write denied
[wsr vm] managed Linux ready; host ~/.npmrc and host node_modules not mounted
[wsr workspace] /workspace/node-001 created; private step scratch allocated separately
[wsr step=checkout] qualified Bun action + Git child; selected source materialized; exit=0
[wsr step=setup-bun] start action-host Bun@<compat-version> action=<resolved-sha>
[wsr state] activate prepared project runtime PATH; retain job-owned executable after action exits
[wsr cache] package namespace key=<lock+target+runtime+install-options+trust>; MISS
[wsr cache] job-local writable package cache prepared; no host cache write access
[wsr step=install] enforce admitted script/native requirements throughout execution
[wsr step=install] exec bun install --frozen-lockfile
[workload install] Locked dependencies installed
[wsr step=install] exit=0; execution remained within enforced policy
[wsr step=types] exec bun run typecheck
[workload types] $ tsc --noEmit
[wsr step=types] exit=0
[wsr step=test] exec bun run test
[workload test] $ bun test
[workload test] 24 pass; 0 fail
[wsr step=test] exit=0; tests executed in this run
[wsr step=build] exec bun run build
[workload build] $ bun build ./src/index.ts --outdir ./dist --target=node
[workload build] Bundle written to dist/index.js
[wsr step=preview] exec bun publish --dry-run
[wsr step=preview] packing/lifecycle code remains sandboxed; registry-write denied
[workload preview] Package contents inspected; no publication performed
[wsr lifecycle] complete registered action cleanup and validated output channels
[wsr cache] eligible package data staged; exclude node_modules, dist, tokens and logs
[wsr result] verify=passed; package preview recorded; registry uploads=0
[wsr cleanup] descendants stopped; private scratch destroyed; bounded results retained
```

The project in this example has no publishing lifecycle hooks. If it adds them, inspect and authorize
their execution requirements. `bun publish --dry-run` prevents publication but can still perform
packing/lifecycle work. In contrast, `wsr run --dry-run` must execute no workload code at all.
See the [Bun publish contract](https://bun.sh/docs/pm/cli/publish).

## 7. What becomes faster on the second run?

| Object | Proposed warm behavior |
| --- | --- |
| GHA provider and resolved actions | Reuse exact verified objects on the Mac |
| Canonical workflow | Reuse only with all relevant compilation inputs unchanged |
| Action-host Bun and project Bun | Reuse independently; deduplicate only identical qualified artifacts |
| Package downloads | Reuse eligible entries through a separate ecosystem-cache contract |
| `node_modules` | Reinstall for the Linux run; do not import the Mac's native addons/symlink layout |
| Tests and `dist/` | Execute/rebuild in this baseline; no WSR test-result cache |

- **A — Recommended:** cache package data and perform a fresh frozen install.
- **B — Alternative:** reuse a whole installed tree after checking platform/runtime/options/scripts.

Use A. Lockfile changes, OS/architecture, runtime, install options, patches, and lifecycle/native
build inputs affect correctness. A cache entry cannot authorize a script that policy previously denied.
Package caching is not a license to share private registry contents between unrelated trust domains.

For action delivery, A is interpreting the qualified action with pinned Bun; B is compiling it into
a standalone executable. Prefer A initially. Bundling your application with `bun build` does not
mean WSR has adopted B for actions.

## 8. Change TypeScript, rerun, and debug a failure

For local edits, **A — Recommended** is an explicit working-tree overlay after isolated checkout;
**B — Alternative** is committed-revision-only input. We assume A for this loop, but its selector,
inclusion rules and source identity are not yet implemented. Record the divergence from hosted
checkout behavior and never include private files by accident.

```text
SIMULATED WARM RUN — one TypeScript source file changed
[wsr source] explicit overlay selected; source digest changed; host checkout untouched
[wsr compile] HIT; workflow/action metadata unchanged; runtime grants reevaluated
[wsr cache] action-host Bun HIT; project Bun HIT; package download entry compatible
[wsr workspace] fresh node_modules install; no old test result accepted
[wsr step=types] exec bun run typecheck
[workload types] src/index.ts(12,3): error TS2322: Type 'string' is not assignable to type 'number'.
[wsr step=types] exit=2
[wsr result] test/build/preview skipped under normal success conditions
[wsr fallback] none; do not retry with Node or wider permissions
```

A cleaned warm VM is A for environment reuse; recreating it each time is B. Prefer A only after
cleanup tests, always with fresh mutable job state. Digest-based incremental source delivery is A;
full copying is B. Prefer A once source identity and deletion handling are proven.

For independent lint and test jobs, A is bounded concurrency with separate workspaces; B is serial
execution. Prefer A when resources permit. Download locking and atomic cache publication are needed;
never share a writable node_modules directory between jobs. Node-version matrix testing is additional
explicit system work, not an automatic fallback from a failed Bun action.

## 9. Publish deliberately

- **A — Recommended:** keep package preview in CI and put real publication in a separate release workflow.
- **B — Alternative:** add a strictly gated publishing job to the same workflow.

Use A. Once that release workflow exists, inspect its intended destination and credentials:

```sh
wsr inspect .github/workflows/release.yml
wsr run .github/workflows/release.yml --event workflow_dispatch --dry-run --verbose
```

The actual release operation could be `bun publish` with an explicitly configured registry and scoped
credential, or publishing an identified prebuilt tarball. It is an external write and is not executed
by this tutorial. Ensure package name/version, exported files, license, and intended access are correct.
A preview is not authorization to upload.

For release inputs, **A — Recommended** is publish the reviewed artifact with an explicit identity;
**B — Alternative** is rebuild while publishing and record the new identity. Prefer A. Bun's documented
prebuilt-tarball path differs from packing a directory, including lifecycle handling; validate the
chosen path rather than assuming identical behavior.

```text
SIMULATED RELEASE PREFLIGHT — no upload
[wsr release] artifact identity and registry selected
[wsr credentials] scoped registry credential not supplied
[wsr policy] registry-write denied -> BLOCKED before workload execution
[wsr result] package preview retained; no automatic login or host credential discovery
```

Credentials may be supplied through the supported registry environment/config mechanism, but never
copied from a personal host home implicitly or written into the shared cache. GHA token permissions,
local owner grants and registry authorization are separate. Interrupted publication needs outcome
inspection; WSR must not blindly replay it under a more permissive runtime.

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
