# wsr — workflow sandboxed runner

A local, Wasm-sandboxed CI runner. Run your workflows on every git hook — before you push, not after.

```bash
cargo install wsr
cd my-repo
wsr init
```

---

## the problem

GitHub Actions is the standard. But the feedback loop is broken: you push, wait for CI, read a
failure, fix it, push again. `act` helped but shells out to Docker — no real sandboxing, root
privileges, and a container daemon you have to babysit.

> **Docker virtualizes the computer. wsr virtualizes the task.**

`wsr` runs your workflows locally, on every git hook, in a Wasm sandbox. Every step gets its own
isolated Wasmtime instance with explicit capability grants. No Docker. No surprises on remote.

---

## how it works

`wsr` maps your workflow triggers to git hooks automatically:

| workflow trigger            | git hook           |
| --------------------------- | ------------------ |
| `on: push`                  | `pre-push`         |
| `on: pull_request`          | `pre-push`         |
| `on: workflow_dispatch`     | `wsr run` (manual) |
| `on: push` with tag pattern | `pre-push`         |

When you run `git push`, your `ci.yml` runs locally first. If it fails, the push is blocked. If it
passes, you already know remote CI will pass too — same workflow syntax, same expressions, same
action versions.

---

## install

```bash
cargo install wsr
```

**requirements**: Rust 1.78+. No Docker. No daemon required.

---

## usage

```bash
wsr init                          # scan workflows, install git hooks, generate wsr.json
wsr run                           # run all workflow_dispatch workflows
wsr run .github/workflows/ci.yml  # run a specific workflow
wsr run --event push              # force a specific trigger event
wsr run --dry-run                 # print execution plan, run nothing
wsr run --verbose                 # show expression eval, sandbox grants, context dump
wsr daemon                        # watch workflow dir, keep hooks in sync
wsr list                          # show all workflows and hook mappings
wsr status                        # show active hooks, daemon state, last sync
wsr cache list                    # list compiled Wasm modules in the cache
```

---

## configuration

`wsr init` generates a `wsr.json` at the repo root. It is optional — sensible defaults apply
without it.

```json
{
  "$schema": "https://wsr.dev/schema/wsr.json",
  "provider": "github",
  "sandbox": {
    "allowed_hosts": [],
    "secrets_from": ".env.wsr"
  }
}
```

The `$schema` field enables IntelliSense and inline validation in VS Code and any JSON
Schema-aware editor. `wsr.json` is plain JSON, readable and writable by any language without extra
dependencies.

---

## sandbox model

Every step runs in its own Wasmtime instance (Tier 1), compiled with Cranelift JIT and running on
WASI Preview 3 — which provides a native async layer to Wasm. Steps get exactly the capabilities
they need and nothing more.

A step that tries to reach a host outside `allowed_hosts` gets a capability denied error —
locally, immediately, with a hint:

```
capability denied: net → registry.example.com
hint: add to sandbox.allowed_hosts in wsr.json
```

Heavy toolchains (`rustc`, LLVM, Go) that require POSIX features like `fork`/`exec` run in
Tier 2 (Wasmer + WASIX) — still fully sandboxed, no Docker required.

---

## provider support

`wsr` is built around a provider adapter pattern. GitHub Actions is the reference implementation.
GitLab CI and Bitbucket Pipelines are planned as first-class adapters — same sandbox, same
execution engine, different workflow syntax and context normalization.

| provider            | status                |
| ------------------- | --------------------- |
| GitHub Actions      | v0.1 — reference impl |
| GitLab CI           | v0.4 — planned        |
| Bitbucket Pipelines | v0.5 — planned        |

---

## github actions compatibility

`wsr` targets 100% GitHub Actions syntax compatibility. MVP coverage:

- [x] `run:` steps with `bash`, `sh`, `pwsh`
- [x] `uses:` — marketplace actions (JS/TS via Javy → Wasm; composite inlined)
- [x] `${{ }}` expression engine — `github.*`, `env.*`, `needs.*`, `fromJSON`, `toJSON`
- [x] matrix expansion
- [x] `needs:` DAG
- [ ] Docker-based actions — compat shim with warning in v0.1, native in v0.2

---

## vs act

|                     | act                  | wsr                                     |
| ------------------- | -------------------- | --------------------------------------- |
| sandbox             | Docker namespaces    | Wasmtime per step (WASI 3)              |
| requires daemon     | Docker daemon        | nothing (optional `wsr daemon`)         |
| CI provider support | GitHub Actions       | GitHub · GitLab · Bitbucket (planned)   |
| local trigger       | manual               | git hooks                               |
| secret safety       | env vars             | capability grants + zeroize             |
| async runtime       | —                    | WASI 3 native async                     |
| cold start          | container cold start | ~1–3 ms (Cranelift JIT + AOT cache)     |

---

## workspace

This repository is a Cargo workspace. See [`crates/README.md`](crates/README.md) for a description
of each crate and how they relate.

```
crates/
  wsr            # binary — src/bin/wsr.rs + src/commands/ dispatch
  wsr-cli        # clap definitions — Cli struct, all arg types (library)
  wsr-engine     # job DAG scheduler and step orchestrator
  wsr-sandbox    # Tier 1 — Wasmtime + WASI Preview 3
  wsr-wasix      # Tier 2 — Wasmer + WASIX (transitional)
  wsr-shell      # run: step executor (bash/sh/pwsh)
  wsr-gha        # GitHub Actions provider adapter (reference impl)
  wsr-expr       # ${{ }} expression evaluator
  wsr-resolver   # action reference resolver + tier assignment
  wsr-git        # git hook management and reconciliation
  wsr-client     # HTTP client for action/component fetching
  wsr-cache      # content-addressed Wasm module cache
  wsr-fs         # filesystem utilities and VFS abstraction
  wsr-types      # shared types, traits, and errors
  wsr-tracing    # structured logging and output formatting
  wsr-bench      # benchmark suite
```

---

## contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributors must follow the [Code of Conduct](CODE_OF_CONDUCT.md).

This project targets Rust developers familiar with systems programming, WebAssembly, and CI/CD
tooling. If you're coming from the [uv](https://github.com/astral-sh/uv) or
[cargo](https://github.com/rust-lang/cargo) ecosystems, the workspace structure and contribution
model will feel familiar.

---

## architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the internal design: layer diagram, provider adapter
pattern, sandbox lifecycle, expression evaluator pipeline, and design decisions.

---

## roadmap

See [ROADMAP.md](ROADMAP.md) for the milestone plan from v0.1 foundations through v1.0 production
readiness.

---

## license

MIT
