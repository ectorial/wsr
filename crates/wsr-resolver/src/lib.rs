//! Action reference resolver and execution tier assignment.
//!
//! The resolver is the primary integration point between the engine and the rest
//! of the ecosystem. It maps every `uses:` reference to a concrete Wasm component
//! and decides which execution tier to use.
//!
//! # Resolution order for `uses: owner/action@ref`
//!
//! 1. Check local content-addressed cache (keyed by resolved SHA digest)
//! 2. Check `ectorial/*` component registry for a native Tier 1 equivalent → assign [`ExecutionTier::Vault`]
//! 3. Check for a WASIX-compatible build → assign [`ExecutionTier::Workshop`]
//! 4. No match → emit advisory warning; leave step flagged as unresolved
//!
//! The resolver **never** silently falls back to Docker. If no Wasm path exists,
//! the step is flagged — not silently containerized.
//!
//! # Action kinds
//!
//! | Kind | Strategy |
//! |---|---|
//! | JS / TS | Fetch `action.yml` + `index.js`, compile via Javy → `.wasm`, cache by SHA |
//! | Composite | Inline step expansion into parent job, no Wasm overhead |
//! | Docker | Advisory warning; fallback or skip in strict mode |
//! | Local path (`./actions/my-action`) | Resolve relative to workspace root |
//!
//! [`ExecutionTier::Vault`]: wsr_types::ExecutionTier::Vault
//! [`ExecutionTier::Workshop`]: wsr_types::ExecutionTier::Workshop
