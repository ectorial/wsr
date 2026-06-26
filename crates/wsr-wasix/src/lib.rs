//! Tier 2 sandbox: Wasmer + WASIX (transitional compatibility layer).
//!
//! Activated when Tier 1 is not yet sufficient — heavy toolchains (`rustc`, LLVM,
//! Go), complex binaries, or GHA Marketplace actions that require POSIX features
//! like `fork`/`exec` or threading.
//!
//! WASIX virtualizes POSIX syscalls *inside* the Wasm sandbox without forwarding
//! them to the host kernel. This eliminates Docker even for workloads Tier 1
//! cannot yet handle.
//!
//! # Ceiling
//!
//! Tier 2 requires binaries to be compiled to `wasm32-wasix`. It cannot execute
//! arbitrary pre-built Linux ELF binaries or pull opaque Docker images.
//! `ptrace`, raw sockets, mount, and kernel namespaces are out of scope.
//!
//! # Transitional status
//!
//! WASIX is explicitly a bridge. As `ectorial/actions` coverage grows and WASI
//! Preview 3 matures, Tier 2 workloads migrate to Tier 1. This crate is maintained
//! as long as it keeps workflows off Docker — not a day longer.
//!
//! # Promotion rule
//!
//! If any step in a job requires capabilities beyond strict WASI, the entire job
//! is promoted to Tier 2. Jobs always communicate across tiers transparently.
