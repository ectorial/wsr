# wsr redesign plan

> Status: **architecture interview in progress — do not implement**
>
> This document is the durable record of the design feedback loop. Confirmed decisions,
> rejected alternatives, unresolved questions, and their consequences are recorded here as the
> conversation progresses. It becomes implementation-ready only after the status above is changed.

## Documentation status and ownership

This is the authoritative product decision record. Accepted decisions describe planned behavior,
not implemented features. The current checkout is a CLI/workspace scaffold: all eight command
handlers are stubs. Substantial prototype work remains in a local stash, separately from current
source and completion accounting.

Use [ARCHITECTURE.md](ARCHITECTURE.md) for source inventory and target boundaries,
[CHECKLIST.md](CHECKLIST.md) for implementation status, [ROADMAP.md](ROADMAP.md) for delivery gates,
and [the security model](docs/SECURITY-MODEL.md) for the accepted trust contract and open exceptions.
Organization docs summarize these records. Historical drafts do not override them.

Documentation can be made consistent while design questions remain open. This does not authorize
implementation, stash restoration, publication, or selection of an unresolved runtime/backend.

## 1. Purpose

Design a from-scratch architecture and implementation plan for `wsr`. The design must not inherit
the current crate structure, runtime choices, compatibility claims, or roadmap without explicit
justification.

## 2. Confirmed product direction

### D-001 — Local-first, cloud-portable workflow execution

**Decision:** `wsr` will first provide an efficient and reliable way to execute GitHub Actions
workflows locally. The same system should later be deployable on infrastructure in any cloud.

**Product benchmark:** improve materially on `act`, especially where its container-based execution
is inefficient or unreliable. Reproducing GitHub's hosted control plane is not the initial goal.

**Architectural consequence:** workflow interpretation and execution must not depend on GitHub's
hosted service or on a particular cloud. Local execution is a real deployment target, not a test
double for a future cloud runtime.

**Still unresolved:** the meaning of “same system” across local and cloud, including which
components, protocols, artifacts, and semantics must remain identical.

### D-002 — Classified compatibility instead of a universal compatibility claim

**Decision:** Existing provider workflows are accepted through provider adapters. Compatibility is
reported per feature as native, faithfully emulated, fallback-required, unsupported, or unknown.
Unsupported and approximate behavior must be diagnosed rather than silently accepted.

**Architectural consequence:** compatibility analysis is a first-class compiler output. It is not
an incidental runtime error path.

### D-003 — Provider-agnostic core with a canonical workflow IR

**Decision:** GitHub Actions, GitLab CI/CD, Bitbucket Pipelines, and future systems enter `wsr`
through provider adapters. The execution core consumes a `wsr`-owned provider-neutral workflow
representation rather than provider YAML or provider-specific objects.

**Architectural consequence:** provider parsing, source semantics, expression languages, implicit
defaults, and diagnostic source locations belong outside the execution core. The canonical model
and its versioning are owned by `wsr`.

**Clarification resolved by D-004:** one source workflow produces one canonical workflow;
individual constructs may lower into zero, one, or multiple nodes. The former “1:1 mapping”
terminology question is not an outstanding decision.

### D-004 — Workflow-level mapping with construct-level lowering

**Decision:** One provider source workflow compiles into one canonical `wsr` workflow with complete
source traceability. Individual provider constructs may lower into zero, one, or multiple canonical
IR nodes.

**Rejected alternative:** requiring one canonical construct for every provider construct. That
would turn the provider-neutral model into an accumulating union of provider keywords and quirks.

**Architectural consequence:** adapters preserve provider source locations and provenance across
lowering. Semantics that cannot be represented canonically produce compatibility diagnostics or an
explicit fallback requirement; adapters may not pass opaque provider operations into the execution
core.

### D-005 — Common semantic kernel with controlled extensions

**Decision:** The canonical workflow IR models concepts shared across CI providers. Workflows,
jobs, steps, dependencies, conditions, variables, outputs, and similar concepts belong in the core
when provider research confirms a common semantic meaning.

Provider syntax is not copied directly into the core. An adapter lowers provider syntax into the
common model. Provider-exclusive behavior remains in the adapter when it can be expressed through
core concepts; behavior that cannot be expressed may require a controlled extension mechanism.

**Architectural consequence:** the IR is a semantic compatibility model, not the union of every
provider's YAML schema. Admission criteria for core concepts and runtime-visible extensions must be
defined before implementing adapters.

### D-006 — No cross-provider source translation

**Decision:** `wsr` will not export a workflow from one provider's syntax into another provider's
syntax. GitHub Actions to GitLab CI/CD and reverse translation are explicitly outside the plan.

**Preserved requirement:** every provider frontend must compile into the same canonical workflow
abstraction so the execution core remains provider-agnostic.

**Architectural consequence:** provider integrations are one-way compiler frontends. The canonical
IR needs semantic fidelity for validation and execution, but it does not need enough presentation
information to reconstruct target-provider YAML.

### D-007 — Rust trait as the provider frontend abstraction

**Decision:** provider integrations will implement a `wsr`-owned Rust trait that translates
provider input into the canonical workflow model and compatibility diagnostics.

**Architectural consequence:** the trait must depend on provider-neutral request, result, IR,
diagnostic, and capability types. It must not expose GitHub-, GitLab-, or Bitbucket-specific types
to the execution core.

**Clarification:** the Rust trait is the in-process host abstraction. A downloaded plugin cannot
directly become a Rust trait object because Rust crates are linked at build time and Rust trait
object layout is not a stable plugin ABI. A host-side trait implementation must delegate across a
separate stable plugin protocol.

### D-008 — Provider discovery and first-use plugin installation

**Decision:** the stock `wsr` binary discovers provider configuration in a repository. During
repository setup, it resolves and installs the corresponding official precompiled provider plugin
when it is not already available. For example, discovering `.github/workflows` selects the official
GitHub Actions provider.

Provider installation also occurs when `wsr` is reinstalled or reinitialized for a repository as
needed. Normal execution uses the installed, verified provider artifact rather than rebuilding a
Rust crate.

**Required lifecycle:** discover, resolve, fetch, verify, install/cache, connect through the host
provider abstraction, then compile the repository's workflows.

**Architectural consequence:** provider identity, protocol compatibility, artifact platform or
runtime compatibility, version selection, integrity verification, trust policy, cache location,
repository pinning, upgrades, offline behavior, and recovery from failed installation are product
requirements—not incidental download details.

**Boundary resolved by D-009:** providers use signed WebAssembly Components with a versioned WIT
contract. Native executable/IPC and dynamic-library/C-ABI formats were earlier candidates,
not outstanding alternatives for the accepted provider plugin boundary.

### D-009 — WebAssembly Component provider plugins with a WIT boundary

**Decision:** downloadable providers are signed WebAssembly Components implementing a versioned
WIT provider contract. Provider authors may implement a component in Rust or another supported
language; the installed artifact is a component, not a dynamically linked Rust crate.

The host contains a `ComponentProvider` implementation of the internal Rust provider trait. It
delegates provider operations across WIT to the selected component.

**Architectural consequence:** the WIT contract, canonical data encoding, protocol negotiation,
component resource limits, and host capability grants form a security and compatibility boundary.
Provider components receive no ambient filesystem, network, environment, credential, or secret
access. Required inputs are passed explicitly by the host.

### D-010 — Repository bootstrap and reproducible provider activation

**Decision:** repository-aware commands bootstrap providers when necessary:

1. find the repository root and read existing provider pins;
2. discover provider fingerprints using logic built into the host;
3. resolve the official plugin compatible with the host protocol;
4. download to staging, verify identity, digest, and signature, then install atomically;
5. store the artifact in a shared machine cache and pin its exact identity for the repository;
6. instantiate it through `ComponentProvider`, compile workflows, validate the result, and continue
   with the requested command.

`--help` and `--version` never scan or download. `init` performs bootstrap explicitly; repository
commands such as `run` and `inspect` may bootstrap an uninitialized repository. Provider upgrades
are explicit. A cache hit for the pinned artifact requires no network access.

Official providers may be installed automatically with visible progress after discovery.
Third-party providers require explicit approval. Missing artifacts in offline mode fail with a
specific remediation rather than selecting an unpinned version.

### D-011 — Multiple providers are supported without implicit selection

**Decision:** one repository may install and pin multiple detected providers. When a command could
target workflows from more than one provider, `wsr` requires an explicit selection or an
unambiguous workflow path. It never guesses which provider the user intended.

### D-012 — Provider plugins terminate at the compilation boundary

**Decision:** provider plugins parse provider expressions, apply provider defaults, classify
evaluation phases, and lower expressions into a canonical expression IR. They are not called during
workflow planning or execution after compilation succeeds.

The provider-neutral core owns deterministic evaluation at the appropriate phase, including event
and input binding, matrix expansion, job conditions, step conditions, status functions, and values
derived from completed step or job outputs.

**Architectural consequence:** a compiled workflow and its execution bundle contain no opaque
provider expressions or runtime callbacks into a provider component. Unrepresentable expressions
produce a declared extension requirement or compatibility diagnostic.

### D-013 — Shared content-addressed cache is a core subsystem

**Decision:** `wsr` maintains a central per-user cache shared across repositories. Repositories pin
logical identities and exact versions or digests; immutable content is stored once and reused by
every repository with matching requirements.

Candidate cache object classes include:

- verified provider components and provider metadata;
- resolved action sources and immutable action artifacts;
- compiled action components or other execution artifacts;
- compiler outputs such as canonical IR snapshots and execution bundles;
- runtime and toolchain artifacts required by compatibility backends;
- provider schemas and safely cacheable resolution metadata.

**Correctness requirement:** derived-object keys include all inputs that can affect output, such as
source digest, provider identity and version, compiler version, target, configuration, feature set,
and relevant policy version. A cached result must never be reused merely because two repositories
have the same filename.

**Security boundary:** secrets, credentials, raw secret-bearing event payloads, mutable run state,
and unredacted logs are not stored in the shared content-addressed cache. Verified and unverified
artifacts occupy distinct trust states and cannot alias.

**Required operations:** inspect, verify, garbage-collect, repair, and explain why an object is
retained. Installation and writes are staged and atomic. Repository pins act as roots for retention
and reproducibility; cache deletion must not silently change the selected version.

### D-014 — Local cache only for the MVP

**Decision:** the MVP implements a per-user, filesystem-backed cache shared by local repositories.
Remote, team, and cloud cache services are deferred until the local content store proves its
correctness, locking, integrity verification, garbage collection, repair, and trust separation.

**Future-proofing boundary:** cache consumers depend on a narrow content-store abstraction rather
than filesystem paths. This preserves the option to add a remote backing store later without
requiring an MVP network protocol or distributed consistency model.

### D-015 — Tiered execution with WebAssembly preferred and compatibility fallbacks

**Decision:** the planner classifies workflow operations by execution requirements. Native
WebAssembly execution is preferred. When semantics or dependencies cannot run there, `wsr` selects
a compatible backend such as a JavaScript runtime or an OCI/container backend. Docker is avoided
when possible but remains available for workflows that genuinely require container semantics.

The tier system is capability-based rather than provider-based: the same workflow may contain
operations with different compatibility classifications. Each classification is visible during
inspection and planning.

**Resolved by D-016:** tier selection precedes side effects, with no automatic replay after
partial execution; jobs use supervision with separate step environments.

**Unresolved:**

- the exact MVP backends and whether OCI support ships in the first milestone;
- how requested operating-system and architecture labels map to local host, container, VM, or later
  remote runners.

**Feedback after initial job-promotion proposal:** promoting every step to the strongest backend
required by any step would unnecessarily widen privileges and reduce step isolation. The design
must distinguish a job's shared compatibility substrate from each step's least-privilege sandbox.
The replacement is a job supervisor plus ephemeral per-step environments.

### D-016 — Job supervisor with least-privilege per-step environments

**Decision:** the job is a controlled state and lifecycle boundary, not a single execution tier.
Every step is classified before execution, receives its own least-privilege environment, executes
once, exports permitted state through host-mediated channels, and is destroyed. A step requiring
OCI does not promote unrelated steps to OCI.

The job supervisor owns the workspace state, environment and path updates, step and job outputs,
toolchain state where required, services, cancellation, and timeouts. It exposes only the relevant
views and capabilities to each step environment.

**State rule:** step isolation applies to processes, secrets, capabilities, scratch space, and
unshared filesystem or network access. Workflow-declared shared state remains observable by later
steps. Changes cross the boundary through explicit mounts, snapshots/diffs, environment files,
output channels, artifact operations, or other audited host interfaces.

**Fallback rule:** tier selection completes before a side-effecting step begins. `wsr` does not
automatically replay a partially executed step in a more permissive backend.

**Platform constraint:** Linux containers share a Linux kernel and can provide different Linux
userlands, but they do not provide faithful Windows or macOS execution. True cross-OS matrices
require matching native hosts, virtual machines where legally and technically available, or later
remote runners.

### D-017 — Local execution followed by reuse inside GitHub Actions

**Decision:** the first product solves running and debugging CI locally. The next integration
reuses that execution engine inside GitHub Actions. Independent CI remains a later possibility,
not a requirement for the first useful release.

**Architectural consequence:** local execution must not require a hosted controller, GitHub App,
worker fleet, or remote database. The Actions integration should invoke the same engine rather
than introduce a second implementation of workflow semantics.

**Still unresolved:** the Actions wrapper's unit of execution (selected job or whole pipeline),
ownership of scheduling and cancellation, input/context binding, credentials, reporting, and how
it avoids recursively invoking its own wrapper workflow. Reusing the engine does not establish
identical local and hosted environments or outcomes.

### D-018 — Component Model as a core security boundary

**Decision:** the WebAssembly Component Model is central to the product's deny-by-default
execution model, not merely an optional performance optimization. Concrete runtimes, WASI
versions, and the initial system compatibility backend remain open choices.

**Architectural consequence:** components interact with the outside world through explicit,
typed imports. Under the default policy, the host supplies no ambient filesystem, network, environment, credential,
secret, or native-process authority. Requested capabilities are granted only within policy
authorized by the machine owner or CI administrator; an untrusted workflow cannot authorize
itself. Grants must also account for composed components and authority passed between them.

**Security scope:** the Component Model defines interfaces; the runtime and host must enforce
the restrictions. A broadly privileged host function can undermine the intended boundary.
Component containment does not automatically constrain a native process launched by the host,
and signatures or content digests do not establish that a component is safe.

**Compatibility consequence:** shell commands, existing JavaScript actions, compilers, and
package managers are not assumed to become components automatically. Their execution requires
a separately defined security contract. A component wrapping a native command must not be
reported as giving that command component-level containment.

**Validation requirement:** security claims require negative tests demonstrating denied access
as well as positive tests demonstrating authorized operations. The contract must cover resources,
state shared between steps, and secrets in addition to typed interface compatibility.

**References:** [Component Model interfaces and composition][component-model];
[WebAssembly sandboxing and WASI capabilities][wasmtime-security]. These establish the design
mechanisms, not verification of WSR's unimplemented host.

[component-model]: https://component-model.bytecodealliance.org/design/why-component-model.html
[wasmtime-security]: https://docs.wasmtime.dev/security.html

### D-019 — Isolated system execution with explicit grants and fail-closed admission

**Decision:** ordinary shell steps and native tools may execute through an isolated system
backend with explicit capability grants. If the selected backend cannot enforce the required
restrictions, WSR rejects execution. A component-only launch and an unrestricted native mode
are not the selected default contract. D-022 adds an explicitly authorized broad-access profile;
it does not authorize an automatic downgrade when this default contract cannot be enforced.

**Admission rule:** the planner matches required OS/architecture, job-state semantics, isolation,
and requested capabilities against the backend's enforced capabilities. Policy authorizes grants;
the backend must enforce the remaining denials. A missing enforcement feature is an actionable
diagnostic, not an implicit permission grant or downgrade. Unsupported steps block admission of
the selected execution plan before repository-controlled code runs.

**Enforcement scope:** filesystem views and writable paths, networking, environment and secrets,
processes and descendants, host/daemon access, and resource/lifecycle limits must be specified.
Clearing environment variables, setting a working directory, or wrapping a command in a component
does not establish this isolation. Containers or VMs are implementation candidates, not evidence
that a particular configuration satisfies the contract.

**Job state:** D-016 still applies. Shared workspace, tools, outputs, and services have explicit
lifetimes and access rules; a grant to one step does not automatically authorize later steps.
Shared files can carry malicious state, so process isolation alone does not establish independent
trust between steps.

**Validation requirement:** test permitted operations and attempts to access ungranted files,
network endpoints, credentials, processes, and host interfaces. Verify descendant cleanup on
timeout/cancellation and rejection when an enforcement capability is unavailable. Document the
tested platform/backend combinations and their residual trust assumptions.

**Still unresolved:** concrete backend, policy/grant format, and enforceable resource and network
restrictions. Initial execution platforms are resolved by D-020; the threat model by D-021.

### D-020 — Linux jobs from macOS and Linux machines first

**Decision:** the first release supports local development on macOS and Linux machines while
executing Linux jobs. Native macOS and Windows jobs are deferred.

**Architectural consequence:** the CLI host OS and a job's execution OS are distinct. On macOS,
Linux system jobs require a VM-backed Linux environment. On Linux, the concrete isolation backend
remains to be selected under D-019. The Component Model's security contract still governs
component operations regardless of the CLI host OS.

**Compatibility rule:** a Linux environment does not satisfy a workflow requesting native macOS
or Windows behavior. Unsupported platform requirements must be diagnosed rather than silently
rewritten as Linux jobs.

**Still unresolved:** supported host/job CPU architectures and any emulation policy, VM/container
runtime selection, environment provisioning, image/toolchain identity, and enforcement parity
between macOS-hosted and Linux-hosted execution. Linux OS support alone does not establish
equivalence to GitHub's hosted runner images.

### D-021 — Malicious workloads on a single-user machine

**Decision:** the first release assumes repository code, workflow definitions, third-party
actions, dependencies, and build scripts may deliberately attempt to steal credentials or
access resources outside their grants. The initial deployment is a single-user machine;
hostile multi-tenant remote execution is deferred.

**Protected assets:** host files and credentials outside grants, host services and runtime control
interfaces, WSR policy and enforcement state, other runs' private state, and machine availability
within declared resource limits. All inputs originating from a workload, including paths,
archives, logs, artifacts, and claimed results, must be treated as untrusted at host boundaries.

**Authority rule:** a workload cannot change the owner-approved policy, turn its capability
requests into grants, acquire inherited host credentials, or silently widen a denied capability.
Trust in a source repository or a component signature does not bypass enforcement.

**Security limits:** granted access remains access. A malicious operation that legitimately
receives source data and an outbound channel may leak that data; an endpoint allowlist alone
does not prevent this. WSR cannot establish truthful test results or trustworthy artifacts merely
by containing the program that produces them. The design must document trusted host/runtime
components, shared-state risks, authorized data flows, and residual vulnerability assumptions.

**Validation requirement:** use adversarial fixtures, not only accidental permission failures.
Include host-file and credential probes, attempts to reach host services or control sockets,
path/symlink/archive traversal, poisoned shared state, resource exhaustion, and descendant
processes surviving termination. Reject unsupported enforcement requirements before execution.

**Still unresolved:** the outer isolation boundary for system jobs, boundary lifecycle and reuse,
and whether a dedicated CI environment can satisfy that boundary without another nested VM.

### D-022 — Explicit broad-access compatibility profile

**Decision:** retain deny-by-default execution as the default while offering an explicit opt-in
profile or wildcard parameter for broad runner access. This supports workflows that need more
authority than the default policy allows. Exact syntax and wildcard scope remain unresolved.

**Authority rule:** the machine owner or CI administrator authorizes this profile through a trusted
invocation or policy. A workflow may request it but cannot enable it merely by editing repository
configuration. A permission denial or backend failure must not activate it automatically.

**Proposed scope, awaiting confirmation:** full workload access inside the managed Linux runner
environment, while preserving the boundary to the personal host and WSR's policy/control
interfaces. Credentials remain those explicitly provisioned for the execution; a wildcard does
not discover host credentials, grant GitHub token scopes, or bypass GitHub's secret-release rules.

**Security consequence:** the broad profile gives up the narrower least-privilege guarantees
within its authorized scope. Reports must identify the effective profile and grants. The default
profile's malicious-workload containment claims must not be attributed to a broader grant set.
Remaining restrictions, including any retained outer boundary, still require enforcement.

**Compatibility limit:** broader permissions remove permission-related barriers; they do not
implement missing action protocols, expressions, OS/toolchain behavior, or GitHub-hosted services.
Component imports also remain limited to implemented host interfaces. A wildcard is not a promise
of complete GitHub Actions compatibility or automatic conversion of native programs to components.

**Validation requirement:** prove the default still denies broad access, repository input cannot
self-enable the profile, activation stays within its declared scope, and the effective grants
are reported. Compatibility tests remain separate from permission-grant tests.

**Still unresolved:** wildcard scope, parameter/profile naming, activation granularity (step,
job, or selected pipeline), and the outer isolation decision in Q-020.

## 3. Superseded or unconfirmed prior claims

The following ideas found in the existing repository are not requirements unless this plan later
accepts them explicitly:

- A promise of 100% GitHub Actions compatibility.
- Wasmtime, WASI Preview 3, Wasmer, or WASIX as predetermined runtime choices.
- Automatic conversion of JavaScript actions to WebAssembly.
- A two-tier “Vault / Workshop” execution model.
- One sandbox instance per step.
- Git hooks and a background daemon as the primary product experience.
- The existing 16-crate workspace boundary.
- GitLab CI and Bitbucket Pipelines as the next provider priorities.
- A separate distributed execution project or service named `wkube`.
- A prohibition on every possible container-backed compatibility path.

## 4. Design principles accepted so far

- Begin with user-visible compatibility and reliability, then derive internal boundaries.
- Keep provider syntax outside the provider-neutral execution core.
- Treat local execution as the first end-to-end milestone.
- Reuse the local execution engine inside GitHub Actions before requiring independent CI.
- Make the Component Model and explicit capability grants central to the security contract.
- Reject system execution when the chosen backend cannot enforce the required restrictions.
- Target Linux jobs from macOS and Linux development machines before native macOS/Windows jobs.
- Design for malicious workloads on a single-user machine, with explicit trusted boundaries.
- Allow owner-authorized broad runner access as an explicit exception to the default policy.
- Preserve a path to cloud execution without requiring cloud infrastructure for local use.
- Make unsupported or approximate behavior visible; never silently claim equivalence.

## 5. Architecture decision queue

Questions are resolved in dependency order, one at a time:

1. ~~GitHub Actions compatibility contract.~~ Resolved by D-002.
2. ~~Meaning and scope of multi-provider support.~~ Resolved by D-003 and D-004.
3. ~~Meaning of adapter “1:1 mapping” and preservation of provider semantics.~~ Resolved by D-004.
4. ~~Cross-provider translation.~~ Rejected by D-006.
5. ~~Provider plugin ABI and artifact format.~~ Resolved by D-009.
6. Core-versus-extension admission policy for provider-exclusive behavior.
7. Stability and serialization boundary of the canonical workflow IR.
8. ~~Expression ownership across compilation and execution.~~ Resolved by D-012.
9. Remaining source representation and translation stages.
10. Action compatibility taxonomy (`run`, JavaScript, composite, Docker, local, reusable workflow).
11. ~~Tier selection timing and job-versus-step execution granularity.~~ Resolved by D-016.
12. Execution substrate and concrete compatibility backends.
13. Local/cloud portability boundary and execution protocol.
14. Workspace, filesystem, artifact, output, and service-container semantics.
15. Capabilities, permissions, secrets, identity, and supply-chain trust.
16. Scheduling, matrices, concurrency, cancellation, retry, and failure semantics.
17. Observability, provenance, diagnostics, explainability, and compatibility reporting.
18. Extensibility and versioning policy for providers, actions, IRs, protocols, and plugins.
19. CLI/product workflows and migration from GitHub Actions or `act`.
20. Crate/module boundaries derived from the accepted architecture.
21. Vertical milestones, tests, compatibility corpus, and completion criteria.
22. Documentation set and architecture decision records.

## 6. Open decision

### Q-020 — Outer isolation for system jobs

Q-017 is resolved by D-019: system execution requires explicit grants and rejects execution
when restrictions cannot be enforced. The platform portion of Q-018 is resolved by D-020:
Linux jobs from macOS and Linux hosts first. Q-019 is resolved by D-021: workloads may be
deliberately malicious on a single-user machine.

Next, decide whether system jobs require a VM boundary from a developer's personal host on both
macOS and Linux, or whether a hardened container sharing the Linux host kernel is acceptable.
Per-step capability enforcement remains necessary inside either environment.

The GitHub Actions integration also needs a boundary qualification contract: a dedicated,
disposable CI VM may be a candidate outer boundary, but its lifecycle, resident credentials,
WSR control state, and step enforcement must be evaluated. It must not be assumed equivalent
merely because the runner is called a VM. Nested virtualization must not become an unexamined
requirement for engine reuse in GitHub Actions.

Current recommendation: require a VM outer boundary for system jobs on developer machines,
then explicitly qualify dedicated CI environments. This is a proposal, not an accepted decision.

D-022 adds an explicit broad-access profile. Its wildcard scope must be resolved separately:
broad access within a managed runner does not by itself decide whether the personal-host boundary
is a VM or a hardened container. The proposed runner-only scope is awaiting user confirmation.

**Research constraints:** containers share their execution host's kernel; VM boundaries add a
separate guest kernel but do not replace capability enforcement or safe host interfaces.
GitHub documents nested virtualization as experimental and not officially supported.
See [containers versus VMs](https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-a-container)
and [GitHub-hosted runner limitations](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners).

Deferred:

- whether the canonical IR itself is a stable serialized contract or lowers into a separate stable
  execution protocol;
- final admission rules for core concepts and provider extensions;
- whether source comments and formatting require preservation for diagnostics and edits.

## 7. Implementation plan

To be written only after the architecture decisions above are resolved.

## 8. Validation and reader test

Before this plan becomes implementation-ready, it will be checked for hidden assumptions,
contradictions, underspecified interfaces, infeasible compatibility claims, and whether a fresh
implementation agent can identify the first vertical slice without conversational context.
