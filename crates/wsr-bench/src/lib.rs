//! Benchmark suite for wsr.
//!
//! Measures the performance characteristics that matter for a git-hook-integrated
//! CI runner, where latency directly affects developer feedback loops.
//!
//! # Benchmarks
//!
//! - **Step cold start** — time from `wsr run` invocation to first step executing,
//!   broken down by tier (Tier 1 Wasmtime vs Tier 2 WASIX).
//! - **Cache hit rate** — percentage of Wasm modules served from the SHA-256
//!   content cache vs recompiled.
//! - **Expression evaluation** — throughput of the `${{ }}` evaluator across
//!   realistic context map sizes.
//! - **Hook reconcile** — time to scan, diff, and atomically rewrite all hooks
//!   on branch switch.
//! - **vs act** — comparative benchmark against `act` + Docker for equivalent
//!   workflow runs (requires Docker daemon on the bench host).
