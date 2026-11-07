# CLI

```mermaid
flowchart LR
    U[Invocation] --> C[Configuration and pins]
    C --> E[Local engine]
    E --> R[Progress or actionable diagnostic]
```

## Current scaffold

[Typed argument definitions](../../crates/wsr-cli/src/lib.rs) use clap;
[dispatch](../../crates/wsr/src/commands/mod.rs) selects eight handlers: `init`, `run`, `daemon`,
`list`, `inspect`, `cache`, `hook`, and `status`. These cover eleven operational leaf paths,
including `cache list/verify/purge` and `hook install/remove`.

The ten remaining placeholder leaves write `error: wsr <command>: not implemented yet` followed by a newline
to stderr, leaves stdout empty, and exits with code 1. The command name includes the nested
leaf, such as `wsr cache verify`. Those placeholders open no workflow file and initialize no configuration, hook, cache,
service, provider, or execution backend. Supplied nonexistent paths reach the same placeholder.
`inspect` instead implements the bounded source-inspection contract below.

Help and root version reporting exit with code 0 and write to stdout. Invalid syntax, omitted
required arguments, and bare `wsr`, `wsr cache`, or `wsr hook` exit with code 2 and write Clap
diagnostics or usage/help to stderr.

`run` defines an optional file and `--event`, `--dry-run`, `--verbose`, `--yes`, and `--format`.
Workflow paths are `PathBuf` values, and `--format` accepts only `human` (default) or `gha`.
These are parsed arguments, not implemented execution/planning/reporting behavior. `--yes` does
not grant access. Help text can be inspected with `cargo run -p wsr -- --help`.

## Static workflow inspection

`cargo run -p wsr -- inspect path/to/workflow.yml` reads only the explicit local UTF-8 YAML
source. The temporary built-in GHA compiler is authorized by D-031. It returns a provider-neutral
source summary, not executable IR or a signed Component compilation result.

Current supported subset:

- Optional literal workflow `name` and optional scalar/mapping manual trigger; the trigger is
  reported, never evaluated. Absence of `on` is accepted for inspection fragments.
- A nonempty `jobs` mapping with unique string identifiers, ordered lexicographically in output.
- Job `name`, a nonempty literal string or string-list `runs-on` requirement, optional static `needs`, and a nonempty ordered `steps` sequence.
- Step `name`, optional explicit `id`, and exactly one nonempty string `run` or `uses`. Scripts are not printed or retained
  in the summary; action references are reported verbatim, never resolved, fetched or opened.

Names are optional strings (null is treated as absent). Required strings retain their YAML type;
numbers and booleans are rejected. Display values escape control characters. Each successful report
states that execution compatibility is not assessed; runner names do not establish environment support.

Success exits 0 and writes a deterministic human-readable summary to stdout, with empty stderr.
Unreadable/malformed source, invalid supported shapes, unknown fields and expressions in inspected
strings exit 1, leave stdout empty and write a diagnostic to stderr. Diagnostics identify the file;
YAML decoding errors include available line/column information and structural context. Semantic
checks identify a field path (step indices are one-based). Dynamic values are explicitly unevaluated.
Clap invocation errors retain exit code 2. The output is pre-alpha, not a frozen machine protocol.

Other trigger forms, runner-group objects, matrices, conditions, environment,
permissions, defaults, services and other GHA constructs are rejected explicitly in this first slice.
The rejection is a limit of current inspection support, not a claim that the workflow is invalid GHA.
Broader inspection and canonical dogfood fixtures remain pending.

### Manual-trigger input declarations

`on: workflow_dispatch`, `on: {workflow_dispatch: null}`, an empty dispatch mapping,
and a dispatch mapping with `inputs` are supported. Other events and event sequences remain
explicitly unsupported. Missing `on`, null dispatch bodies, absent inputs and empty input mappings
remain distinct in the source summary, though the trigger-only CLI line is the same.

Inputs display in identifier order, with only explicitly declared description/type/required/default/
options metadata. Types are string, boolean, number, choice and environment; omission is preserved.
Defaults retain string/boolean/number kind and render as `Default: "pass" (not applied)`.
Numbers are rendered from the YAML number value; original numeric lexemes are not retained.
Choices render quoted options in declared order, including repetitions. Text is escaped.
Inspection neither supplies values nor prompts for required inputs, selects environments, applies
defaults or evaluates expressions.

Declarations require unique string identifiers beginning with an ASCII letter/underscore, followed
by letters/digits/underscores/hyphens. Metadata types are strict: no scalar coercion. Null metadata,
unknown fields, duplicate keys and expressions fail before stdout. Choice declarations require
nonempty options; options on other types (including omitted type) fail. Boolean defaults must be
booleans; string/choice/environment defaults must be strings. Omitted type uses the string rule
for validation but is not inserted into the summary. Numeric-type defaults retain any supported
scalar, matching the pinned schema/converter's validation boundary; this does not certify numeric
input compatibility. Choice defaults are not checked for membership in options by this subset.

Diagnostics identify the file and structural route, such as
`on.workflow_dispatch.inputs.mode.options[2]`. YAML duplicate-key/parse errors retain available
source context; semantic checks do not introduce exact source spans. The pinned official schema
and converter references and intentional source-preservation differences are recorded in the
[iteration record](../implementation/inspection-dispatch-inputs.md).

The canonical example workflows still contain unsupported fields outside these declarations.
Their manual reference runs are not evidence of WSR execution support.

### Literal runner labels

`runs-on` accepts a nonempty scalar string or nonempty list of nonempty strings.
Scalars retain `Runner: ubuntu-latest`; lists render as
`Runner labels: ["self-hosted", "linux", "x64"]`, including quoted/escaped labels.
Source form, order, exact spelling and repetitions are preserved; a one-element list stays a list.
No runner is discovered, selected or provisioned, and labels do not establish platform support.

Null/non-string values, nested lists, empty lists and blank members fail before stdout.
Runner-selector mappings are explicitly unsupported. Expressions are unevaluated and rejected,
including inside lists. Semantic diagnostics identify `jobs.build.runs-on[2]` for a second label;
decoding errors retain available source context. No normalization or label resolution occurs.

### Explicit step identities

An explicit `steps[].id` is retained in the source summary and displayed as
`Step 1 [compile]: run (Compile sources)` or `Step 2 [upload]: uses owner/action@v1`.
Steps without IDs retain their previous output; inspection never generates an ID or changes step
order. IDs do not resolve outputs or expressions, select steps, or establish runtime identities.

An ID must be a nonempty string of fewer than 100 ASCII identifier characters: a letter or
underscore first, then letters, digits, underscores or hyphens. The `__` prefix is reserved.
Explicit null/non-string values and expressions fail. Duplicate IDs within one job fail with
both the first and repeated structural step locations; exact spelling is used for comparison.
Case variants and reuse across different jobs are accepted. Semantic diagnostic indices are
one-based. Validation completes before stdout is rendered; existing graph checks remain in force.

### Static dependency inspection

`needs` accepts one literal job identifier or a list of identifiers. Omission and `[]` mean no
edges. A nonempty dependency list renders after the job's runner line as `Needs: build, test`.
Declared dependency order and repetitions are preserved. Job display order remains lexicographic,
not an execution schedule. See the [shared fixture](../../tests/fixtures/inspection/static-needs.yml).

All targets must name jobs in this workflow, using exact identifiers. Forward references, valid
diamonds, repeated edges and disconnected jobs are supported. Unknown targets are checked before
cycle detection. Self-dependencies and longer cycles fail with explicit diagnostics; a downstream
job blocked by a cycle is not reported as a cycle member. Validation uses an explicit traversal
stack rather than recursive calls, including a tested 4,096-job chain.

Malformed types, null, empty identifiers and expressions in `needs` fail before any stdout summary.
Semantic diagnostics use routes such as `jobs.test.needs` or `jobs.test.needs[2]` (one-based list
index). YAML decoding errors may report the enclosing job route plus the field/index in the error
message and line/column. Successful static validation does not establish scheduling, conditions,
matrix expansion, skipped-job behavior or workflow execution.

Inspection does not read `wsr.toml`, discover repositories, install providers, execute commands,
fetch tools/actions or mutate repository state. [CLI acceptance tests](../../crates/wsr/tests/inspection.rs)
verify real file handling, deterministic output, ordered steps, diagnostics, and malicious command
and sentinel behavior. This proves source inspection, not security enforcement for future execution.

## Skeleton implementation and verification

[The CLI skeleton plan](../archive/cli-skeleton.md) records the implemented slice, using uv's
argument/application separation as a reference. It retains the existing commands, adds typed
arguments and CLI contract tests, and gives each operational leaf consistent unimplemented
output. The final product interface remains open.

[Parser tests](../../crates/wsr-cli/tests/cli.rs) verify the command graph, options/defaults,
required inputs, format validation, and OS-string paths.
[Executable tests](../../crates/wsr/tests/cli.rs) run every remaining placeholder leaf, check help/version and error
codes, exercise nonexistent paths, and compare repository sentinels before/after invocation.
The smoke contract runs with `cargo xtask test smoke_remaining_operational_subcommands`; complete
standard verification uses `cargo xtask ci`. These checks validate the skeleton, not workflow
execution, isolation, or GitHub Actions compatibility.

## Accepted design and open interface

Local inspection, execution and debugging are independent of GitHub scheduling. Hosted reuse
and reporting are optional integrations; cached inputs enable disconnected execution.
Planned `wsr init` creates root `wsr.toml` for parameters, providers, and invocation defaults,
with GitHub Actions as the primary provider and local execution as the default path. Normal
commands use repository defaults; explicit provider/target options in the CLI proposal override
them. Multiple providers remain supported. See [configuration](../config/README.md) and D-023
in [PLAN.md](../context/decisions.md). These settings and proposed flags are not implemented by the skeleton.

Final command structure, selection, event binding, and output formats are not settled.
A planning-only path must not execute repository commands. Effective permissions and unsupported
requirements must be visible before execution.

`plan` exists in preserved prototype work, not the current command enum. `--allow '*'` is proposed
syntax for an explicit broad-access profile, not an implemented option. There is no `actions`
subcommand. Hooks/daemon behavior is not the primary product commitment.

See [PLAN.md](../context/decisions.md) and [the security model](../security/README.md).
