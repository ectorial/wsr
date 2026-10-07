# Integrations and deferred hook synchronization

## Current scaffold

[wsr-git](../../crates/wsr-git/src/lib.rs) contains documentation only. `hook`, `daemon`, and
`status` handlers are stubs. No hook shims are installed or reconciled by the current implementation.
Legacy `wsr run --hook` examples are not a supported CLI contract.

## Accepted delivery order — planned

Run/debug CI locally first, then invoke the same engine inside GitHub Actions. Define the wrapper's
job/pipeline selection, inputs, scheduling, cancellation, credentials, reporting, and recursion
prevention before implementation. Identical engine code does not imply identical environments.

Independent CI using a GitHub App, webhooks, durable scheduling, and Checks projection is a later
possibility. The organization's profile generator is unrelated to that integration. Its lifecycle
requires a separate maintenance decision.

Hooks and daemon synchronization are secondary possibilities, not the primary product experience.
Do not infer a concurrency-safe implementation from legacy comments about Git filesystem locks.
See [PLAN.md](../../PLAN.md) and [ROADMAP.md](../../ROADMAP.md).
