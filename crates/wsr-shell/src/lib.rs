//! Shell step executor for `run:` directives.
//!
//! Handles the execution of `run:` steps inside the wsr sandbox, supporting
//! `bash`, `sh`, and `pwsh` shells. Shell binaries are resolved from the Wasm
//! component catalog when running in Tier 1; on Tier 2 they are resolved via
//! WASIX path lookup.
//!
//! # Responsibilities
//!
//! - **Script preparation** — injects `env:` variables, applies `working-directory:`,
//!   and wraps the script in the appropriate shell invocation.
//! - **Exit code propagation** — maps shell exit codes to wsr step outcomes,
//!   respecting `continue-on-error:` and `if:` conditions.
//! - **`set -eo pipefail`** — applied by default for `bash` steps, matching
//!   GitHub Actions runner behaviour.
