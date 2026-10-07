# wsr documentation

The checked-out project is a scaffold. Subsystem descriptions below distinguish source inventory
from accepted design; none is a workflow-execution tutorial.

| Guide | Responsibility |
| --- | --- |
| [Decisions](../PLAN.md) | Authoritative accepted decisions and open proposals |
| [Architecture](../ARCHITECTURE.md) | Current source and target boundaries |
| [Roadmap](../ROADMAP.md) | Delivery order and completion gates |
| [Checklist](../CHECKLIST.md) | Implementation status |
| [Security model](SECURITY-MODEL.md) | Planned grants, containment, and compatibility exception |
| [CLI](cli/README.md) | Defined arguments versus implemented commands |
| [Configuration](config/README.md) | Unsettled configuration and policy ownership |
| [Providers](provider/README.md) | Compilation/plugin boundary and compatibility |
| [Engine](engine/README.md) | Planning, evaluation, job/step state, and results |
| [Sandbox](sandbox/README.md) | Component/system execution and open backends |
| [Sync](sync/README.md) | Integration boundaries and deferred hooks |
| [Crates](../crates/README.md) | Existing workspace inventory |

## Keeping repositories consistent

Use these labels consistently: **Current scaffold** (code present, with actual behavior stated),
**Preserved prototype** (local unpublished recovery material), **Accepted design** (planned), and
**Open proposal** (not decided). “Next” describes a delivery gate, not a completed feature.

When changing a decision, update `PLAN.md` first, then affected architecture/security/subsystem guides
and delivery gates. Update the organization summary and both profile files together. When changing
implementation status, cite source and meaningful verification in `CHECKLIST.md`; check organization
status language too. Historical drafts and release notes describe their own time, not current support.

The [organization docs](https://github.com/ectorial/.github/blob/main/README.md) link here rather than
maintaining a separate runtime specification. Publish coordinated changes and verify cross-repository links.
