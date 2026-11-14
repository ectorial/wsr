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
source. The temporary built-in GHA compiler is authorized by D-031. It returns a temporary GHA-derived
source summary (D-033), not executable IR or a signed Component compilation result.

Current supported subset:

- Optional literal workflow `name` and required `on` events (scalar, sequence or mapping); events are
  reported in source order, never evaluated or subscribed to. A workflow without `on` is invalid
  (``missing field `on` ``), as the pinned root schema requires it. See [trigger events](#trigger-events).
- Optional workflow `run-name` source text, reported but never evaluated; see [run names](#unevaluated-run-names).
- A nonempty `jobs` mapping with unique string identifiers, ordered lexicographically in output.
- Job `name`, a nonempty string or string-list `runs-on` requirement, optional static `needs`, and a nonempty ordered `steps` sequence.
- Or a reusable-workflow call job: `uses` with optional `with` and `secrets`, never loaded; see [workflow calls](#reusable-workflow-calls).
- Step `name`, optional explicit `id`, and exactly one nonempty string `run` or `uses`. Scripts are not printed or retained
  in the summary; action references are reported verbatim, never resolved, fetched or opened.

Names are optional strings (null is treated as absent). Scalars keep their YAML type: `5` and `"5"`
are different, and the converted text of a number or boolean is never rendered. The pinned reader
converts an unmatched null, boolean or number to a string and validates that string, so a
conversion the schema accepts (a numeric `name`, `run`, `working-directory`, `runs-on` label,
dispatch `description` or choice option, a boolean step `id`, a null `env` or `with` value) is
unsupported (`non-string value is not inspected yet`), while one the schema or the runner rejects (a
numeric `shell`, `uses`, dispatch `type`, `required` or typed `default`, a numeric job ID or step `id`)
stays invalid. See [scalar conversion](../implementation/inspection-scalar-conversion.md).
Display values escape control characters. Each successful report
states that execution compatibility is not assessed; runner names do not establish environment support.

Success exits 0 and writes a deterministic human-readable summary to stdout, with empty stderr except the 500 KB size warning below.
Unreadable/malformed source, invalid supported shapes, unknown fields and expressions in inspected
strings exit 1, leave stdout empty and write a diagnostic to stderr. Diagnostics identify the file;
YAML decoding errors include available line/column information and structural context. A location
follows the parser and may be absent: custom value diagnostics point at the start of the enclosing mapping,
unknown-field diagnostics at the key, and other decoding errors (for example a wrong value type) and YAML syntax
errors keep the parser's own position. Semantic
checks identify a field path (step indices are one-based). Dynamic values are explicitly unevaluated.
Clap invocation errors retain exit code 2. The output is pre-alpha, not a frozen machine protocol.

The source must be a regular file (symlinks are followed); directories, FIFOs and devices such as
`/dev/zero` exit 1 with a `not a regular file` diagnostic and are never read. WSR imposes no size
limit (D-032). When a successfully inspected file exceeds GitHub's documented 500 KB limit, read by WSR
as 512,000 bytes (GitHub does not define the unit), one line is written to stderr before the unchanged report, and the exit code stays 0:
`warning: wsr inspect: ci.yml: 512001 bytes exceeds GitHub's documented 500 KB (512000-byte) workflow
file limit; GitHub would not run this workflow`. A closed stderr does not stop the report. Because workflow content is untrusted, diagnostics are written with control characters
and other unprintable characters escaped (`\n`, `\t` or `\u{…}`), so no raw terminal escape sequence, bidi override or
isolate, or zero-width character reaches stderr (`jobs.b\u{202e}: invalid job identifier 'b\u{202e}'`).
Every character `char::escape_debug` escapes is escaped, including combining marks; quotes and
backslashes stay as written. If stdout is closed early (for example
`wsr inspect big.yml | head -1`), the command stops writing and exits 0 quietly, without a panic.

A key the pinned schema recognizes but WSR does not inspect reads ``unsupported field `strategy`; this field is
not inspected yet`` (the key is recognized, nothing is claimed about its value); an unknown key keeps the
parser's ``unknown field `x`, expected one of …`` text. Event names and dispatch fields that are invalid read
`unknown event 'x'` and ``unknown field `x` ``. B-20 intentionally changed this presentation; categories and
routes are unchanged ([record](../implementation/inspection-diagnostic-text.md)).

Event configuration other than `workflow_dispatch` inputs, runner-group objects, matrices,
services and other GHA constructs are rejected explicitly in this first slice.
The rejection is a limit of current inspection support, not a claim that the workflow is invalid GHA.
Broader inspection remains pending.

### Trigger events

Each `on` event prints `Trigger: <event> (not evaluated)` in source order, from a scalar
(`on: workflow_call`), a sequence (`on: [push, workflow_dispatch]`) or a mapping whose
non-dispatch bodies are null or `{}` (`on: {workflow_call: null, push: {}}`). Every event name of the
[pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
is accepted; an unknown name is invalid (`on[2]: unknown event 'bogus'`). Other non-empty bodies
(type filters, `workflow_call` inputs, secrets and outputs) and any `schedule` sequence are
unsupported (`on.release: event configuration is not inspected yet`), as is a repeated sequence
entry or an empty sequence or mapping (`on: an empty event list is not inspected yet`). A body on a
null-only event such as `create` (`expected null`), a null `schedule`, other body shapes and `null`
are invalid. No
event is subscribed to, matched or delivered.

### Event filters

`push`, `pull_request` and `pull_request_target` bodies with branch, tag or path filters print the
filters in source order, each as a pattern list (a single string is one pattern), never matched:

```text
Trigger: push (not evaluated)
  Filters (not matched):
    branches: ["main"]
```

Per the pinned schema, `push` takes `branches`, `branches-ignore`, `tags`, `tags-ignore`, `paths` and
`paths-ignore`; the pull request events take the same without the tag filters, plus `types`, which is
unsupported (``on.pull_request.types: unsupported field `types`; …``). Any other key is invalid
(``on.pull_request: unknown field `tags` ``). A pattern must be a non-empty string; filters have no
expression context, so `${{` text is invalid, as are null, nested values and a filter given with its
`-ignore` counterpart (``on.push.branches-ignore: cannot be combined with `branches` ``). Numbers and
booleans, which the pinned reader converts to strings, are unsupported. An empty list is reported as
`[]`.

### Manual-trigger input declarations

`on: workflow_dispatch`, `on: {workflow_dispatch: null}`, an empty dispatch mapping,
and a dispatch mapping with `inputs` are supported, alone or among other events. Null dispatch bodies, absent inputs and empty input mappings
remain distinct in the source summary, though the trigger-only CLI line is the same; inputs print
under the `workflow_dispatch` line.

Inputs display in identifier order, with only explicitly declared description/type/required/default/
options metadata. Types are string, boolean, number, choice and environment; omission is preserved.
Defaults retain string/boolean/number kind and render as `Default: "pass" (not applied)`.
Numbers are rendered from the YAML number value; original numeric lexemes are not retained.
Choices render quoted options in declared order, including repetitions. Text is escaped.
Inspection neither supplies values nor prompts for required inputs, selects environments, applies
defaults or evaluates expressions.

Declarations require unique string identifiers beginning with an ASCII letter/underscore, followed
by letters/digits/underscores/hyphens. Metadata types are strict, except that a null, boolean or number
`description` or choice option and a null `default` of a non-boolean input are valid GHA that is
unsupported here. Other null metadata, unknown fields, duplicate keys and expressions fail before stdout. Choice declarations require
nonempty options; options on other types (including omitted type) fail. Boolean defaults must be
booleans; string/choice/environment defaults must be strings. Omitted type uses the string rule
for validation but is not inserted into the summary. Numeric-type defaults retain any supported
scalar, matching the pinned schema/converter's validation boundary; this does not certify numeric
input compatibility. Choice defaults are not checked for membership in options by this subset.

Diagnostics identify the file and structural route, such as
`on.workflow_dispatch.inputs.mode.options[2]` or `on.workflow_dispatch.inputs.bad id: invalid
input identifier 'bad id'`. Input identifiers repeated ignoring case are invalid, as in the pinned
reader (`on.workflow_dispatch.inputs.MODE: duplicate input identifier 'MODE'; already defined as
'mode'`). YAML duplicate-key/parse errors retain available source context;
semantic checks do not introduce exact source spans. The pinned official schema
and converter references and intentional source-preservation differences are recorded in the
[iteration record](../implementation/inspection-dispatch-inputs.md).

The canonical example workflows inspect (their reports are the dogfood goldens). Their manual
reference runs are not evidence of WSR execution support.

### Unevaluated run names

An optional workflow `run-name` must be a nonempty string; null, booleans, numbers, lists,
mappings and blank text fail before stdout with a `run-name` diagnostic (a null, boolean or number is unsupported, the rest invalid). Its decoded text is
reported on the line after the workflow name, with control characters escaped:

```text
Run name: Local reference (literal)
Run name: Shell reference — ${{ inputs.mode }} (unevaluated; expression syntax not validated)
```

Text containing a `${{` opening fence is classified as unevaluated and retained whole, including
multiple, multiline or unterminated fences. This is not an expression parser: no syntax is validated,
no context or input is looked up and no default replaces `inputs.mode`. Omission adds no line.
The [pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
defines `run-name` as a string with `github`, `inputs` and `vars` contexts; WSR does not bind them.
Other fields keep their literal-only expression rejection.

### Requested permissions

Workflow-level and job-level `permissions` accept the `read-all`/`write-all` shorthands or a
mapping (possibly empty) from a scope in the
[pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
to `read`, `write` or `none`; `id-token` and `copilot-requests` take only `write`/`none`,
`models` and `vulnerability-alerts` only `read`/`none`. The workflow request follows the trigger
lines, and a job's request follows its runner and needs lines, with scopes sorted by name:

```text
Permissions: read-all (requested; not granted)
Permissions (requested; not granted):
  contents: read
Permissions: {} (requested; not granted)
```

These are source requests only. Inspection grants no token, does not combine a job's request with
the workflow's or with repository defaults, and does not enforce any permission. Null and other
scalars, sequences, unknown or non-string scopes, levels outside a scope's set and duplicate scopes
fail before stdout (`permissions.secrets: unknown permission scope 'secrets'`,
`jobs.b.permissions.models: expected read or none`). The pinned schema gives `permissions` no
expression context, so `${{` text fails as an unevaluated expression the field does not allow.

### Job timeouts and step shells

A job's `timeout-minutes` and a run step's `shell` are reported as source data on lines after the
job's runner, needs and permissions lines and after the step line:

```text
  Timeout minutes: 5 (not applied)
  Timeout minutes: ${{ inputs.minutes }} (unevaluated; expression syntax not validated; not applied)
  Step 1: run
    Shell: bash (not selected)
```

A timeout must be a finite positive YAML number (a WSR subset rule; the schema sets no minimum), rendered from the number value, or text with a
`${{` fence, which stays unevaluated as for `run-name`. A shell must be a nonempty string on a
`run` step; the [pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
gives `shell` no expression context, so `${{` text fails, as does `shell` on a `uses` step.
Literal shell text (here and in `defaults.run`) must be a built-in shell (`bash`, `pwsh`, `python`,
`sh`, `cmd`, `powershell`, any case, as the command before the first space with no arguments) or
contain a `{0}` placeholder, since the runner rejects anything else
(`shell: unknown shell 'perl'; expected bash, pwsh, python, sh, cmd, powershell or a command
containing {0}`). A literal `uses` must be `./path` (or `.\path`), `docker://image` or `owner/repo[/path]@ref`
(`uses: invalid action reference 'actions/checkout'; …`); references are checked for shape only,
and a `$/…` self-repository reference is unsupported.
No timeout is applied and no shell is selected, located or invoked. Job `cancel-timeout-minutes`
remains unsupported.

### Continue-on-error and step timeouts

Job and step `continue-on-error` and step `timeout-minutes` print after the job's timeout and after a
step's condition, each marked `(not applied)`:

```text
  Continue on error: ${{ matrix.experimental }} (unevaluated; expression syntax not validated; not applied)
  Step 1: run
    Continue on error: true (not applied)
    Timeout minutes: 1.5 (not applied)
```

`continue-on-error` takes a boolean or `${{` text (`boolean-strategy-context` and
`step-continue-on-error` have expression contexts); a number, null, the string `'true'` or any other
shape is invalid (`expected a boolean`). A step timeout follows the job timeout rules: a finite
positive number or `${{` text, a non-positive number unsupported, anything else invalid. A failing
job is never tolerated and no step is timed.

### Step environment, action inputs and working directories

Step `env` (any step), `with` (`uses` steps only) and `working-directory` (`run` steps only)
are reported after the step line, with keys sorted and values shown with their YAML type:

```text
  Step 1: uses actions/checkout@…
    With (not passed to any action):
      persist-credentials: false
      python-version: "3.13"
  Step 3: run (…)
    Working directory: examples/python-basic (not resolved)
    Env (not bound):
      WSR_EXAMPLE_MODE: "${{ inputs.mode }}" (unevaluated)
```

Following the [pinned workflow schema](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json),
values may be strings, numbers or booleans; strings are quoted and escaped, and text with a `${{`
fence is marked unevaluated. An empty mapping renders as `{}`. No variable is bound, no input is
passed to an action, no directory is resolved and no expression is evaluated.

Non-mapping values, empty keys, nested values, keys duplicated ignoring case (as the pinned
parser rejects), `with` on a `run` step, `working-directory` on a `uses` step and blank or
non-string directories fail before stdout (`jobs.b.steps[1].env.A: expected a string, number or
boolean`). A whole-mapping expression, an expression in a key, a number/boolean key or a null value
(the pinned reader converts it to an empty string) is valid GHA that is reported as unsupported. Placement is checked before contents.

### Workflow and job environment and run defaults

Workflow-level and job-level `env` follow the step `env` rules above and render, unindented or
under the job, as `Env (not bound):`. `defaults.run` at either level accepts only `shell` and
`working-directory` strings and renders as source data; a job's values are never merged with the
workflow's and nothing is applied to any step:

```text
Env (not bound):
  MODE: "${{ inputs.mode }}" (unevaluated)
Run defaults (not applied):
  Shell: bash
  Working directory: src
Job: b
  Runner: linux
  Run defaults (not applied):
    Shell: ${{ matrix.shell }} (unevaluated; expression syntax not validated)
```

`defaults: {}` and `defaults: {run: {}}` both render `Run defaults (not applied): {}`. Following the
pinned schema, workflow and job `env` and job `defaults.run` have expression contexts: an expression
replacing the whole mapping is unsupported, and job `shell`/`working-directory` text with a `${{`
fence (or an expression key) is reported as unevaluated or unsupported. Workflow `defaults.run` and
`defaults` itself have none, so an expression there is invalid (`defaults.run.shell: expression is unevaluated and not allowed in
workflow defaults`). Unknown keys (``defaults.run: unknown field `bogus`, expected `shell` or
`working-directory` ``, routed to the key), non-mappings, null or nested values and empty strings
are invalid. A number or boolean `working-directory`, which the pinned reader converts to a string, is
unsupported; a number or boolean `shell` is invalid because the runner rejects a bare shell that is not
a built-in shell and has no `{0}`. A blank shell is invalid and a blank directory unsupported, as for steps.

### Workflow and job concurrency

Workflow and job `concurrency` print as a block after the level's run defaults:

```text
Concurrency (not applied):
  Group: ci
  Concurrency (not applied):
    Group: ${{ github.ref }} (unevaluated; expression syntax not validated)
    Cancel in progress: true
    Queue: max
```

A string is the group; a mapping takes a required `group`, an optional boolean `cancel-in-progress`
and an optional `queue` of `single` or `max`. Both pinned definitions (`workflow-concurrency`,
`job-concurrency`) have expression contexts, so `${{` text in any value is unevaluated; an
expression key is invalid because the pinned converter requires literal keys. The job string form and `group` must be non-empty: empty text or
null is invalid there, while an empty or null workflow string is unsupported. A number or boolean
group, and blank text, are unsupported. Unknown keys, a missing `group`, a non-boolean
`cancel-in-progress` (including the string `'true'`), another `queue` value and other shapes are
invalid. No group is formed and nothing is queued or cancelled.

### Job outputs

Job `outputs` print by name in sorted order after the job's concurrency and environment, as `Outputs (not resolved):`
lines in the step-input value format (`result: "${{ steps.s.outputs.r }}" (unevaluated)`), or
`Outputs (not resolved): {}`. Values are `string-runner-context` text with an expression context, so
`${{` text is unevaluated; the
[pinned](https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json)
`job-outputs` mapping itself has none, so an expression replacing it or used as a key is invalid
(`jobs.b.outputs: expression is unevaluated and not allowed in job outputs`). Empty, null, number and
boolean keys (the pinned reader rejects raw scalar keys), keys repeated ignoring case, non-mappings
and nested values are invalid; number, boolean and null values, which the pinned reader converts to
strings, are unsupported. No output is
resolved or exposed to `needs`.

### Job environments

A job `environment` prints after its concurrency as a block:

```text
  Environment (not selected; no protection rules evaluated):
    Name: ${{ inputs.target }} (unevaluated; expression syntax not validated)
    URL: https://example.test
    Deployment: false
```

A string is the name; a mapping takes a required `name`, an optional `url` and an optional boolean
`deployment`. The pinned `job-environment` has an expression context, so `${{` text in any value is
unevaluated; an expression key is invalid because the pinned converter requires literal keys.
Empty or blank names and URLs, and number, boolean or null ones (which the pinned reader converts
to strings), are unsupported. Unknown keys, a missing `name`, a non-boolean `deployment` (including
the string `'false'`), nested values and other shapes are invalid. No environment is selected or
created, no protection rule, secret or variable is evaluated, and no URL is opened.

### Reusable-workflow calls

A job with `uses` (the pinned `workflow-job`) prints its reference and declared values instead of a
runner and steps:

```text
Job: check-fmt (format)
  Calls workflow: ./.github/workflows/check-fmt.yml (not loaded; not resolved)
  With (not passed):
    mode: "fast"
  Secrets: inherit (not passed)
```

`uses` must be `./path` or `owner/repo[/path]@ref`, following the pinned `parseFileReference`
(`jobs.b.uses: invalid workflow reference 'octo/repo/w.yml'; …`); it has no expression context, so
`${{` text is invalid, and a `$/` self-repository reference is unsupported. `with` and a `secrets`
mapping render like step `with` (sorted, scalars, `${{` values unevaluated, keys unique ignoring
case); the mappings themselves have no expression context, so an expression in place of either
mapping or in a key is invalid. `secrets` is otherwise only `inherit`. `runs-on`, `steps` and other
keys the pinned `job-factory` has but `workflow-job` lacks are invalid with `uses`
(``jobs.b.runs-on: `runs-on` cannot be combined with `uses` at line 4 column 5``), as are `with` and
`secrets` without it, whatever the key order and before the value is read; `strategy` stays
unsupported. The referenced file is never opened, read or resolved, and no input
or secret is passed.

### Unevaluated job and step conditions

A job or step `if` string is reported as written, bare or `${{ }}`-fenced, after the job's runner
and needs or the step's header line, with `escape_debug` escaping:

```text
Job: b
  Runner: linux
  Condition: github.event_name == \'push\' (not evaluated; expression syntax not validated)
  Step 1: run
    Condition: ${{ failure() }} (not evaluated; expression syntax not validated)
```

The pinned schema's `job-if` and `step-if` are strings that GitHub evaluates as expressions with or
without the fence. Inspection does not parse, validate or evaluate the text (any text is accepted, so
the line says the syntax is not validated), and a condition does not
change which jobs or steps are reported. A number or boolean (which the pinned reader converts to a
string) and a null, empty or blank condition (which GitHub treats as `success()`) are unsupported
(`jobs.b.if: non-string condition is not inspected yet at line 3 column 5`); sequences and mappings
are invalid.

### Literal runner labels

`runs-on` accepts a nonempty scalar string or nonempty list of nonempty strings.
Scalars retain `Runner: ubuntu-latest`; lists render as
`Runner labels: ["self-hosted", "linux", "x64"]`, including quoted/escaped labels.
Source form, order, exact spelling and repetitions are preserved; a one-element list stays a list.
No runner is discovered, selected or provisioned, and labels do not establish platform support.

Null values, nested lists, empty lists and blank members fail before stdout; a number or boolean
(scalar or list entry, converted to a string by the pinned reader) is unsupported rather than invalid.
Runner-selector mappings, including one whose `group` or `labels` hold numbers or booleans, are
explicitly unsupported. `${{` text in a scalar or list label is kept unevaluated and never
selects a runner: `Runner: ${{ matrix.os }} (unevaluated; expression syntax not validated)`, or a
list followed by `(some unevaluated; expression syntax not validated)`. Semantic diagnostics identify `jobs.build.runs-on[2]` for a second label;
decoding errors retain available source context. No normalization or label resolution occurs.

### Explicit step identities

An explicit `steps[].id` is retained in the source summary and displayed as
`Step 1 [compile]: run (Compile sources)` or `Step 2 [upload]: uses owner/action@v1`.
Steps without IDs retain their previous output; inspection never generates an ID or changes step
order. IDs do not resolve outputs or expressions, select steps, or establish runtime identities.

An ID must be a nonempty string of fewer than 100 ASCII identifier characters: a letter or
underscore first, then letters, digits, underscores or hyphens. The `__` prefix is reserved.
Explicit null, number and other non-string values and expressions fail; a boolean `true` or `false`
converts to a valid identifier and is reported as unsupported. Duplicate IDs within one job fail with
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
Job identifiers are validated before `needs` is resolved, so an invalid identifier is reported
ahead of an unknown dependency.
A job identifier is also validated before its body is decoded, so a hostile identifier cannot forge
the route of a body diagnostic; when both are wrong, the identifier is the one reported. An expression in a field the pinned schema gives no expression
context (workflow `name`, `on`, job and step identifiers, `needs`, `uses`, manual-input
declarations) is invalid; where the schema allows expressions it is valid GHA that inspection does
not evaluate: job and step `name` and `runs-on` keep the text unevaluated
(`Job: b (${{ matrix.os }}; unevaluated; expression syntax not validated)`, likewise after a step
heading), while `run` stays unsupported. See the
[classification record](../implementation/inspection-classification.md).

### Diagnostic routes

Decoding and semantic diagnostics share one route grammar, `jobs.<id>.<field>[<n>]` with one-based
list indices: `jobs.test.needs[2]: expected a string`, `jobs.b.runs-on[2]: expected a string
label`, `jobs.b.steps[1].id: expected a string`. An unknown key is reported at its enclosing
mapping, whose detail names the key (``jobs.b.steps[1]: unknown field `bogus`, …``). Decoding
errors keep the parser's line/column. Identifiers in diagnostic text are quoted as `'…'` and
escaped once (`jobs.b\u{1}: invalid job identifier 'b\u{1}'`); a duplicate job names its
identifier, and one repeated ignoring case also names the first spelling (`jobs.Build: duplicate
job identifier 'Build'; already defined as 'build'`). Errors raised by the YAML parser or decoder itself (syntax, a scalar where a step
mapping belongs, duplicate keys inside a value, recursion/alias limits) keep the decoder's wording
and zero-based path. Successful static validation does not establish scheduling, conditions,
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
