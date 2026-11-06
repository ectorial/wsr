# GHA state across execution paths

```mermaid
flowchart LR
    S[Step writes authorized channel] --> V[Validate publication]
    V --> O[Step and job outputs]
    O --> N[Eligible downstream evaluation]
```

**Planned contract, not an implemented VFS.** [D-016](../../context/decisions.md#d-016--job-supervisor-with-least-privilege-per-step-environments)
assigns state ownership to the job supervisor. GHA-specific channel interpretation belongs to this
provider; shared storage, views, grants, and lifecycle enforcement belong to the core and backends.

## State mapping

| GHA concept | Intended mapping | Required decisions/evidence |
| --- | --- | --- |
| Workspace and working directory | Supervisor-owned job workspace with explicit per-step views | Stable logical paths, mount/preopen mapping, allowed writes, symlinks, permissions, rename/deletion behavior |
| `GITHUB_ENV` | Validated updates for subsequent eligible steps | Parsing, multiline values, reserved variables, timing, malformed input and failure behavior |
| `GITHUB_PATH` | Validated changes to subsequent step executable search paths | Ordering, scope, path mapping, and executable authorization |
| `GITHUB_OUTPUT` | Step outputs projected into canonical output channels | Parsing, size limits, expression visibility, and step/job output mapping |
| `GITHUB_STATE` | State scoped to the same action's lifecycle | Pre/main/post visibility, action-instance isolation, retention and cleanup |
| Tools and services | Supervisor-managed resources, exposed only where required | Tool identity, service lifetime, readiness, credentials, cancellation and cleanup |
| Artifacts and caches | Explicit operations separated from mutable workspace state | Identity, integrity, retention and trust separation |
| Private scratch and secrets | Per-step grants and private storage | No unintended sharing; bounded lifetime and cleanup |

The environment-file meanings above follow GitHub's
[workflow commands reference](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands).
Exact syntax, visibility, ordering, and failure behavior must be captured in fixtures before claiming
compatibility. Action state must not become a job-global environment map.

## Cross-backend invariants

An authorized write exported from a component step must be visible at the corresponding logical
workspace path to a later Bun/system step, and vice versa. Changing backend must not reset declared
shared state or share private scratch/secrets. Files remain untrusted even when produced by an isolated
step. Reading a file does not authorize executing it or expanding the reader's grants.

Bun and child processes remain subject to the step's authorized workspace view and system restrictions. Component
host interfaces expose only the allowed view; a host process launched through an import is not thereby
contained by Wasm. Environment-file parsing and state export must resist path substitution and malformed
or oversized data; exact limits and rejection diagnostics remain to be specified.

## Mechanism still open

No overlay filesystem, VFS, mount manager, snapshot transfer, or shared scratch lifecycle exists in
`wsr-fs`. Choose live mounts, snapshots/diffs, copy-on-write, or another mechanism only after defining:

- Whether and how writes from failed or cancelled steps become visible; no implicit rollback claim.
- Atomicity of state publication, crash recovery, and concurrent access rules.
- Path normalization, traversal/symlink handling, ownership, executable bits, and platform differences.
- Environment/output publication timing and action cleanup ordering.
- Job isolation, private directory lifetimes, and secret/log separation from reusable content.

A minimal implementation may use explicit shared workspace views with private per-step scratch if
it satisfies the required semantics and security tests. This is an option, not a selected mechanism.
See [validation](validation.md) for the mixed-backend acceptance scenario.
