# Project context for contributors and agents

Use this directory to establish project authority and current status before selecting work.
Read the root [AGENTS.md](../../AGENTS.md) for repository operating instructions. Documentation
records decisions and evidence; it does not grant permission to resume paused work or publish.

## Read only the context needed for the task

1. [Project overview](../../README.md): what the checkout actually supports.
2. [Implementation status](status.md): delivered behavior and explicitly missing capabilities.
3. [Decisions](decisions.md): accepted direction, proposals, open choices and implementation holds.
4. [Architecture](architecture.md): source boundaries, preserved prototype and target responsibilities.
5. [Roadmap](roadmap.md): delivery order and evidence required to complete each gate.
6. [Documentation index](../README.md): select the relevant subsystem, test or worker guide.

For a small task, inspect its owning source and guide rather than loading every design document.
Read the relevant decision sections before interpreting planned behavior as an implementation task.
Root PLAN, ARCHITECTURE, ROADMAP and CHECKLIST files are redirects, not additional owners.

## Task-specific context

| Task | Read next |
| --- | --- |
| Rust changes and verification | [Contribution guide](../../CONTRIBUTING.md), [test strategy](../testing/strategy.md), [coverage evidence](../testing/coverage.md), [current test commands](../../tests/README.md) |
| Subsystem design | [Ownership map](../README.md#ownership-map), then the owning guide |
| GHA compatibility | [GHA provider](../provider/gha/README.md), including its validation gates |
| Java worker or scheduled implementation | [Worker index](../implementation/README.md), contract and current checkpoint; Java implementation remains paused pending explicit owner resumption |
| Tooling or prototype recovery | [Template migration](../development/template-migration.md), [crate inventory](../../crates/README.md) and architecture's preservation rules |
| Past implementation or documentation audits | [Archive index](../archive/README.md); historical next actions are not current instructions |

## Keep context durable

Keep accepted decisions in decisions.md, completion evidence in status.md and testing/coverage.md,
and worker continuity in the owning checkpoint. Link between them instead of copying their contents.
Date observations and identify revisions and evidence limits. Refresh Git, PR and runtime state live;
do not store changing branch inventories here. Preserve original evidence and explicit stop instructions.
