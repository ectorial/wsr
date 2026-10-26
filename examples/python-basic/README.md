# Python pass/failure reference

A single standard-library unittest makes the CI outcome deterministic. Python 3.11+ is required;
there are no package dependencies, installation steps or lockfiles. GitHub selects Python 3.13;
local evidence records the interpreter actually used rather than claiming identical environments.

From this directory:

```sh
python3 -m unittest discover -s tests -v
WSR_EXAMPLE_MODE=fail python3 -m unittest discover -s tests -v
```

The first command runs one test and succeeds. The second exits 1 with `test_addition` and
`controlled WSR example assertion`. Return to normal mode to reproduce repaired success.

The canonical [reference workflow](../../.github/workflows/example-python.yml) executes the same
command directly on GitHub with a typed `mode` input (`pass` or `fail`). A fail-mode GitHub run is
expected to have a failed conclusion. It is evidence only when the test assertion caused the
failure, not when setup or checkout failed. See [recorded reference runs](../../tests/reference-runs.md).

WSR translation and execution of this workflow are **pending**. `cargo xtask corpus` tests the native reference in
disposable copies; it does not invoke the WSR runner or establish isolation.
