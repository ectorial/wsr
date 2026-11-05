# Java on macOS: Maven and Gradle through WSR

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
Backend/platform selection and image/tool identities in simulated logs require separate decisions.

## 1. Meet the repository

“We have a Java library on a Mac. Let's run its Linux checks, package a JAR, and see what a warm
run reuses.” The Maven version of the repository has `pom.xml`, `mvnw`, `.mvn/wrapper/`,
`src/main/java/`, and `src/test/java/`. It commits the wrapper and pins plugin/dependency versions.
The POM binds unit tests and, if used, integration tests and quality plugins into `verify`.
A Gradle variant follows below; a project need not adopt both build tools.

- **A — Recommended:** execute the repository's reviewed Maven or Gradle wrapper.
- **B — Alternative:** use a globally installed build-tool version from the runner image.

We use A. The wrapper distribution/checksum and JDK become part of execution identity. Running a
wrapper is workload execution, never inspection. Maven `verify` includes earlier lifecycle phases;
it does not mean every project automatically has integration tests or a formatter configured.
See the [Maven lifecycle](https://maven.apache.org/guides/introduction/introduction-to-the-lifecycle.html).

## 2. The familiar GHA workflow

Reference input: [java.yml](java.yml). The inline workflow below matches that file.
Copy it to an example project's `.github/workflows/ci.yml` after supplying the project prerequisites
and reviewing action/runtime pins. Its location here does not register a workflow or prove WSR support:

```yaml
name: Java CI
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
      - uses: actions/setup-java@v6
        with:
          distribution: temurin
          java-version: '21'
      - name: Verify and package
        run: ./mvnw --batch-mode --no-transfer-progress verify
```

The readable action tags above are upstream examples, not WSR-qualified pins. A real fixture should
pin reviewed action commit SHAs and record exact resolved JDK/build-tool/image identities. Java 21
is an example project target, not a promise about the host's installed Java. No registry credentials
are needed for public-dependency CI; private repositories require separately supplied credentials.

`setup-java` installs/selects a JDK and updates the job environment. Its JavaScript implementation
would run through Bun only after qualification; the JVM itself runs in the isolated system backend.
See [setup-java](https://github.com/actions/setup-java).

We omit its `cache: maven` input in this baseline deliberately:

- **A — Recommended for this walkthrough:** a separately designed WSR-managed local dependency cache,
  with the GHA action's remote cache integration disabled/not enabled.
- **B — Alternative:** qualify the action's GitHub cache-service behavior or explicitly provide a
  compatible local service with correct scopes, keys, lifecycle and diagnostics.

Use A. `cache: maven` on an existing workflow cannot silently become A; inspect must explain the
unsupported service requirement until B exists or the user explicitly changes the workflow.

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

## 6. Run the Maven job, with the boundaries visible

```sh
wsr run .github/workflows/ci.yml --event workflow_dispatch --verbose
```

This is a future successful path **assuming all listed contracts have been implemented and tested**.
Run IDs, names and counts are invented. Prefixes distinguish WSR events from child-process output;
the log format is proposed, not a stable schema.

```text
SIMULATED VERBOSE RUN — Java / cold ecosystem cache
[wsr run=java-001] source=committed-revision event=workflow_dispatch provider=gha
[wsr provider] signed component HIT; identity and protocol accepted
[wsr compile] workflow+action metadata MISS; lower -> validate -> record canonical input identity
[wsr plan] job=verify requested-os=linux target=linux/arm64 image=<pinned-image>
[wsr plan] checkout,setup-java: qualified Bun action paths; verify: isolated system command
[wsr policy] registry-write=denied host-home=denied; network grants scoped per step
[wsr runtime] Linux JDK21 artifact MISS; acquire/verify using approved tool provisioning
[wsr vm] managed Linux environment ready; no personal host home or control socket mounted
[wsr workspace] create /workspace/java-001; private step scratch separate
[wsr step=checkout] start runtime=Bun@<qualified-version> action=<resolved-checkout-sha>
[wsr step=checkout] Git child allowed; materialize selected revision; credentials not persisted
[wsr step=checkout] exit=0; private descendants/scratch cleaned
[wsr cache] namespace=maven-dependencies target=linux/arm64 identity=<pom+wrapper+toolchain+policy>
[wsr cache] MISS; create job-local writable repository; macOS cache remains broker-controlled
[wsr step=setup-java] start runtime=Bun@<qualified-version> action=<resolved-setup-java-sha>
[wsr step=setup-java] expose selected Linux JDK to action tool-cache contract
[wsr state] validate/export JAVA_HOME and PATH; retain job-owned tool files
[wsr step=setup-java] exit=0; next step gets explicit environment updates
[wsr step=verify] exec ./mvnw --batch-mode --no-transfer-progress verify
[wsr step=verify] cwd=/workspace/java-001 network=approved-dependency-endpoints
[workload verify] [INFO] Building weather-core
[workload verify] [INFO] Tests run: 42, Failures: 0, Errors: 0, Skipped: 0
[workload verify] [INFO] Building jar: target/weather-core-0.1.0.jar
[workload verify] [INFO] BUILD SUCCESS
[wsr step=verify] exit=0; artifacts remain in run workspace
[wsr lifecycle] complete any registered action cleanup; do not omit qualified post behavior
[wsr cache] stage eligible package data; exclude settings.xml, credentials, logs and target/
[wsr cache] publish dependency entry atomically in separate trust namespace
[wsr result] verify=passed; JAR/test reports identified; external upload=none
[wsr cleanup] stop descendants; destroy private scratch; retain bounded approved diagnostics
```

The action tool-cache bridge and dependency import/export are proposals, not existing integrations.
A JAR in the workspace is not an `upload-artifact` operation. An artifact action would introduce
another action/API contract requiring qualification.

## 7. If this is a Gradle project

Use `gradlew`, `gradle/wrapper/`, `settings.gradle.kts`, `build.gradle.kts`, and, where adopted,
dependency locks and verification metadata. Replace the Maven job's setup/build steps with:

```yaml
      - uses: actions/setup-java@v6
        with:
          distribution: temurin
          java-version: '21'
      - uses: gradle/actions/setup-gradle@v6
        with:
          cache-provider: external
      - name: Check and assemble
        run: ./gradlew --no-daemon --build-cache build
```

This variant assumes the proposed external cache integration exists. The current
[setup-gradle contract](https://github.com/gradle/actions/blob/main/docs/setup-gradle.md) provides
`cache-provider: external` for another Gradle User Home caching mechanism. Do not also enable
`setup-java` Gradle caching or layer `actions/cache` over the same mutable directory.

- **A — Recommended:** one owner for Gradle User Home cache import/export; keep secrets and
  credential-bearing properties out, and use job-local writable state.
- **B — Alternative:** use the action's supported cache provider after its service is qualified.

For this example use A. Gradle task-output caching is a different layer from dependency downloads.
Only cacheable tasks with matching inputs may report `FROM-CACHE`; WSR must not reinterpret that
as having executed their bodies. See [Gradle build caching](https://docs.gradle.org/current/userguide/build_cache.html).

```text
SIMULATED GRADLE VARIANT — warm dependency/task cache
[wsr step=setup-gradle] qualified Bun action; external cache owner=WSR proposed integration
[wsr cache] Gradle dependencies HIT; task-output namespace restored under its own trust rules
[wsr step=build] exec ./gradlew --no-daemon --build-cache build
[workload build] > Task :compileJava FROM-CACHE
[workload build] > Task :test
[workload build] > Task :jar
[workload build] BUILD SUCCESSFUL
[wsr result] compile reused by Gradle; test executed in this example; no WSR success-cache hit
```

- **A — Recommended initially:** disable persistent cross-run Gradle daemons; accept possible
  single-use daemon behavior and supervise all descendants.
- **B — Alternative:** retain a daemon only within an explicitly bounded job/toolchain lifetime.

Use A (`--no-daemon`). Configuration-cache persistence is another opt-in design problem: assess
serialized sensitive state and inputs before retaining it. No default shared configuration cache
is promised here.

## 8. Edit, rerun, and understand the warm path

For source selection, **A — Recommended for local DX**
is an explicit, recorded working-tree overlay applied after checkout in the isolated workspace;
**B — Alternative** is committed-revision-only execution. We assume A for this edit loop, but its
selection syntax and inclusion rules remain open. It is not a silent rewrite of checkout semantics.
The overlay includes deletions and selected untracked files, and excludes private files by contract.
After explicitly selecting that proposed overlay mode, change a test and rerun the workflow. The
unchanged `wsr run` command alone does not select the overlay.

```text
SIMULATED SECOND RUN — one Java test changed
[wsr source] local-overlay selected explicitly; changed test captured; host checkout untouched
[wsr compile] HIT; workflow/action metadata unchanged; dynamic state is fresh
[wsr cache] provider/action/JDK artifacts HIT; dependency snapshot eligible -> restore
[wsr workspace] fresh mutable workspace; never restore target/ as a WSR test-success shortcut
[workload verify] [INFO] Tests run: 42, Failures: 1, Errors: 0, Skipped: 0
[workload verify] [ERROR] WeatherTest: expected Celsius conversion did not match
[wsr step=verify] exit=1; no runtime fallback or broader grants
[wsr result] failed; permitted reports retained; no release operation scheduled
```

Changing POMs, wrapper properties, JDK, platform, Maven settings semantics or plugin inputs affects
reuse. Maven snapshots/dynamic metadata need freshness policy; a dependency cache is not a lockfile.
Gradle additionally owns its task keys. No cache hit proves an arbitrary build is offline-ready.

For faster subsequent runs, A is a warm but cleaned Linux environment with fresh job state; B is
recreate the VM every time. Prefer A only after cleanup/isolation tests pass. For a JDK matrix,
A is bounded parallel jobs with separate workspaces and target/toolchain-aware cache namespaces;
B is serial jobs. Prefer A when resources allow; do not parallelize dependent steps.
Concurrent downloads require locking and atomic publication, not shared writable Maven/Gradle homes.

## 9. Package locally; publish in a separate release path

- **A — Recommended:** ordinary CI ends at `verify`/`build`; an explicit release workflow handles deploy.
- **B — Alternative:** a conditionally gated publish job lives in the CI workflow.

Use A. If `.github/workflows/release.yml` is added, inspect it separately:

```sh
wsr inspect .github/workflows/release.yml
wsr run .github/workflows/release.yml --event workflow_dispatch --dry-run --verbose
```

The release workload might use `./mvnw --batch-mode deploy` or `./gradlew --no-daemon publish` after
repository/plugin/signing configuration is complete. These are real external writes, not tutorial
commands to execute now. Maven `install` targets a local repository; `deploy` targets a remote one.
Gradle needs the publishing plugin and configured publications/repositories.

```text
SIMULATED RELEASE PREFLIGHT — no upload
[wsr release] destination and signing/credential contract required
[wsr policy] registry-write grant absent; secrets absent
[wsr plan] BLOCKED before workload execution; existing CI success does not authorize release
```

Cache dependency downloads, not `settings.xml`, signing keys, tokens, or release authorization.
An actual release records produced artifact identities and upload outcomes; uncertain uploads are
not automatically replayed. GitHub permissions and local owner grants are distinct.

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

For corruption, A is visible quarantine/reacquisition of the same pinned object when allowed; B is
stop and require explicit repair. We prefer A for this example, but repair policy is still open.
No repair silently changes a pinned version.

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
