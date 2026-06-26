//! Content-addressed Wasm module cache.
//!
//! # Responsibilities
//!
//! - **Storage** — persists compiled `.wasm` modules on disk, keyed by SHA-256
//!   digest of the source artifact (action source or compiled output).
//! - **Lookup** — `get(digest)` returns a cached module if present, enabling
//!   Wasmtime to skip re-compilation via `Module::deserialize`.
//! - **Invalidation** — cache entries are immutable once written; a new digest
//!   produces a new entry. There is no TTL-based eviction.
//! - **Management CLI surface** — provides the data layer backing `wsr cache list`,
//!   `wsr cache verify`, and `wsr cache purge`.
//!
//! The cache directory defaults to `$CARGO_HOME/wsr/cache` and is shared across
//! all repositories on the same machine.
