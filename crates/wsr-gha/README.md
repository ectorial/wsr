# GitHub Actions provider

```mermaid
flowchart LR
    Y[GHA YAML and action metadata] --> G[GHA compiler]
    G --> C[Canonical requirements]
    C --> E[Shared WSR engine]
```

**Current scaffold:** this crate's library contains documentation only. Its tests validate example
inputs and native shell references; they do not translate or execute workflows through WSR.

**Accepted design:** retain the provider adapter and lower GitHub Actions semantics into the
provider-neutral execution model. Use Bun for compatible GHA JavaScript actions inside the isolated
system backend. No GHA parser, Bun integration, component executor, or state bridge is implemented.

## Documentation ownership

[PLAN.md](../../PLAN.md), especially D-024, owns accepted decisions. This crate owns their GHA-specific
interpretation and validation requirements. Shared scheduling, authorization, execution, and state
mechanisms remain core responsibilities. These pre-alpha documents describe the design at this
revision; they do not freeze APIs, schemas, crate boundaries, or compatibility across commits.

| Document | Responsibility |
| --- | --- |
| [Architecture](docs/ARCHITECTURE.md) | Provider boundary, execution paths, and remaining design gaps |
| [Compatibility](docs/COMPATIBILITY.md) | Action categories, Bun qualification, and unsupported behavior |
| [State](docs/STATE.md) | GHA state channels mapped to shared supervisor contracts |
| [Validation](docs/VALIDATION.md) | Evidence required before advertising support |

The generic [provider contract](../../docs/provider/README.md) remains authoritative for all providers.
The [implementation checklist](../../CHECKLIST.md) records delivered behavior. Design approval does
not complete any execution or security gate.
