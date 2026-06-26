//! HTTP client for fetching actions and Wasm components.
//!
//! # Responsibilities
//!
//! - **Action fetch** — downloads `action.yml` and associated JS/TS entry points
//!   from GitHub at a pinned SHA, not a mutable tag.
//! - **Component registry** — queries the `ectorial/*` component registry for
//!   native Tier 1 equivalents of marketplace actions.
//! - **SHA pinning** — resolves mutable refs (`@v4`, `@main`) to immutable SHAs
//!   before any download, so the cache key is always content-stable.
//! - **Retry + backoff** — handles transient network failures transparently.
//!
//! Callers should always go through `wsr-resolver` rather than this crate
//! directly; the resolver owns the decision of which source to use.
