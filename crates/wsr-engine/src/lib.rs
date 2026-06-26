//! Job DAG scheduler, step orchestrator, and matrix expansion engine.
//!
//! The engine is the runtime core of wsr. It takes a [`WorkflowIR`], resolves
//! the execution plan, and drives steps through the appropriate sandbox tier.
//!
//! # Responsibilities
//!
//! - **DAG scheduling** — topological sort of `needs:` dependencies; runs
//!   independent jobs in parallel where the DAG allows.
//! - **Matrix expansion** — expands `strategy.matrix` axes into individual job
//!   instances, honouring `include`/`exclude` rules.
//! - **Step runner** — dispatches each step to `wsr-shell` (`run:`) or
//!   `wsr-resolver` + `wsr-sandbox` / `wsr-wasix` (`uses:`).
//! - **Context threading** — threads `outputs:` from upstream jobs into
//!   downstream `needs.<job>.outputs` context maps.
//! - **`if:` evaluation** — evaluates step and job conditions via `wsr-expr`
//!   before dispatch; honours `continue-on-error:` and `timeout-minutes:`.
//! - **Exit code propagation** — maps final job results to process exit codes
//!   so git hook shims can block or allow the git operation.
//!
//! [`WorkflowIR`]: wsr_types::WorkflowIR
