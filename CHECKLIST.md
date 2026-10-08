# wsr implementation status

Status checked against the current workspace on October 8, 2026. This file counts implemented
behavior, not design approval. See [PLAN.md](PLAN.md) for decisions and [ROADMAP.md](ROADMAP.md)
for delivery gates.

## Present in the current checkout

- [x] Sixteen-product-crate Cargo workspace and binary entry point, with private template-based xtask tooling.
- [x] Typed CLI argument definitions, help/version, command dispatch, and consistent placeholder diagnostics/exit codes.
- [x] CLI parser/executable contract tests and smoke coverage of all eleven operational command paths.
- [x] Early serializable types, legacy provider trait, and errors.
- [x] CI/release workflow definitions (their existence is not runtime feature coverage).
- [x] Documentation distinguishes current scaffold, preserved prototype, accepted design, and proposals.

## Not implemented in the current checkout

- [ ] Workflow parsing and functional `inspect`/planning commands.
- [ ] Repository setup, configuration loading/generation, and provider bootstrap/pinning.
- [ ] Signed Component Model provider execution and canonical expression lowering.
- [ ] Workflow scheduling, expressions, matrices, outputs, and action resolution/execution.
- [ ] Component host, capability enforcement, and runtime/resource limits.
- [ ] Isolated Linux system execution on macOS and Linux hosts.
- [ ] Default permission policy and owner-authorized broad-access profile.
- [ ] Source snapshots, shared content store, artifacts, and structured execution evidence.
- [ ] Job/step state management, cancellation, timeout cleanup, and failure semantics.
- [ ] Engine reuse inside GitHub Actions.
- [ ] Git hook management or daemon synchronization.
- [ ] Independent CI service, GitHub App deployment, remote workers, and Checks writes.
- [ ] Security/compatibility conformance suites or measured performance claims.

All eleven operational command paths report “not implemented yet” and exit with code 1.
See the [CLI verification record](docs/cli/SKELETON-PLAN.md#verification-record--2026-10-08).
Help and argument parsing do not
establish command functionality. Green scaffold CI is not proof of workflow execution or containment.

## Preserved prototype — not current completion

The local July 29 stash contains inspection/planning, snapshots/evidence, a development command
backend, and GitHub adapter work. The audit counted 29 test bodies but did not run them. These
features stay unchecked above until recovered, reconciled, tested, and present in the working implementation.
The original stash identity and recovery rules are recorded in [ARCHITECTURE.md](ARCHITECTURE.md).
