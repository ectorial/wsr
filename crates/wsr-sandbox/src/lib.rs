//! Tier 1 sandbox: Wasmtime + WASI Preview 3.
//!
//! The default execution environment. Every job that can run here, does. Provides
//! ~1–3 ms cold starts via Cranelift JIT and AOT module caching.
//!
//! # Per-step lifecycle
//!
//! Each step spawns a fresh, isolated Wasmtime instance. Instances are cheap
//! because compiled modules are cached by SHA-256 and deserialized, not
//! recompiled. Memory is zeroed on drop.
//!
//! # Capability model
//!
//! Steps receive only the capabilities they need — nothing more:
//!
//! | Capability | Default | Override |
//! |---|---|---|
//! | `preopened_dirs` | workspace root | `wsr.json` |
//! | `allowed_hosts` | none | `sandbox.allowed_hosts` in `wsr.json` |
//! | `env_vars` | declared `env:` keys only | step definition |
//! | `secrets` | declared secret names | injected, never written to disk |
//!
//! A step that reaches outside its grants receives an immediate, human-readable
//! error — not a silent failure minutes later on remote CI:
//!
//! ```text
//! capability denied: net → registry.example.com
//! hint: add to sandbox.allowed_hosts in wsr.json
//! ```
//!
//! # WASI Preview 3
//!
//! WASI 3's native async layer means steps with async I/O do not block the
//! Wasmtime thread — the runtime drives the async executor directly without
//! polling adapters or callback wrappers.
