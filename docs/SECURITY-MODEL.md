# wsr security model

## Status

This is an accepted design summary, not implemented enforcement. [PLAN.md](../PLAN.md), especially
D-018 through D-022, owns decisions. The current checkout has no working sandbox. Report software
vulnerabilities through [SECURITY.md](../SECURITY.md).

## Threat model and authority

Assume repository code, workflow definitions, actions, dependencies, and build scripts may be
malicious on a single-user machine. Hostile multi-tenant operation is deferred. Protect ungranted
host files/credentials, host services/control interfaces, policy/enforcement state, other runs'
private state, and availability within declared limits.

A workflow requests capabilities. Machine-owner or CI-administrator policy authorizes grants.
The backend enforces the granted access and remaining denials. Repository configuration cannot
silently authorize its own requests. Missing enforcement blocks execution before workload code runs.

## Component execution

Components use explicit typed imports with no ambient filesystem, network, environment, secret,
credential, or native-process authority under the default policy. The runtime and host enforce the
contract, including capability delegation through composition and resource/lifecycle limits.
A native process launched through a host function does not inherit Wasm containment automatically.
Concrete runtime, WASI version, and initial host interfaces remain open.

## System execution

Existing tools use a separately isolated Linux backend. First targets are Linux jobs from macOS
and Linux machines; macOS needs a VM-backed Linux environment. Whether Linux also requires a VM
outer boundary is still a proposal. No unrestricted host execution is an automatic fallback.

Filesystem views, network access, processes/descendants, credentials, host/daemon access, resources,
and cleanup require enforceable contracts. Job supervision preserves explicitly shared state while
steps retain their own grants. Shared files can carry malicious state across those boundaries.

## Broad-access compatibility exception

An explicit owner-authorized profile/wildcard may grant broad runner access. It never activates
because a permission check failed. Reports identify the effective profile and grants; broad access
cannot claim the default policy's narrower protection.

The proposed scope is the managed runner, keeping the personal host and policy/control interfaces
outside the grant. That scope is not yet confirmed. Syntax and activation granularity are also open;
`--allow '*'` is an illustrative proposal, not an available CLI option.

A wildcard does not create credentials, change GitHub token permissions, release protected secrets,
implement missing workflow semantics, or turn native tools into components. Only explicitly supplied
credentials can be made available under the eventual policy.

## Limits and verification gates

Granted read access plus an outbound channel can permit data exfiltration. Endpoint allowlists alone
do not prove data-flow safety. Containment, signatures, and content digests do not establish truthful
results, safe artifacts, or complete GitHub Actions compatibility.

Before making security claims, test denied host-file/credential access, host/control-socket probes,
path/symlink/archive traversal, poisoned shared state, resource exhaustion, and descendant cleanup.
Test positive grants and broad-profile authorization separately. Document trusted runtime/host/image
components and residual vulnerability assumptions. No performance or absolute-security guarantee.

Primary mechanism references: [Component Model](https://component-model.bytecodealliance.org/design/why-component-model.html),
[Wasmtime security](https://docs.wasmtime.dev/security.html), and
[GitHub credential/security controls](https://docs.github.com/en/actions/reference/security/secure-use).
