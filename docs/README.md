# WSR documentation

Start with the [project context guide](context/README.md) for reading order and authority.
[AGENTS.md](../AGENTS.md) remains the agent entry point. The checkout is a CLI/types scaffold.
The following documents specify planned behavior, open contracts and verification requirements. Start with the architecture, then follow one subsystem.

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
| [Decisions](context/decisions.md) | Accepted direction and open decisions |
| [Architecture](context/architecture.md) | Whole-system responsibilities and handoffs |
| [Providers](provider/README.md) | Syntax/semantic lowering and provider plugin boundary |
| [GHA provider](provider/gha/README.md) | GHA action/state/compatibility contracts |
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
| [Security model](security/README.md) | Threat model, invariants and residual risks |
| [CLI](cli/README.md) | Actual scaffold behavior and open interface |
| [Roadmap](context/roadmap.md) | Delivery gates |
| [Checklist](context/status.md) | Implemented behavior with evidence |
| [Testing context](testing/README.md) | Reading routes for strategy, evidence and current commands |
| [Testing plan](testing/strategy.md) | Development and compatibility test strategy |
| [Coverage index](testing/coverage.md) | Observed evidence and pending tests |
| [Testing WSR](../tests/README.md) | Exact current commands and fixture conventions |
| [Template migration](development/template-migration.md) | Tooling provenance and recovery rules |
| [Crates](../crates/README.md) | Source inventory, maintenance dispositions and publication constraints |

## Historical records and paused work

Completed records are grouped in the [archive](archive/README.md); paused worker contracts and
checkpoints remain under [implementation](implementation/README.md) for saved external prompts.
They are not active specifications or permission to execute historical next-action commands.

| Record | Status and authority |
| --- | --- |
| [CLI skeleton plan](archive/cli-skeleton.md) | Completed implementation record; the [CLI guide](cli/README.md) owns current behavior. |
| [Design refactor record](archive/design-docs-refactor.md) | Historical audit and completed documentation separation; current contracts live in the ownership map above. |
| [Java worker](implementation/java-e2e.md) | Paused worker contract; explicit resumption is required before implementation. |
| [Java progress](implementation/java-progress.md) | Dated preparation evidence; its current checkpoint supersedes historical next actions. Machine-local artifacts are not clean-clone inputs. |

## Walkthroughs and proposals

[Java](dx/java.md), [Python](dx/python.md) and [Node](dx/node.md) show simulated target experiences.
Their adjacent YAML files remain reference inputs outside .github/workflows. Their A/B choices
require decisions before implementation. Shared contracts above override illustrative logs/paths.

Maintain one owner for each contract. PLAN records authority and status; detailed guides own the
explanation. Preserve historical test/prototype evidence instead of relabelling it as implementation.
When a decision changes, reconcile overview, subsystem, security, tests and affected walkthroughs.
Organization summaries should link to this design without introducing separate runtime commitments.

For cleanup, classify content as active, planned, historical, superseded or an archive candidate.
Age or lack of implementation alone does not invalidate an accepted design. Preserve decisions,
licenses, release history, native-reference evidence, migration backups and the prototype stash.
Label historical records and remove them from active guidance before considering relocation;
any later move must repair incoming links and preserve provenance. Do not duplicate subsystem
contracts in a new cleanup roadmap. Package dispositions live in the crate inventory.

## Documentation maintenance

[Layout and consolidation plan](development/documentation-layout.md) records canonical locations,
compatibility entry points, preserved local guides and the old-to-new mapping.
