# GHA frontend modeling references

Status: **reference-guided design proposal**, not an implemented schema or a frozen Rust API.
The owner requested a scan for concrete implementation models that WSR agents can reuse as
engineering guidance. This document owns that catalog; the
[central provider guide](../../../docs/provider/gha/README.md),
[decisions](../../../docs/context/decisions.md) and
[validation gates](../../../docs/provider/gha/validation.md) retain architectural authority.

## Scope, sources and authority

Scanned zizmor's workflow/action models, common value/reference types, expression AST and YAML
location APIs at **`a80b19c9a2a8df847e80c9681a1e43a9dde38deb`**. Checked the official frontend's
schema, token model and conversion rules at **`4043eda158e16579cc5fb1b0b07a4bce2a76f0b5`**.
Links below pin those revisions. These are source observations, not assurances of current or
complete GHA compatibility.

Use the official schema and tested interpretation as the compatibility reference. Use zizmor
as an implementation-pattern reference. When they disagree, investigate a fixture rather than
copy a static-analysis shortcut into executable behavior. Neither frontend's convenience model
is automatically WSR's canonical execution model.

The shipped WSR frontend remains Rust. GitHub's TypeScript implementation can serve as a
pinned development/CI oracle; introducing that harness is future work, not completed by this doc.
D-031 currently permits the small built-in Rust inspection path. This document changes neither
the signed Component target boundary nor the paused Java worker. It does not add an upstream
crate, symlink, submodule, vendored source or runtime sidecar.

## Keep the representation stages separate

| Stage | Representation and responsibility |
| --- | --- |
| Source | Retain original bytes plus structural YAML nodes and locations; preserve unknown fields and source spelling. |
| GHA syntax model | GHA-specific enums/structs describe source forms and retain relevant distinctions. Parsing is not permission to execute. |
| Validated GHA model | Check field legality, required/exclusive fields, context rules, identifiers and relationships; record normalization and provenance. |
| WSR lowering | Translate the declared supported semantics into provider-neutral requirements and expressions; diagnose unrepresentable behavior. |
| Execution | Provision and enforce the validated plan under trusted policy. No source/frontend library types become implicit execution authority. |

These are conceptual stages, not a requirement to create five crates or duplicate every value.
The current `WorkflowSummary` is an inspection result; legacy `WorkflowIR` is not an accepted
complete GHA schema. Do not expand either indiscriminately to stand in for all stages.

## Candidate type shapes at a glance

These are proposed WSR names and payloads, not a transcription of upstream APIs. The numbered
catalog explains the evidence and validation obligations for each shape.

| Concept | Candidate shape | Payload distinction to preserve |
| --- | --- | --- |
| Job body | `Steps(StepsJob)` or `WorkflowCall(CallJob)` | Runner/steps versus called-workflow reference/inputs/secrets |
| Step body | `Command(CommandStep)` or `Action(ActionStep)` | Script/shell/directory versus action reference/inputs |
| Field value | `Literal(T)` or `Expression(ExpressionSource)` | A literal value versus a whole unevaluated expression |
| String template | `Vec<TextSegment>`; segment is text or expression | Ordered interleaving, raw spelling and spans |
| Optional event body | `Absent`, `DefaultBody`, `Configured(T)` | No event versus enabled event with defaults versus configured event |
| Action reference | `Local`, `Repository`, `Image` | Relative path; owner/repository/subpath/ref; image/tag/digest |
| Runner selector | `Labels` or `GroupAndLabels` | Label requirements versus group selection plus labels |
| Filter form | `Include(Vec<Pattern>)` or `Ignore(Vec<Pattern>)` | Exclusive source forms and ordered patterns |
| Permission declaration | `Inherited`, `AllRead`, `AllWrite`, `Scopes` | Omission versus blanket declaration versus explicit map, including empty map |
| Secret passing | `Absent`, `Inherit`, `Explicit` | Omitted secrets versus inheritance versus supplied mappings |
| Action metadata body | `JavaScript`, `Composite`, `Image` | Runtime/lifecycle; nested steps; container entrypoint/arguments |
| Expression node | literal, call, access/index, projection, unary, binary | Operator/operand structure, allowed function/context rules and source origin |

## Concrete modeling catalog

Each recommendation below is a proposed WSR adaptation. The linked symbol is the observed
upstream pattern. Names in the sketches are illustrative; existing accepted interfaces win.

### 1. Job kinds are a sum type

Observed: [Job][jobs] distinguishes a job with steps from a reusable-workflow invocation.

Propose `JobBody = StepsJob | WorkflowCall`, with variant-specific required fields. Keep runner,
steps and services on the steps-job variant; keep workflow reference, supplied inputs and secrets
on the call variant. Validate forbidden combinations before choosing a variant. A struct with
optional `runs_on`, `steps` and `uses` everywhere admits contradictory states.

Acceptance: both valid job kinds retain their data; a normal job mixed with job-level `uses`
fails explicitly; unknown call targets are never fetched merely to parse the caller.

### 2. Step variants share metadata, not incompatible bodies

Observed: [Step, SharedStepFields, RunStep and UsesStep][job-fields] separate kinds and common
metadata. The snapshot also models wait/cancel/parallel forms; their presence is not WSR support.

Propose a common step record containing id, name, condition and provenance, plus a body enum
for command or action invocation. Keep script/shell/working-directory together and action
reference/inputs together. Gate additional variants by explicit language-feature support.

Acceptance: exactly one body; preserve order; reject both `run` and `uses` rather than letting
an untagged deserializer choose whichever variant succeeds first.

### 3. Runtime-valued fields distinguish literals and expressions

Observed: [LoE and ExplicitExpr][literal-expression] encode a literal-or-expression choice.

Propose `ValueOrExpression<T>` where the schema permits a whole expression. Distinguish an
expression from a literal string before generic string conversion. Delimiter recognition alone
is not expression validation; parse the body and check its allowed contexts at the field site.

Acceptance: literal values, whole expressions, malformed delimiters and expression-looking
quoted strings follow explicit, fixture-backed rules.

### 4. Mixed strings need a sequence of segments

Observed: [the official expression tests][embedded] preserve embedded expressions while building
a normalized format expression; [the token type][expression-token] retains expression metadata.

Propose a source string template containing ordered literal and expression segments. Preserve
raw text and offsets; lowering may normalize later. This complements `ValueOrExpression<T>`:
`echo ${{ github.ref }}` is not one literal value or one whole expression.

Acceptance: multiple expressions, escaped braces, multiline YAML scalars and unterminated fences
retain correct boundaries. Do not inspect embedded shell syntax by running it.

### 5. Conditions are expressions with field-specific rules

Observed: [If][conditions] distinguishes scalar booleans and expression text; the
[official converter][if-normalization] applies implicit status-function semantics.

Propose a condition model that preserves original form and a parsed expression, with explicit
normalization. Bare and fenced conditions cannot be handled only by the whole-expression wrapper.
Do not turn a condition into a Rust boolean before its evaluation phase is known.

Acceptance: job versus step contexts, scalar coercions, fenced/bare forms and implicit `success()`
behavior agree with reference fixtures. A successful AST parse does not authorize every context.

### 6. References are parsed variants, not resolved identities

Observed: [Uses and its reference models][references] distinguish local, repository and image
references and retain the raw spelling.

Propose separate reference variants for those forms. Keep repository owner/name/subpath/ref,
image tag/digest and local path structured. Validate job-level workflow-call references separately
from step-level actions. Retain a reference as unresolved until verified acquisition selects an
identity; do not interpret every ref as a commit digest or grant filesystem access to a local path.

Acceptance: local paths, repository subpaths, tags/digests and forbidden placement are checked
without filesystem traversal, network requests or execution.

### 7. Runner syntax is independent of a selected backend

Observed: [RunsOn][runner] represents label targets and group/label objects, wrapped as a possible
expression by the job model.

Propose `RunnerSelector = Labels | GroupAndLabels`, optionally expression-valued where legal.
A scalar label can normalize to one label while source provenance retains the original form.
Avoid a closed enum of familiar hosted labels. Labels describe requirements, not a chosen VM,
container, CPU architecture or proof that enforcement is available.

Acceptance: string, list, group/labels, invalid empty selectors and dynamic selectors get precise
results; no successful parse qualifies an execution environment.

### 8. Trigger shape and trigger configuration are distinct

Observed: [Trigger and TriggerSyntax][triggers] retain scalar/list/mapping form while exposing
normalized event entries. [OptionalBody][event-presence] distinguishes an absent event, a present
event using defaults, and a configured body.

Propose normalized event entries plus source shape and three-state presence. An event-kind enum
can cover recognized kinds, with an explicit unknown-source case for diagnostics. Do not collapse
`no workflow_dispatch` and `workflow_dispatch:` into the same missing `Option`.

Acceptance: the three YAML shapes, present-empty configuration, repeated/unknown event names and
source routes are covered. Unknown events must not become silently executable default triggers.

### 9. Event filters encode exclusive forms

Observed: [BranchFilters, TagFilters and PathFilters][filters] use separate include/exclude variants.

Propose a filter choice per supported domain, with original pattern order preserved. Validate
mutually exclusive source keys before normalization. Do not silently sort patterns or assume
matching is a generic string-prefix operation.

Acceptance: conflicting include/ignore keys, ordered negative patterns and the interaction between
branch and path requirements match the official reference; parsing and event matching are separate.

### 10. Source scalar coercion must be field-specific

Observed: [EnvValue and conversion helpers][scalars] retain scalar kinds before conversion.

Propose a source scalar value representation and explicit coercion at schema-defined locations.
Missing, null, false, numeric zero and empty text are distinguishable until the applicable rule
is applied. Do not globally accept every YAML scalar as a string, or globally forbid coercion
merely because an initial strict inspection slice does so.

Acceptance: the same scalar in an environment value, identifier, condition and required script
is tested independently against the reference.

### 11. Permission requests and inherited defaults differ

Observed: [Permissions, BasePermission and Permission][permissions] separate blanket settings,
explicit scope maps and access levels.

Propose source permission requests with inherited/default, broad-read, broad-write and explicit
scope-map forms. Preserve an explicit empty map separately from omission. GitHub token scopes
are GHA semantics; they are not WSR filesystem/network capability grants.

Acceptance: omission, `{}`, scope-level denial and overrides survive lowering without increasing
trusted permissions. A workflow cannot select its effective WSR grants by declaring a request.

### 12. Matrices retain value structure until expansion

Observed: [Strategy and Matrix][matrix] distinguish axes, inclusion/exclusion entries and expressions.

Propose axes plus include/exclude records using structured values rather than `Vec<String>`.
Keep expression-produced matrices unresolved until inputs are available; expansion is a later
validated operation with explicit limits, not a deserialization side effect.

Acceptance: scalar kinds, dynamic axes, include/exclude effects, malformed keys and expansion
bounds are verified. Unsupported dynamic matrices cannot become empty successful plans.

### 13. Action metadata uses an execution-kind enum

Observed: [Runs and action-specific structs][actions] separate JavaScript, composite and image actions.

Propose corresponding source variants with their own required metadata. Preserve declared runtime,
entrypoint, pre/post lifecycle and conditions for JavaScript; nested source steps for composites;
image/entrypoint/arguments for image actions. Validate the discriminator instead of accepting any
string in a struct selected by field shape.

Acceptance: wrong discriminators, missing main/steps/image, lifecycle defaults and unknown runtimes
produce honest diagnostics. A Node runtime declaration does not prove Bun compatibility or permit
switching to another backend silently.

### 14. Workflow inputs and action inputs are different models

Observed: [workflow event input types][inputs] differ from [action Input and Output][actions].

Propose distinct workflow-dispatch, workflow-call and action-metadata input records. Share only
fields with equivalent semantics. Preserve default values and their scalar types, requiredness,
choice options, output expressions and source locations. The audited workflow-input models omit
`default`; do not reproduce that omission. A supplied input value is distinct from its declaration.

Acceptance: absent versus explicit default, required inputs, typed defaults, call outputs and
metadata inputs keep all relevant information through parsing and lowering.

### 15. Secret passing is a source policy choice

Observed: [Secrets][secrets] distinguishes inheritance from an explicit name/value mapping.

Propose an explicit choice for workflow calls, preserving omission separately. Expression text
is a reference to secret input, not the secret value. No secret resolution occurs during parsing;
source/debug rendering must not become a channel for resolved credentials.

Acceptance: inherited, explicit and absent cases stay distinct; invalid forms fail; inspection
never needs actual credentials.

### 16. Expression structure is a recursive enum with provenance

Observed: [Expr and SpannedExpr][expression-ast] represent literals, calls, context/index access,
operators and wildcards. [Context][context-access] represents structured access paths;
[BinOp/UnOp][operators] and [Function][functions] encode recognized operations.

Propose an AST with explicit operator/call/access nodes and a source origin for each node. Preserve
dot access, computed indexing and projections structurally; do not split context strings on dots.
Choose owned source plus spans or another lifetime strategy when implementation proves the need;
borrowing upstream's lifetime/self-referential machinery is not an architectural requirement.

Acceptance: precedence, grouping, escaped strings, computed indexing, wildcards, function arity,
unknown contexts and resource-depth limits have focused fixtures. Do not infer supported operators
from a broad comment when the actual enum/parser recognizes a narrower set.

### 17. Diagnostics need byte spans and source routes

Observed: [yamlpath Location and Feature][locations] connect structural queries to concrete source;
[expression Origin][expression-ast] records source spans and raw expression text.

Propose diagnostics with a code, category, structural route, primary span and optional related
spans. Keep file identity and original bytes available. Decoded multiline scalar offsets need an
explicit mapping back to YAML source; expression offsets are not automatically file offsets.

Acceptance: block/flow YAML, multiline values, duplicate keys and multibyte text locate the right
source. Define byte/character units and zero/one-based coordinates at interfaces rather than
mixing the upstream libraries' conventions.

### 18. Syntax variants do not replace validation state

Observed: [GitHub's schema][schema] defines field contexts and combinations;
[job conversion][job-converter] and [single-job conversion][single-job] add relationship and identifier
checks beyond token decoding.

Propose an explicit validation boundary before lowering. Check unknown/exclusive fields while raw
structure still exists, validate IDs before inserting into normalized maps, then validate references
and graph cycles. An expression AST and a typed source struct can both exist for an invalid workflow.

Acceptance: all supported constructs retain their values; invalid combinations are rejected; valid
but unimplemented behavior is categorized as unsupported. Never let successful deserialization alone
mark a model executable.

## Illustrative implementation shape

The following is a WSR design sketch, not copied upstream code or an approved API. Types omitted
from the sketch must be justified by actual slices rather than scaffolded preemptively.

```text
ParsedWorkflow
  source + provenance
  triggers: entries with source shape and presence
  jobs: ordered entries, retaining duplicate-key evidence until validated

ParsedJob
  metadata + source
  body: StepsJob | WorkflowCall

ParsedStep
  shared metadata + source
  body: CommandBody | ActionBody | ExplicitUnsupportedConstruct

FieldValue<T>
  literal T | whole expression
StringTemplate
  ordered text/expression segments
Condition
  original form + AST + normalization record
```

Use enums when alternatives have different required data or meaning; use structs for data that
coexists; use optional values only for genuine absence. Do not force every YAML union into the
same generic wrapper. Known unsupported constructs retain enough source to report a diagnostic,
but cannot pass the executable-validation boundary.

## How agents should use this catalog

For each authorized feature:

1. Locate the relevant pattern and inspect the pinned source and official schema/converter.
2. State the source forms, preserved distinctions, invalid combinations and unsupported behavior.
3. Add a real CLI/compiler acceptance test; observe red; implement the smallest complete behavior.
4. Check both acceptance decisions and retained/normalized values against pinned official fixtures.
   Acceptance agreement alone is not semantic equivalence or a reusable-code percentage.
5. Keep GHA models private to the frontend and lower through the owning WSR boundary. Update the
   relevant checkpoint and supported-subset documentation with observed evidence.

This is guidance for a WSR-owned implementation, not a requirement to depend on or clone zizmor
inside WSR. If code is later copied/adapted, retain applicable MIT notices and source provenance;
third-party workflow fixtures may carry their original repositories' licenses. Do not import a
new MSRV, experimental language feature or upstream dependency update without verification.

## Deliberate adaptations and remaining decisions

Do not adopt permissive unknown-field dropping, missing input defaults/container ports, or untagged
variant fallback that erases conflicting fields. Preserve those fields or reject them explicitly.
Do not import static-analysis constant evaluation as the complete runtime evaluator. Trusted policy,
provisioning, scheduling and backend enforcement remain outside the GHA source model.

Unsettled choices include the public/internal frontend API, source-tree implementation, duplicate-key
representation, diagnostic schema, evaluation-phase representation, supported GHA release/features,
and eventual Component/WIT serialization. These require bounded implementation decisions and tests;
this reference catalog does not settle them or authorize a broader parser rewrite.

## Source index

All source links below use immutable revisions; refresh deliberately in a reviewed change. Symbol
names identify observations even if a future upstream layout changes.

[jobs]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/mod.rs
[job-fields]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs
[literal-expression]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/common/expr.rs
[embedded]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/expressions.test.ts
[expression-token]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/templates/tokens/basic-expression-token.ts
[conditions]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/common.rs
[if-normalization]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/if-condition.ts
[references]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/common.rs
[runner]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs
[event-presence]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/event.rs
[filters]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/event.rs
[scalars]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/common.rs
[permissions]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/common.rs
[matrix]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs
[actions]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/action.rs
[inputs]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/event.rs
[secrets]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/job.rs
[expression-ast]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-expressions/src/lib.rs
[context-access]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-expressions/src/context.rs
[operators]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-expressions/src/op.rs
[functions]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-expressions/src/call.rs
[locations]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/yamlpath/src/lib.rs
[schema]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/workflow-v1.0.json
[job-converter]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/jobs.ts
[triggers]: https://github.com/zizmorcore/zizmor/blob/a80b19c9a2a8df847e80c9681a1e43a9dde38deb/crates/github-actions-models/src/workflow/mod.rs
[single-job]: https://github.com/actions/languageservices/blob/4043eda158e16579cc5fb1b0b07a4bce2a76f0b5/workflow-parser/src/model/converter/job.ts
