# wsr roadmap

Status legend: `done` · `in progress` · `planned` · `future`

Layers build sequentially because adding actions before the runtime can reliably execute them
wastes everyone's time. Each milestone gates the next.

---

## v0.1 — execution core `in progress`

Prove the runtime works end-to-end. The core loop: init, parse a workflow, execute a step in the
sandbox, block the push on failure. GitHub Actions as the reference provider.

**workspace skeleton**

- [x] cargo workspace with `wsr-*` crates
- [x] `wsr-types` — `WorkflowProvider` trait, `WorkflowIR`, `TriggerEvent`, `GitHook`, `ExecutionTier`
- [x] `wsr-cli` — clap v4 skeleton with all planned subcommands

**CLI**

- [ ] `wsr init` — scan workflows, generate `wsr.json`, install git hook shims
- [ ] `wsr run` — execute workflows matching `workflow_dispatch`
- [ ] `wsr run <file>` — run specific workflow, prompt if no dispatch trigger
- [ ] `wsr run --event <n>` — force a specific trigger event
- [ ] `wsr run --dry-run` — print execution plan, run nothing
- [ ] `wsr run --verbose` — expose expression eval, sandbox grants, context dump
- [ ] `wsr run --yes` — skip interactive prompts (for scripted use)

**configuration (`wsr-types`)**

- [ ] `wsr.json` generation on `wsr init` with `$schema`, provider, sandbox defaults
- [ ] `serde_json` parsing — no extra crates
- [ ] JSON Schema published at `https://wsr.dev/schema/wsr.json`
- [ ] VS Code IntelliSense via `$schema` field

**provider adapter — github actions (`wsr-gha`)**

- [ ] `WorkflowProvider` impl — `parse()`, `context()`, `trigger_map()`
- [ ] GitHub Actions YAML parser — `serde-yaml`
- [ ] `on:` trigger → git hook mapping
- [ ] `github.*` context builder — `event_name`, `ref`, `sha`, `actor`, `repository`
- [ ] `env.*`, `runner.*`, `secrets.*` contexts

**GHA compat — `run:` steps**

- [ ] `run:` with `bash` / `sh`
- [ ] `run:` with `pwsh`
- [ ] `env:` at step and job level
- [ ] `if:` conditions on steps and jobs
- [ ] `continue-on-error:`
- [ ] `working-directory:`

**GHA compat — expression engine (`wsr-expr`)**

- [ ] `${{ }}` evaluation — lexer + parser + evaluator
- [ ] `fromJSON` / `toJSON`
- [ ] string functions — `contains`, `startsWith`, `endsWith`, `format`
- [ ] status functions — `success()`, `failure()`, `always()`, `cancelled()`

**GHA compat — jobs**

- [ ] single job execution
- [ ] `needs:` DAG — sequential and parallel
- [ ] matrix expansion — basic axes
- [ ] `outputs:` between jobs

**Tier 1 sandbox (`wsr-sandbox`)**

- [ ] Wasmtime engine with Cranelift JIT
- [ ] WASI Preview 3 — native async layer
- [ ] one instance per step, killed after completion
- [ ] WASI capability grants — preopened dirs, env vars
- [ ] `allowed_hosts` network enforcement
- [ ] secret injection — zeroize on drop, never written to disk
- [ ] AOT module cache — `wasmtime::Module::serialize` + SHA-256 pin (`wsr-cache`)

**action resolver (`wsr-resolver`)**

- [ ] JS/TS action fetch from GitHub (tags, SHAs)
- [ ] SHA pinning — resolve mutable refs to immutable SHAs before download
- [ ] Javy compile JS/TS → Wasm
- [ ] cache compiled `.wasm` by content hash (`wsr-cache`)
- [ ] `actions/checkout@v4`
- [ ] `actions/setup-node@v4`
- [ ] `actions/cache@v4`

**git hook management (`wsr-git`)**

- [ ] `post-checkout` — resync on branch switch
- [ ] `post-merge` — resync after pull
- [ ] `post-rewrite` — resync after rebase/amend
- [ ] atomic shim writes — `write tmpfile → rename()`
- [ ] manifest comment in shim — no lock file
- [ ] named warning on workflow deletion

**observability (`wsr-tracing`)**

- [ ] structured step logs via `tracing`
- [ ] per-step timing
- [ ] sandbox violation events
- [ ] exit code propagation to git hook
- [ ] GitHub Annotations format output (`--format=gha`)

---

## v0.2 — compatibility depth `planned`

Close the remaining GitHub Actions surface area.

**GHA compat**

- [ ] composite actions — inline step expansion
- [ ] reusable workflows — `uses: ./.github/workflows/shared.yml`
- [ ] `workflow_call` trigger
- [ ] matrix `include` / `exclude`
- [ ] `strategy.fail-fast`
- [ ] `concurrency` groups — cancel-in-progress
- [ ] `services:` containers — Tier 2 (WASIX) shim with warning
- [ ] Docker-based actions — capability-wrapped Tier 2 shim
- [ ] `secrets: inherit`
- [ ] `permissions:` blocks — respected as capability hints
- [ ] `timeout-minutes:` per step and job

**action resolver**

- [ ] `actions/upload-artifact@v4` / `download-artifact`
- [ ] `actions/setup-python`, `setup-go`, `setup-java`
- [ ] local actions — `uses: ./actions/my-action`
- [ ] private repo actions (with token)

**Tier 1 sandbox**

- [ ] per-step memory limits
- [ ] per-step CPU time limits (Cranelift fuel)
- [ ] network proxy — DNS allowlist + TLS inspection
- [ ] async step execution via WASI 3 — concurrent I/O within a step

**CLI**

- [ ] `wsr list` — show all workflows and hook mappings
- [ ] `wsr inspect <file>` — parse and pretty-print, validate expressions
- [ ] `wsr cache list / verify / purge`
- [ ] `wsr hook install / remove`

---

## v0.3 — Tier 2 WASIX + heavy toolchains `planned`

First real Tier 2 workloads. Validates the WASIX sandbox against `rustc`, LLVM, and Go.

**Tier 2 sandbox (`wsr-wasix`)**

- [ ] Wasmer + WASIX integration
- [ ] `fork`/`exec` virtualisation inside the sandbox
- [ ] threading support
- [ ] socket virtualisation — no host kernel forwarding
- [ ] `wasm32-wasix` toolchain resolution
- [ ] Tier promotion rule — job upgrades to Tier 2 if any step requires it
- [ ] transparent cross-tier `outputs:` propagation

**action catalog**

- [ ] `ectorial/setup-rust` (Tier 2 — `rustc` + `cargo`)
- [ ] `ectorial/setup-go` (Tier 2 — Go toolchain)
- [ ] `ectorial/setup-python` (Tier 2 — CPython WASIX build)

---

## v0.4 — daemon and DX `planned`

The authoring experience. Fast feedback while writing workflows.

- [ ] `wsr daemon` — persistent file watcher via `notify` crate
- [ ] auto-resync hooks on workflow file edits
- [ ] debounce — 300 ms, coalesce rapid saves
- [ ] IPC socket for `wsr status` to query daemon state
- [ ] `wsr daemon install` — register as launchd / systemd service
- [ ] `wsr status` — active hook map, daemon state, last sync
- [ ] hot reload — recompile changed Wasm modules without restarting

---

## v0.5 — gitlab ci adapter `planned`

First non-GitHub provider. Validates the adapter pattern against a real alternative syntax.

- [ ] `WorkflowProvider` impl for GitLab CI (`wsr-gitlab`)
- [ ] `.gitlab-ci.yml` parser
- [ ] GitLab CI trigger → git hook mapping (`push`, `merge_request`)
- [ ] `CI_*` variable context normalization → `WorkflowIR`
- [ ] `stage:` and `needs:` DAG execution
- [ ] GitLab-specific `rules:` evaluation
- [ ] `wsr.json` — `"provider": "gitlab"`

---

## v0.6 — bitbucket pipelines adapter `planned`

- [ ] `WorkflowProvider` impl for Bitbucket Pipelines (`wsr-bitbucket`)
- [ ] `bitbucket-pipelines.yml` parser
- [ ] Bitbucket trigger → git hook mapping
- [ ] `BITBUCKET_*` variable context normalization → `WorkflowIR`
- [ ] `step:` execution model
- [ ] `wsr.json` — `"provider": "bitbucket"`

---

## v0.7 — component registry + signing `planned`

Production-grade supply chain. Matches the `ectorial/wit` and `ectorial/actions` ecosystem.

- [ ] Component registry client — fetch, verify, and cache signed Wasm components
- [ ] SHA pinning for registry components (content-addressed, not tag-based)
- [ ] Signature verification before execution — unsigned components rejected in Tier 1
- [ ] `wsr publish` — build and publish a Wasm component to the registry
- [ ] `ectorial/wit` WIT interface enforcement at resolution time
- [ ] Coverage: 80% of top-100 GHA Marketplace steps without Docker

---

## v1.0 — production ready `future`

Stable APIs, infrastructure-agnostic runner deployment, full cross-provider coverage.

- [ ] Stable `WorkflowProvider` trait API (semver-stable)
- [ ] Stable `wsr.json` schema (published, versioned)
- [ ] All `wsr-*` crates at `1.0` with documented stability guarantees
- [ ] `wsr run --watch` — re-run on file save (TDD-style workflow authoring)
- [ ] Remote cache — share compiled `.wasm` artifacts across a team via S3/R2
- [ ] VS Code extension — inline step results, expression hover evaluation
- [ ] Native Rust actions — compile Rust-based actions directly, skip Javy

---

## performance `future`

Once compatibility is proven across providers, optimize the hot path.

- [ ] AOT compilation on `wsr init` — pre-compile all action Wasm at install time
- [ ] Shared Wasmtime `Engine` across steps — amortize JIT cost
- [ ] Parallel step execution where DAG allows
- [ ] Incremental expression caching — memoize pure `${{ }}` eval results
- [ ] Cranelift optimization flags for release builds
- [ ] Benchmark suite (`wsr-bench`) — compare step startup time vs `act` + Docker

---

## non-goals

Things `wsr` will deliberately not do:

- replace remote CI — `wsr` is a local pre-flight, not a CI server
- run Docker-based actions natively — the security boundary is the point
- support `act`'s `--platform` flag — no container runtime dependency, ever
- auto-update workflows — `wsr` reads workflows, never writes them
- lock developers to a single CI provider — the adapter pattern is a first-class design goal
- Windows/macOS Tier 2 — deferred beyond v1.0
- GUIs or managed hardware
