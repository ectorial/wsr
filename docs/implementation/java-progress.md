# Java E2E worker progress

```mermaid
flowchart LR
    I[Recorded inputs] --> C[Checkpoint evidence]
    C --> L[Limits and pending work]
    L --> N[Next verified action]
```

## Current checkpoint — November 5, 2026

Design documentation is merged on main at `031b583e60ce93b6b0af3b41afdec02b393aedff`
([PR #17](https://github.com/ectorial/wsr/pull/17)). Refresh live state before work; historical branch
names and dirty-document lists below describe their original sessions.

The selected source remains junit-team/junit-examples at
`5bd6594a2336f1eff52527733ba28682a3215d83`, subproject junit-jupiter-starter-maven.
Last recorded prerequisite results are wrapper bootstrap and a partial 51-artifact dependency
repository. Native Maven success, the intended assertion failure, JAR checks and WSR execution
remain unverified. Liberica 25 / clean verify evidence below is an adapted reference; the Java
walkthrough requests Temurin 21 / verify and requires separately matching evidence.

The owner stopped smoke-test implementation during the subsequent design discussion. This document
does not resume that work. Before implementation resumes, reconcile the live owner instruction and
external worker prompt with the current architecture, verify the required orchestrator model,
and settle the next bounded behavior. Documentation-only approval does not resume the Java milestone.

After explicit resumption, verify existing fixture/artifact identities and qualify the chosen
reference path before running workloads. Do not select another project or repeat completed source
selection. Acquire live checkout ownership and preserve unrelated edits.

All target/wsr-tests paths below are ignored, machine-local evidence/preparation. They are not
included in a clean clone. The historical Python scripts and their exact resume commands apply
only if those files and their recorded hashes are present. A fresh checkout requires documented
input reconstruction and verified source/tool acquisition before any execution claim.

## Historical checkpoints

Dated entries below preserve observations and their original next-action suggestions. The current
checkpoint above is authoritative for handoff; later evidence must be added with its own date.

## Initial checkpoint — November 1, 2026

- Authorization: replace the existing test-preparation schedule with the [Java implementation worker](java-e2e.md).
- Checkout: `/Users/carlosferreyra/Development/ectorial/wsr`.
- Branch at handoff: `docs/gha-bun-execution-design`; verify live state before acting.
- Base revision at handoff: `a64136113d0a42f20c96a585ead5cc76dcbe0a94`.
- Existing dirty work: prior architecture/GHA/DX documentation and four package-description edits,
  plus this worker contract. These are intentional user-session changes; preserve them.
- Requested agent arrangement: GPT-6 Astra orchestrator; GPT-6.1 Sol/Terra subagents.
- Delegation is saved in the schedule prompt; heartbeat API has no model field. Verify the actual
  parent model before implementation. The authorized configuration-only turn
  `01a19a2e-5ae8-7b52-aaa3-59c0ae1c96a2` ran on `gpt-6-astra` (runtime metadata verified).
  Persistent model selection for future heartbeats is not exposed by the available API; each run
  must verify its actual parent model and stop implementation if it differs or cannot be verified.
- Product state: CLI/types scaffold. No Java workflow execution or isolation is proven.
- Selected Java repository/commit: not selected yet.
- Native Linux reference environment: not prepared or verified yet.
- Native reference results: not run.
- Active implementation test: none; no red/green result claimed.

## Historical initial next invocation

Inspect the live checkout and prerequisites, acquire exclusive ownership, then select and pin a
small Maven/GHA reference project. Record its license, workflow/actions, toolchain requirements,
and safe native reference environment. Prepare a bounded inspection/translation contract from the
actual inputs; if unresolved, record the smallest decision needed before implementation. Do not
start by installing/running arbitrary upstream build scripts on the host.

## Per-run checkpoint fields

Record run time/branch/revision and dirty-work ownership; selected source/action/tool identities;
current behavior and prerequisite decisions; exact commands; observed red/green/regression evidence;
raw-result locations; changes made; blockers; next action. Keep implementation evidence indexed in
`docs/testing/coverage.md`; this file tracks worker continuity, not duplicate coverage claims.

## November 2 prerequisite checkpoint

Runtime metadata confirms GPT-6 Astra orchestration. Read-only reference review assigned to
GPT-6.1 Sol agent `java_reference_review` with a self-contained handoff. No product implementation
or native workload was run. Branch and owner edits remain preserved.

Docker CLI is present and selects `desktop-linux`, but `docker info` fails because its daemon
socket is absent. No podman, colima, limactl or orb command was found on PATH. A usable isolated
Linux reference environment is not yet established. Options: (A) start and qualify the existing
Docker Desktop Linux environment with restricted mounts/credentials and resource limits
(recommended); (B) supply an independently managed disposable Linux environment. Neither has
been started or provisioned. Next action: finish the pinned Maven fixture review, then qualify
the chosen Linux environment before any upstream script execution. No red/green claim.

Sol read-only review selected `junit-team/junit-examples` at
`5bd6594a2336f1eff52527733ba28682a3215d83`, subproject `junit-jupiter-starter-maven`.
License EPL-2.0; wrapper Apache-2.0. Wrapper 3.3.4 downloads Maven 3.10.0 without
an upstream distribution checksum. POM declares compiler release 17, JUnit 6.1.3,
compiler plugin 3.16.0 and Surefire 3.6.1. Upstream `build-all-examples.yml` uses
Liberica JDK25, checkout `3d3c42e5aac5ba805825da76410c181273ba90b1` and setup-java
`b843ebf9b913146b38da2eaa7aeddf76074ae1ad` (both node24 declarations, not Bun proof).
Review source at https://github.com/junit-team/junit-examples/tree/5bd6594a2336f1eff52527733ba28682a3215d83/junit-jupiter-starter-maven .

Next preparation must preserve licenses, independently verify Maven/JDK/image digests,
and record an adapted manual-only workflow selecting only this Maven subproject,
with credentials disabled and explicit cache omission. Upstream runs multiple ecosystems
through Builder.java; this reduction is not unchanged upstream compatibility.
Inside the qualified disposable Linux environment, run `./mvnw --batch-mode
--no-transfer-progress -Dmaven.repo.local=/workspace/maven-repository clean verify`.
Expect five test invocations and the project JAR containing Calculator.class; all remain
unexecuted. In a second disposable copy change the addsTwoNumbers expected result from
2 to 3, rerun the same command, and require a real JUnit assertion failure plus Surefire XML.
No source has been downloaded or executed. Exact next action: qualify the reference-only
Linux environment and pin its image/JDK/Maven identities before these commands.

## November 2 manual preparation — downloaded inputs only

- Manual run `manual-preparation-20261102`, started `2026-11-03T05:14:27Z`
  (November 2, 22:14 Denver). Branch `docs/gha-bun-execution-design`, base
  `a64136113d0a42f20c96a585ead5cc76dcbe0a94`; pre-existing owner changes preserved.
- Parent runtime: `gpt-6-astra` (verified by parent metadata). Requested delegated fixture/docs
  assignment: `gpt-6.1-sol`, `fork_turns=none`, no reasoning override, successful spawn;
  delegated runtime metadata is not independently exposed. Sole writer for this slice.
- Atomic directory lock `target/wsr-tests/java-worker.lock` held by live PID 15454 with
  owner/run/start metadata throughout preparation and checks; released on completion.
- Materialized the complete selected Maven subtree (8 files), root EPL-2.0 `LICENSE.md`, and
  original `build-all-examples.yml` at `5bd6594a2336f1eff52527733ba28682a3215d83`.
  GitHub commit API verified commit and tree `5b7bf2f0aa1420ebd9575a96c2c4745fbb9dea2e`;
  bounded raw downloads verified each regular-file Git blob SHA-1 and byte length against
  that tree. No archive extraction, Git repository creation, or upstream script execution.
  Upstream executable mode is recorded in provenance; downloaded scripts remain review inputs.
- Evidence root: ignored `target/wsr-tests/java/manual-preparation-20261102/`.
  `provenance.json`, `commit.json`, `tree.json`, and `SHA256SUMS` record exact input identities;
  `source/` preserves original relative paths and license headers (wrapper Apache-2.0).
  `action-identities.json` and `action-metadata/` independently verify both action commit
  identities and capture actual metadata. Node declarations are not Bun qualification.
- Read actual POM, wrapper, tests and workflow. Confirmed compiler release 17, JUnit 6.1.3,
  compiler plugin 3.16.0, Surefire 3.6.1, wrapper script 3.3.4 and Maven 3.10.0 URL without
  upstream distribution checksum. Five expected invocations are source-derived, not observed.
- `manual-maven-derived.yml` is a proposed manual-only Maven reduction with pinned checkout
  revision, `persist-credentials: false`, Liberica Java 25, explicit Maven repository path,
  no remote cache and no other ecosystems/secrets. This differs from the upstream multi-build
  workflow; it is neither a new accepted contract nor unchanged-upstream compatibility evidence.
  Exact resolved Maven/JDK/image artifacts remain unverified and must be pinned before execution.
- `docker info`: exit 1, context `desktop-linux`, daemon socket absent; see `docker-info.log`
  and `.result.json`. Daemon was not started. Native Linux pass/fail remains blocked.
- `cargo xtask ci`: exit 0; format, locked compilation/Clippy, workspace tests and strict docs
  passed; see `cargo-xtask-ci.log` and `.result.json`. `git diff --check`: passed after these
  documentation updates. This is repository regression evidence only. No Java native or WSR
  execution, behavioral red/green, isolation, caching, state or cleanup claim is made.
- Exact next read-only command on this same branch: `docker info`. Once a separately authorized
  and qualified disposable Linux environment exists, pin its image/JDK/Maven artifacts, preserve
  the wrapper executable mode in a disposable copy, then run
  `./mvnw --batch-mode --no-transfer-progress -Dmaven.repo.local=/workspace/maven-repository clean verify`
  from the selected Maven subproject. Retain Surefire XML and JAR contents, then apply only the
  previously described expected-result 2-to-3 patch in a second disposable copy and require a real
  assertion failure. Do not run either command on macOS or count prerequisite failure as red.

## November 3 Linux reference prerequisite qualification

- Run `linux-qualification-20261103`, November 3, 2026 UTC (November 2 Denver), on
  `docs/gha-bun-execution-design`, base `a64136113d0a42f20c96a585ead5cc76dcbe0a94`.
  Preserved all pre-existing owner documentation/package changes and prepared source inputs.
  Parent reports runtime-verified `gpt-6-astra`; requested delegate `gpt-6.1-sol`,
  `fork_turns=none`, no reasoning override. Delegated runtime metadata was not independently verified.
  Atomic `target/wsr-tests/java-worker.lock` owner PID 31342 remained live during mutations/checks;
  ownership is released at completion. This slice qualifies reference prerequisites only.
- `docker context inspect desktop-linux`, `docker image ls --digests`, and `docker info`
  established an initially empty image inventory and available Docker Desktop 29.8.2,
  Linux `aarch64`, kernel `7.0.14-linuxkit`, cgroups v2. No daemon/settings changes.
- Evidence root: ignored `target/wsr-tests/java/linux-qualification-20261103/`.
  Every preparation/probe command and exit is recorded in `manifest-command.json`,
  `pull-command.json`, and `commands.json`; raw manifests, image/container inspection,
  probe scripts/logs, cleanup and CI records are retained there.
- Public preparation used an empty task-local Docker client config (`config.json` = `{}`),
  with explicit local endpoint `-H unix:///Users/carlosferreyra/.docker/run/docker.sock`.
  `docker --config target/wsr-tests/java/linux-qualification-20261103/docker-config -H
  unix:///Users/carlosferreyra/.docker/run/docker.sock manifest inspect
  bellsoft/liberica-openjdk-debian:25` resolved the Linux ARM64/v8 manifest. Pulled with
  `pull --platform linux/arm64 bellsoft/liberica-openjdk-debian@sha256:c8c34e65bac71254ddbccdfe107a0a572a3660c569fbfb7fa9c55fe28a418312`.
  Pull and local image inspection agree on that content digest and Linux ARM64 identity;
  no registry credential helper or personal Docker client configuration was supplied.
  The tag is discovery input only; subsequent use must retain the digest.
- The image reports BellSoft JDK `25.0.5.0.3+5-LTS`, release date `2026-10-24`,
  `JAVA_HOME=/usr/lib/jvm/jdk-25.0.5.0.3-bellsoft-aarch64`. In-container SHA256:
  `bin/java` = `0de729fed9f5bdfe3d8a97df63f7c13f11ce7bd24c76fe5edeef49504453886c`;
  `lib/modules` = `db62dd0deb5f0a63047e8302778cadad3c95a40603f55e1c9b1858f3d2319a1a`.
  This pins the image-contained JDK, not an independently downloaded vendor JDK archive,
  publisher signature verification, or the hosted setup-java action's resolved artifact.
- Exact container creation argv is in `commands.json`. Both disposable containers used
  `--platform linux/arm64 --user 10001:10001 --network none --read-only --cap-drop ALL
  --security-opt no-new-privileges=true --cpus 2 --memory 1g --memory-swap 1g
  --pids-limit 128 --ulimit nofile=1024:1024`; `/tmp` was a 64MiB noexec/nosuid/nodev
  tmpfs and `/workspace` a 256MiB nosuid/nodev tmpfs, both UID/GID 10001 mode 0700.
  No bind/volume/socket mounts, devices, host-home access, secrets, or host environment
  forwarding were requested. Inspected `Mounts=[]`; image has no declared volumes.
  Environment contained only explicit HOME/LANG and image PATH/LANGUAGE/JAVA_HOME.
- Observed UID/GID 10001, read-only `/etc` write rejection, writable private tmpfs,
  absent host `/Users` and control sockets, capabilities zero, `NoNewPrivs: 1`,
  `Seccomp: 2`. Cgroup reads matched memory 1073741824, PIDs 128, CPU quota
  `200000 100000`; no default route. Explicit TCP attempt to `1.1.1.1:443` failed
  with `Network is unreachable` (`NETWORK_DENIED`). A second fresh container lacked
  first-container tmpfs sentinels (`FRESH_TMPFS_OK`). Both exited 0 and were removed;
  final filtered `docker ps --all` output is empty. The pinned image remains reusable.
  These bounded probes do not establish a hostile-code security audit or WSR isolation.
- `cargo xtask ci`: exit 0 (format, locked compilation/Clippy, workspace tests, strict docs).
  `git diff --check`: passed after this checkpoint/index update. This is regression evidence,
  not behavioral red/green. No Maven, upstream script, native Java tests, hosted workflows,
  WSR translation/execution, Bun qualification, cache or state behavior was run.
- Next bounded slice: independently fetch/pin Maven 3.10.0 and its checksum/signature inputs,
  decide how to use the verified image-contained JDK versus pinning a separate vendor archive,
  and prepare dependency inputs with an explicit restricted acquisition plan. Preserve the
  selected source at `target/wsr-tests/java/manual-preparation-20261102/`. Do not start
  wrapper/network execution merely because the no-network probe passed. Later run the
  recorded wrapper command in disposable Linux, retain Surefire XML/JAR contents, then
  execute the expected-result 2-to-3 controlled-failure copy. Native pass/fail remains pending.


## November 4 Maven input preparation — no workload execution

- Run `maven-inputs-20261104`, started November 4, 2026 UTC (November 3 Denver), on
  `docs/gha-bun-execution-design`, base `a64136113d0a42f20c96a585ead5cc76dcbe0a94`.
  Preserved all owner changes and stash `daee0885832df55a9a872037d3ce6741b9e6e72a`.
  Parent reports runtime-verified `gpt-6-astra`; requested delegated `gpt-6.1-sol`;
  delegate runtime metadata is not independently exposed. Sole writer; atomic lock keeper
  PID 51830 remained live throughout preparation/checks and is released at completion.
- Read the actual pinned wrapper properties: exact distribution URL is
  `https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.10.0/apache-maven-3.10.0-bin.zip`.
  Bounded credential-free HTTPS downloads captured ZIP, upstream `.sha512`, `.sha1`, `.asc`
  and official `https://downloads.apache.org/maven/KEYS`; `.sha256` returned 404 (retained).
  ZIP: 10,102,082 bytes; SHA256
  `1f6d9909266510f039f59aa0e13dcd2c66da85f043e56276e41f2918f8bddaff`.
  Computed SHA512 and SHA1 match their upstream files. These checks establish byte integrity,
  not publisher identity. No version substitution occurred.
- `gpgv` with a task-local dearmored official KEYS keyring returned exit 0 / VALIDSIG for
  fingerprint `88BE34F94BDB2B5357044E2E3A387D43964143E3` (Tamas Cservenak).
  This verifies the signature against the HTTPS-fetched Apache key set; no independent
  fingerprint trust anchor or ownertrust certification was established. Initial GPG import
  stalled and was terminated; its subsequent NO_PUBKEY result is retained, superseded by
  successful direct-keyring `gpgv`, without touching the personal GPG home.
- Inspected 138 ZIP inventory entries for absolute/traversal/backslash paths and symlinks;
  all passed, no extraction or downloaded code execution. Ignored evidence root:
  `target/wsr-tests/java/maven-inputs-20261104/` (`downloads.json`, `integrity.json`,
  `archive-inventory.json`, signature/KEYS provenance, exact GPG commands/raw logs).
- `dependency-acquisition-plan.md` records a reference-only plan: reuse the prior digest-pinned
  image/JDK identity, empty HOME/settings/repository, SHA256-pinned wrapper adaptation, restricted
  Central-only acquisition, full transitive/plugin inventory, then fresh offline pass/fail runs.
  Docker bridge networking alone does not enforce the required endpoint restriction. POM seeds
  are reviewed but no dependency closure or dependencies were acquired. No Maven/upstream script ran.
- `git fetch origin` succeeded after lock acquisition; read-only GitHub checks found no PR
  (all states) or workflow runs for this branch. Initial unsupported `gh pr status` JSON-field
  query failed, then corrected to `gh pr list --head ... --state all` (empty).
- Next exact action: qualify the restricted acquisition path and validated container input
  delivery in the evidence plan before any Maven invocation. Native Java pass/fail, Surefire/JAR,
  action/Bun, WSR translation/execution, cache/state/cleanup and product red/green remain pending.

Validation for this slice: `cargo xtask ci` exit 0 (format, locked compilation/Clippy, workspace
tests and strict docs); `git diff --check` passed. Logs/results remain in the ignored evidence root.
This is repository regression evidence only; no Java behavioral red/green claim.

## November 4 dependency preparation — bounded offline discovery

- Run `dependency-preparation-20261104`, started about `2026-11-04T15:04Z`, branch
  `docs/gha-bun-execution-design`, revision `a64136113d0a42f20c96a585ead5cc76dcbe0a94`.
  Preserved owner changes and stash `daee0885832df55a9a872037d3ce6741b9e6e72a`;
  fetched origin after atomic lock acquisition. Atomic directory owner metadata recorded keeper PID 68651. Final release found that PID
  absent: the subprocess did not survive tool-session lifecycle. The directory remained exclusive,
  but live-owner continuity was not established; this is an ownership-protocol defect. Next run
  must use a persistent PTY/exec session and verify its owner PID before each mutation.
  Parent reports independently verified `gpt-6-astra` at `2026-11-04T15:03:36.801Z`;
  requested delegate `gpt-6.1-sol`, whose actual runtime metadata was not independently exposed.
- Ignored evidence root: `target/wsr-tests/java/dependency-preparation-20261104/`.
  `prepare.py` SHA256 `2efffe50d804c519057bee8538bd614f8b428e9eb01eb29b0ed1dc6366cc29be`
  records exact acquisition/delivery/workload argv; `commands.json`, raw logs and `attempt-01/`
  through `attempt-27/` preserve earlier commands/scripts/failures. Final attempt is 28.
  Host Python only validates/copies bytes and performs explicit HTTPS downloads; no fetched
  wrapper, Maven, plugin or Java workload executed on macOS.
- Rechecked every source file against the prior provenance SHA256 and the Maven ZIP against
  `1f6d9909266510f039f59aa0e13dcd2c66da85f043e56276e41f2918f8bddaff` before extraction.
  Revalidated ZIP paths/symlink exclusion. Delivered regular files through bounded in-memory tar
  to nonroot Linux, with no bind mounts. Final `delivery-integrity.log` verifies SHA256 of every
  delivered file inside Linux; `delivery-files.json` inventories the inputs.
- Reused image digest `sha256:c8c34e65bac71254ddbccdfe107a0a572a3660c569fbfb7fa9c55fe28a418312`
  and prior credential-free Docker configuration/endpoint. Containers retained Linux ARM64,
  UID/GID 10001, network none, read-only root, dropped capabilities, no-new-privileges,
  CPU 2, memory/swap 1GiB, PIDs 128 and prior tmpfs/descriptor limits. Inspections show no
  mounts/credentials/host environment forwarding. Initial delivery failed because Docker
  created the nested workdir root-owned; corrected creation workdir to `/workspace` before
  nonroot transfer. A second attempt exposed Docker's default noexec workspace tmpfs;
  explicitly selected `rw,exec,nosuid,nodev` for the private 256MiB workspace, retaining
  `/tmp` noexec. Actual mount options and UID are retained in `delivery-probe.log`.
- Disposable wrapper properties add the verified distribution SHA256. Preseeded only its
  reviewed URL-hash cache `home/.m2/wrapper/dists/apache-maven-3.10.0/dae9a06d` from verified
  extracted Maven bytes, so the wrapper does not download/extract a distribution. No original
  source was modified. `./mvnw --version` exits 0: Maven 3.10.0, BellSoft 25.0.5.0.3,
  Linux aarch64. This qualifies local input delivery/bootstrap, not setup-java or hosted GHA.
- Acquisition used explicit fixed public `repo.maven.apache.org/maven2/` HTTPS URLs with
  Python proxy handling disabled, redirects rejected, 20-second request timeouts and 8MiB
  per-artifact bounds. Each artifact matched its fetched SHA1; SHA256/size/URL are recorded
  in `artifacts.json`. These are same-origin integrity checks, not publisher-signature trust.
  No Maven/network-connected workload or proxy was used. Policy probes cover rejected example
  URLs; this qualifies the recorded fixed-URL batch, not arbitrary dynamic URL admission.
- Acquired **51 POM/JAR artifacts, 2,745,759 bytes**, including reviewed project dependencies,
  lifecycle plugin inputs, parents and imported BOMs. Unchanged artifacts are reused after
  hash checks. `delivery/repository/` is a resumable **partial** repository, not a dependency
  closure/lockfile. Successive fresh-container `clean verify` attempts used `--offline`, empty
  private HOME, explicit user/global settings, Central-only mirror and private repository.
  Final exact workload: `./mvnw --offline --errors --batch-mode --no-transfer-progress
  --settings /workspace/settings.xml --global-settings /workspace/global-settings.xml
  -Dmaven.repo.local=/workspace/repository clean verify`.
- Final workload exits 1 after entering clean/resources goals: unresolved descriptors for
  `org.apache.maven.shared:maven-filtering:jar:3.5.0` and
  `org.codehaus.plexus:plexus-utils:jar:3.6.0`. Their actual POMs name still-unacquired parents
  `org.apache.maven.shared:maven-shared-components:pom:47` and `org.codehaus.plexus:plexus:pom:19`.
  No compilation/tests/Surefire/JAR outcome or native baseline pass is established. These are
  setup failures, not Java assertion failures or product behavioral red evidence.
- Exact next preparation command, **after reacquiring the worker lock on this branch**:
  `python3 target/wsr-tests/java/dependency-preparation-20261104/add-reviewed.py 28
  org/apache/maven/shared/maven-shared-components/47/maven-shared-components-47.pom
  org/codehaus/plexus/plexus/19/plexus-19.pom` (one command line). This archives final attempt28,
  acquires the two reviewed fixed parent paths, then repeats fresh isolated offline discovery.
  No host Maven invocation is authorized. Native pass/fail remains a later lane after closure.
- Every created container was removed; final `cleanup-list.log` is empty. `CARGO_NET_OFFLINE=true
  cargo xtask ci` exits 0 (format, compilation/Clippy, workspace tests, strict docs).
  `git diff --check` passed after documentation updates. Validation is repository regression
  only; no WSR translation/execution, action/Bun, caching, state or product cleanup claim.
  Lock ownership is released after final checks; ignored `lock-release.json` records release.
