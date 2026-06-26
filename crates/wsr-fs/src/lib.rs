//! Filesystem utilities for wsr.
//!
//! # Responsibilities
//!
//! - **Atomic writes** — `write(path, content)` uses `tmpfile → rename()` to
//!   guarantee readers never see a partial write.
//! - **VFS abstraction** — a thin trait over real and in-memory filesystems,
//!   used by the sandbox to enforce capability grants without spawning subprocesses.
//! - **Path helpers** — workspace root detection, `.git/` location, hook path
//!   resolution.
//!
//! All other crates that touch the filesystem should go through this crate, not
//! `std::fs` directly, so that the VFS seam remains testable.
