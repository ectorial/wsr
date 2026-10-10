# Configuration and policy

## Current scaffold

The current checkout has no configuration loading/generation implementation or supported published
schema. `init` is a stub. Earlier `wsr.json`, schema URL, and secret-file examples are historical,
not a working setup contract. Root `wsr.toml` is the selected planned format; the exact TOML schema
is still open. [Shared types](../../crates/wsr-types/src/lib.rs) do not define configuration.

## Accepted design — planned

`wsr init` creates `wsr.toml` at the repository root, preserving existing configuration and pins
on reinitialization. It records repository parameters, configured providers, the primary provider,
workflow discovery settings, and invocation defaults. GitHub Actions is the initial primary
provider; local execution is the default product path. Exact key names are not yet specified.

Normal commands use these defaults. Explicit invocation options override applicable repository
settings, which override built-in defaults. In the CLI proposal, `--provider` and `--target` are
optional overrides. A provider interprets workflow syntax; a destination selects where execution
happens. The current skeleton does not parse these proposed options or load `wsr.toml`.

Repository setup discovers workflows, verifies/pins provider components, and reuses a shared local
cache. Multiple providers can coexist. Explicit provider selection or an unambiguous workflow path
takes precedence over the configured primary provider; conflicting or ambiguous selection is an
error. Cached pins support offline use; upgrades are explicit. See D-008 through D-014 and D-023
in [PLAN.md](../../PLAN.md).

Repository-owned configuration may describe workflows and capability requests. Trusted owner or
administrator policy authorizes grants; an untrusted workflow cannot widen that policy. An explicit
broad-access profile is a policy exception, not a repository-controlled permission bypass.

Configuration must not contain raw secret values or enable third-party providers/broad access
without trusted authorization. CLI overrides also remain subject to that policy.

Exact TOML keys and schema versioning, parameter binding, pin/lock schema and location,
environment-variable overrides, policy location, grant vocabulary, wildcard scope/syntax,
and credential provisioning remain open. No example here should be treated as a stable API.
See [the security model](../SECURITY-MODEL.md).
