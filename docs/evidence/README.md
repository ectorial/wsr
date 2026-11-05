# Execution results and attestations

## Scope and status

D-030 separates local results from optional signatures and publication. Results, signing and
connectors remain unimplemented. Format, key custody and trust policies require explicit decisions.

```mermaid
flowchart LR
    E[Supervised execution] --> R[Local result bundle]
    R --> S[Optional trusted signing]
    S --> P[Optional connector publication]
    R --> P
```

## Local evidence

Record source/workflow revision or digest, resolved actions/providers/packages, platform/backend,
engine/runtime identities, policy identity/effective grants, inputs suitable for safe retention,
job/step outcomes and artifact digests. Link bounded logs and metrics under their access rules.
Mutable logs need stable references or digests when included in claims.

Capture authoritative host observations separately from workload-supplied assertions. A workload
can print success or forge its own report. Containment and a signature do not establish truthful
tests. Cross-run comparisons need compatible environments and controlled inputs; timestamps,
randomness and external services may affect reproducibility.

## Signing and verification

A signature binds claims to a signer. The verifier must check subject digests, trusted issuer/key,
claim semantics, policy/environment requirements and freshness/revocation rules. Signing keys stay
outside workload access. Attestation format, signer identity, secure collection and key rotation are
open. Redact/exclude secrets before persistence or signing.

Attestations do not automatically satisfy GitHub required checks. A separately configured reporting
integration and receiver trust policy must determine acceptance.

## Publication and recovery

Execution success and publication success are separate statuses. Keep immutable local results if a
connector is unavailable. Any configured outbox needs stable result identities, bounded retention,
idempotency and explicit retry/cancellation semantics. Signing/publishing failures must not rewrite
a completed execution outcome. Offline publication waits for explicitly enabled connectivity.

Test identity binding, artifact mutation, forged workload claims, unauthorized signing, secret
exclusion, reporting outages and duplicate delivery. These are future gates.
