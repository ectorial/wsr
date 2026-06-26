//! Provider-agnostic `${{ }}` expression evaluator.
//!
//! Implements the full expression surface used by GitHub Actions and normalized
//! for use across all provider adapters. The evaluator is decoupled from any
//! specific provider — the context map is supplied by the active provider adapter
//! at evaluation time.
//!
//! # Supported contexts (GitHub adapter)
//!
//! `github.*` · `env.*` · `runner.*` · `secrets.*` · `needs.*` · `steps.*` · `inputs.*`
//!
//! # Supported functions
//!
//! `contains` · `startsWith` · `endsWith` · `format` · `join` · `toJSON` ·
//! `fromJSON` · `success` · `failure` · `always` · `cancelled` · `hashFiles`
//!
//! # Pipeline
//!
//! ```text
//! raw string  →  lexer  →  parser  →  evaluator  →  resolved string
//!                                          ↑
//!                                     context map
//!                                  (from provider adapter)
//! ```
