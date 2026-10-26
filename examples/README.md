# WSR dogfood workflows

Canonical example workflows live in `.github/workflows/` and have **only** a
`workflow_dispatch` trigger. They are intended to run manually on GitHub and,
once GHA-to-IR translation and execution are implemented, through WSR using the
same files. No copied local-only workflow or catalog registration is required.

| Workflow | Behavior | Current local reference |
| --- | --- | --- |
| [example-shell.yml](../.github/workflows/example-shell.yml) | One shell step with controlled pass/failure; no checkout or setup actions | `cargo test -p wsr-gha --test workflow_examples --locked` |
| [example-python.yml](../.github/workflows/example-python.yml) | Checkout, Python setup, and a controlled unittest pass/failure | `cargo test -p wsr --test corpus --locked`; [Python guide](python-basic/README.md) |

Both accept `mode=pass` or `mode=fail`. A native fail-mode run must conclude failed
because of the controlled workload error, not setup problems. The shell failure
prints `controlled WSR shell failure` and exits 1; pass prints
`WSR shell example passed` and exits 0. Neither workflow publishes, deploys or needs secrets.

On GitHub, open Actions, select the example workflow and use **Run workflow** with
the desired mode. Record the revision, inputs and run link in
[reference-runs.md](../tests/reference-runs.md) when using a run as evidence.

The Rust workflow guard checks every `example-*.yml`/`example-*.yaml` file for manual-only
triggers and nonempty jobs. Its shell test runs the actual YAML script with an explicitly
bound native environment variable. This is a native reference, not expression evaluation,
GHA-to-IR translation, WSR execution, or sandbox qualification.

Add examples with a useful behavior and explicit expected outcomes. Grow job dependencies,
outputs, matrices, artifacts and services one behavior at a time as needed. Test translation
through the real translator when it exists, then execution through the real WSR CLI/engine
when supported; see [Testing WSR](../tests/README.md). Keep ordinary CI independent of the
manual payloads so examples cannot recursively run the entire WSR suite.
