# GitHub Actions provider

```mermaid
flowchart LR
    Y[GHA YAML and action metadata] --> G[GHA compiler]
    G --> C[Canonical requirements]
    C --> E[Shared WSR engine]
```

**Current implementation:** `wsr-gha` inspects a [small static source subset](../../cli/README.md#static-workflow-inspection)
through the temporary built-in path authorized by D-031. CLI tests exercise real source inspection;
example guards still establish only input/native-reference contracts, not execution support.

**Accepted design:** retain the provider adapter and lower GitHub Actions semantics into the
provider-neutral execution model. Use Bun for compatible GHA JavaScript actions inside the isolated
system backend. General GHA semantic lowering, Bun integration, Component providers and state bridges remain unimplemented.

## Documentation ownership

[PLAN.md](../../context/decisions.md), especially D-024, owns accepted decisions. This guide owns their GHA-specific
interpretation and validation requirements. Shared scheduling, authorization, execution, and state
mechanisms remain core responsibilities. These pre-alpha documents describe the design at this
revision; they do not freeze APIs, schemas, crate boundaries, or compatibility across commits.

| Document | Responsibility |
| --- | --- |
| [Architecture](architecture.md) | Provider boundary, execution paths, and remaining design gaps |
| [Compatibility](compatibility.md) | Action categories, Bun qualification, and unsupported behavior |
| [State](state.md) | GHA state channels mapped to shared supervisor contracts |
| [Validation](validation.md) | Evidence required before advertising support |
| [Frontend modeling references](../../../crates/wsr-gha/docs/frontend-modeling.md) | Crate-local upstream modeling patterns, proposed WSR adaptations and acceptance obligations |

The generic [provider contract](../README.md) remains authoritative for all providers.
The [implementation checklist](../../context/status.md) records delivered behavior. Design approval does
not complete any execution or security gate.
