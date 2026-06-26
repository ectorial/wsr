//! Git hook management and stateless hook reconciliation.
//!
//! # Responsibilities
//!
//! - **Shim generation** — produces executable shell scripts for `.git/hooks/`
//!   that delegate to `wsr run --hook <name>`.
//! - **Reconcile** — `reconcile(desired, actual)` computes the diff between the
//!   desired hook map (derived from the active provider's `trigger_map()`) and the
//!   installed shims, then atomically applies adds, updates, and removals.
//! - **Manifest embedding** — each shim carries a comment header encoding the
//!   desired state (`# wsr:managed provider=github workflows=ci.yml …`). This is
//!   the only persistent state; there is no lock file.
//! - **Git event hooks** — `post-checkout`, `post-merge`, and `post-rewrite`
//!   trigger a reconcile automatically when the branch or workflow files change.
//!
//! # No coordination logic
//!
//! Git holds the filesystem lock during pull, checkout, and rebase. The daemon only
//! fires on manual edits between git events. These windows never overlap — `rename()`
//! atomicity is the only guarantee needed.
