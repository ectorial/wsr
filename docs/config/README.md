# Configuration and policy

## Current scaffold

The current checkout has no configuration loading/generation implementation or supported published
schema. `init` is a stub. Earlier `wsr.json`, TOML, schema URL, and secret-file examples are not a
working setup contract. [Shared types](../../crates/wsr-types/src/lib.rs) do not define configuration.

## Accepted design — planned

Repository setup discovers providers, verifies/pins components, and reuses a shared local cache.
Multiple providers require explicit selection when the workflow target is ambiguous. Cached pins
support offline use; upgrades are explicit. See D-008 through D-014 in [PLAN.md](../../PLAN.md).

Repository-owned configuration may describe workflows and capability requests. Trusted owner or
administrator policy authorizes grants; an untrusted workflow cannot widen that policy. An explicit
broad-access profile is a policy exception, not a repository-controlled permission bypass.

Configuration format, pin/lock schema, policy location, grant vocabulary, wildcard scope/syntax,
and credential provisioning remain open. No example here should be treated as a stable API.
See [the security model](../SECURITY-MODEL.md).
