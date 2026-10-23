# Owned WSR examples

These small projects demonstrate intended CI use cases and provide independent native references.
They are not evidence that WSR can execute them. Begin with [python-basic](python-basic/README.md).

Canonical GitHub reference workflows live in `.github/workflows/example-*.yml` and use manual
`workflow_dispatch` triggers only. They can run on GitHub; manual dispatch is not local-only.
They never publish or deploy. Normal WSR development runs the corresponding native commands
locally without dispatching GitHub workflows. Future WSR acceptance drivers will consume the same
workflows once the required behavior exists.

Add examples with a defined behavior, pass/failure expectations and prerequisites, then register
cases in `tests/cases.toml`. Add outputs, artifacts, matrices and services incrementally with their
product support rather than creating a large disabled suite.
