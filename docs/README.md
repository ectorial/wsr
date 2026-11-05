# WSR design documentation

The checkout is a CLI/types scaffold. The following documents specify planned behavior, open
contracts and verification requirements. Start with the architecture, then follow one subsystem.

```mermaid
flowchart LR
    A[Architecture overview] --> P[Compile and validate]
    P --> R[Provision environments]
    R --> E[Schedule and execute]
    E --> O[Observe and record]
    O --> C[Optional report]
```

## Ownership map

| Guide | Owns |
| --- | --- |
| [Decisions](../PLAN.md) | Accepted direction and open decisions |
| [Architecture](../ARCHITECTURE.md) | Whole-system responsibilities and handoffs |
| [Providers](provider/README.md) | Syntax/semantic lowering and provider plugin boundary |
| [GHA provider](../crates/wsr-gha/README.md) | GHA action/state/compatibility contracts |
| [Planning and validation](planning/README.md) | Structural validation, binding and admission phases |
| [Runner provisioning](provisioning/README.md) | Profiles, tools, dependency closure and environment readiness |
| [Content cache](cache/README.md) | Verified immutable reuse, acquisition, deletion and retention |
| [Trusted policy](policy/README.md) | Requests, grants and admission authority |
| [Engine](engine/README.md) | Scheduler, dispatcher, supervisor and job lifecycle |
| [Execution backends](sandbox/README.md) | Platform and enforced isolation mechanisms |
| [Observability](observability/README.md) | Scoped logs/metrics, device pressure and contention |
| [Evidence](evidence/README.md) | Results, artifact identity and optional attestations |
| [Integrations](sync/README.md) | Optional publication/hosted reuse and external dependencies |
| [Configuration](config/README.md) | Repo/global settings, pins and metadata proposals |
| [Security model](SECURITY-MODEL.md) | Threat model, invariants and residual risks |
| [CLI](cli/README.md) | Actual scaffold behavior and open interface |
| [Roadmap](../ROADMAP.md) | Delivery gates |
| [Checklist](../CHECKLIST.md) | Implemented behavior with evidence |
| [Testing plan](TESTING-PLAN.md) | Development and compatibility test strategy |
| [Coverage index](TEST-COVERAGE.md) | Observed evidence and pending tests |
| [Testing WSR](../tests/README.md) | Exact current commands and fixture conventions |
| [Design refactor record](implementation/design-docs-refactor.md) | Audit, scope and reconciliation record |
| [Java worker](implementation/java-e2e.md) | Authorized bounded TDD worker contract |
| [Java progress](implementation/java-progress.md) | Historical reference/preparation evidence and next action |
| [Template migration](TEMPLATE-MIGRATION.md) | Tooling provenance and recovery rules |
| [Crates](../crates/README.md) | Source inventory |

## Walkthroughs and proposals

[Java](dx/java.md), [Python](dx/python.md) and [Node](dx/node.md) show simulated target experiences.
Their adjacent YAML files remain reference inputs outside .github/workflows. Their A/B choices
require decisions before implementation. Shared contracts above override illustrative logs/paths.

Maintain one owner for each contract. PLAN records authority and status; detailed guides own the
explanation. Preserve historical test/prototype evidence instead of relabelling it as implementation.
When a decision changes, reconcile overview, subsystem, security, tests and affected walkthroughs.
Organization summaries should link to this design without introducing separate runtime commitments.
