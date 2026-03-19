# Malinka Autonomous Development Platform: Complete Capabilities Review

## Executive Summary

Malinka is a **task-first autonomous development kernel** designed to replace the existing Autonomy framework for unattended repository execution. It is a production-grade platform featuring:

- **Deterministic task dispatch** with dependency resolution and conflict detection
- **Proof-gated delivery** with fail-closed semantics and honest verification
- **Recurring autonomous brains** (strategy, security, operations, learning) that steer work through validated source artifacts
- **Company OS integration** for doctrine-driven execution with explicit budgets and circuit breakers
- **Zero-human operational capability** with pre-authorized deploy/rollback actions and self-healing
- **Truthful runtime surfaces** (queue, session, health, execution audit) for operator visibility

Current stage: **stage_0_bootstrap** — replacement-critical platform work takes priority over feature expansion.

---

## 1. TASK DISPATCH MODEL

### 1.1 Queue Compilation

**Files:** `src/queue.rs`, `src/plan.rs`, `src/task_graph.rs`

Malinka compiles delivery and recurring work into **task packets** that feed the scheduler.

#### Delivery Task Packets
- **Source:** `ralph/IMPLEMENT.md` (parsed into Plan structure)
- **Fields parsed:**
  - Task ID, title, status (Pending/Complete/Blocked)
  - `Where`: file/directory locations
  - `Depends on`: dependency task IDs (comma or backtick-delimited)
  - `Conflicts with`: explicit conflict task IDs
  - `Tags`: string tags for workflow routing
  - `Affinity`: grouping key for cache/workspace reuse
  - `Blocked by`: lineage references that prevent dispatch
  - `Quality gate`: approval requirement marker
  - `Proof commands`: optional task-level proof overrides
  - `Spec path`: spec reference
  - `Deployment target`, `Deploy commands`, `Rollback commands`: deployment metadata
  - `Monitoring checks`: post-deploy verification
  - `Requires services`: service names needed at proof time

#### Recurring Task Packets
- **Source:** `project.yaml` recurring task definitions + `state/tasks/*/source-output.json`
- **Fields:**
  - `id`: recurring task role (strategy, security, operations, learning)
  - `cadence`: interval (e.g., 24h, 15m)
  - `focus_source`: doctrine/specs/operations
  - `freshness`: max age before re-run
  - `no_change_yield_after`: skip threshold for unchanged outputs
  - `engine`: adapter and model config
  - `semantic_adapters`: normalization rules for recurring output
  - `proof_commands`: recurring-specific proofs
  - `pre_authorized_action_classes`: actions allowed without explicit approval

#### Task Packet Structure (QueueState/TaskPacket)
```
TaskPacket {
  task_id: String,
  title: String,
  plan_status: TaskStatus,
  recurring: bool,
  allowed_paths: Vec<String>,
  proof_commands: Vec<String>,
  task_tags: Vec<String>,
  affinity_key: Option<String>,
  spec_path: Option<String>,
  deployment_target: Option<String>,
  monitoring_checks: Vec<String>,
  deploy_commands: Vec<String>,
  rollback_commands: Vec<String>,
  ... (lineage, authority, quality gate fields)
}
```

### 1.2 Dependency Resolution

**File:** `src/task_graph.rs`

The task graph validates and selects ready work:

#### Validation
- Detects duplicate task IDs → error
- Validates all dependency references exist → error
- Validates all conflict references exist → error
- **Cycle detection** via DFS with visit-state tracking → error if cycle found

#### Ready Task Selection
Tasks are ready when:
1. Status is `Pending`
2. All dependencies are `Complete` in the plan
3. No conflicting tasks are running/selected
4. Compatible with running worker slots

#### Scoring & Ordering
Ready tasks are ranked by:
1. **Priority** (explicit `Priority` field or inferred)
2. **Dependency depth** (downstream dependents)
3. **Recent failure age** (backoff penalty for failed tasks, 900s window)
4. **Family pressure** (penalty for tasks in same affinity group)
5. **Potential conflicts** (fewer is better)
6. **Task ID** (deterministic tiebreaker)

**Backoff:** Failed tasks apply `FAILURE_BACKOFF_WINDOW_SECONDS` (900s) penalty. Retry count tracked in session.

**Affinity:** Tasks with same `Affinity` key share cache and workspace, reducing penalty when grouped.

### 1.3 Conflict Detection

**Explicit conflicts:** `Conflicts with` field lists task IDs that cannot run in parallel

**Implicit conflicts:** 
- Tasks with same affinity key that are already running
- Delivery tasks that write overlapping allowed paths

**Enforcement:** Task graph filters selected ready tasks against running task conflicts before dispatch.

---

## 2. PROOF SYSTEM

### 2.1 Proof Commands & Gates

**Files:** `src/adjudicator.rs`, `project.yaml`, `IMPLEMENT.md`

Malinka enforces fail-closed proof verification before completion.

#### Proof Command Hierarchy
1. **Repo-level proof** (default):
   ```yaml
   proof_commands:
     - cargo test
     - cargo test -p malinka execution_trust_plane::
     - cargo clippy -- -D warnings
   ```

2. **Task-level proof override** in `IMPLEMENT.md`:
   ```
   - [ ] **TASK-01** — Implement feature
     - Proof commands: `cargo test module::`, `cargo clippy`
   ```

3. **Per-command specs** (advanced):
   ```
   {
     command: "cargo test",
     timeout_ms: 60000,
     expect_output: "test result: ok",
     expected_exit_code: 0
   }
   ```

#### Proof Execution
- **Timeout:** Configurable per command or global engine timeout (600s default)
- **Service start:** Optional managed services (devnet, relay) started before proofs
- **Remote proof:** If task binds to host (e.g., systemd unit on ns-db-1), proof runs via SSH
- **Exit code validation:** Default 0; can override with `expected_exit_code`
- **Output validation:** Optional `expect_output` regex matching

#### Proof Results
- `Passed`: All commands succeeded
- `Failed`: Command failed, stderr captured
- `InvalidProofContract`: Proof command malformed or not executable
- `ZeroTestsMatched`: Named test framework output shows 0 tests matched

### 2.2 Adjudication

**File:** `src/adjudicator.rs`

After engine run, output is parsed and adjudicated:

#### Turn Outcome Parsing
Engine stdout must end with **one** of:

**Success:**
```
RESULT: <item_id> <proof_summary> <commit_or_none> <next_item>
RESULT: EA-01 passed none EA-02
```

**Blocked:**
```
BLOCKED: <item_id> <blocker_tag> <prerequisite> <next_item>
BLOCKED: EA-01 needs_binary codex_binary EA-02
```

**Refused:** (bounded refusal)
```
REFUSED: <item_id> <reason> <suggested_action> <invariants_violated>
REFUSED: EA-01 invariant_violated check_invariants INV-001,INV-002
```

**Incomplete:** No structured line found → adjudicates based on evidence (changed files, proof signal, etc.)

#### Adjudication States
- `Verified`: Structured outcome + proof passed
- `Rejected`: Structured outcome + proof failed
- `Blocked`: Structured blocked outcome (no proof run)
- `BoundedRefusal`: Refused outcome + evidence of partial work
- `ProductiveIncomplete`: No structured outcome + evidence of useful work (changed files + proof signals)
- `Incomplete`: No outcome + no evidence

### 2.3 Proof Failure Handling

**Failures trigger:**
- Plan status reverted to `Pending`
- Task marked as failed in session with timestamp
- Retry scheduled with exponential backoff (up to `max_retry_backoff_ms`)
- Workspace preserved for repair retry
- Adjudication recorded in execution audit

**Honesty:** INV-002 mandates that proof commands must actually execute. False-green placeholders (e.g., `true` or `echo pass`) block readiness.

---

## 3. TRUNK INTEGRATOR (LANDING)

### 3.1 Land Model

**File:** `src/trunk_integrator.rs`

Verified delivery work lands to git trunk atomically:

#### Prerequisites
1. Adjudication passes (Verified or BoundedRefusal that passes proof)
2. Land gates open (no `no_ship`, no error budget exhausted, etc.)
3. Execution policy allows (allowed paths, proof honesty, trust metadata)
4. Workspace changes staged cleanly

#### Landing Steps
1. **Stage workspace delta** to git index (only allowed paths)
2. **Stage plan file** with task status updates
3. **Validate staged changes** exist and are coherent
4. **Run semantic land assertions** (optional per task)
5. **Create commit** with message `<task_id> — <task_title>`
6. **Push to origin/<branch>** (typically `origin/trunk`)

#### Commit Message Format
```
<task_id> — <task_title>

Malinka verification: <proof_summary>
Lineage: <lineage_id>
Authority: <authority>
```

#### Land Failure Handling
- Commit succeeds but push fails → `CommittedButPushFailed` state (commit locally, push will retry)
- Staging fails → error, no commit attempted
- Plan status rolled back on failure

#### Gate Bypass
Land gates can be bypassed if:
- Task has bypass-exception tags (e.g., `critical`, `rollback-only`)
- Gate state lists allowed exception tags
- Bypass is recorded in execution audit

### 3.2 Allowed Paths & Validation

**Files:** `src/execution_policy.rs`, `IMPLEMENT.md`

Each delivery task declares allowed paths:

```
- [ ] **TASK-01** — Implement feature
  - Where: `src/feature.rs`, `src/feature/*.rs`
```

**Validation:**
- Git diff against workspace is filtered to allowed paths only
- Escaping allowed-path boundary is an execution policy violation (INV-004)
- Plan file mutations always allowed (status updates)
- Spec and control-plane artifact writes require trust metadata (`Lineage`, `Authority`, `Quality gate`)

---

## 4. RECURRING LANES

### 4.1 Recurring Task Lifecycle

**Files:** `src/recurring/mod.rs`, `project.yaml`

Recurring tasks (strategy, security, operations, learning) run on cadence and mutate doctrine/work through **source artifacts**.

#### Task Execution
1. Load prior source artifact (if exists)
2. Compile prompt with doctrine context, function memos, board minutes
3. Run engine command in workspace
4. Parse output for `RESULT:` or `BLOCKED:`
5. Validate semantic adapters (normalization)
6. Apply source writes (to allowed targets in `WORKFLOW.md`)
7. Record reports and state
8. Schedule next run based on cadence

#### Source Artifacts

Each recurring task writes a JSON source artifact:

**Strategy:** `state/tasks/strategy/source-output.json`
```json
{
  "schema_version": 1,
  "change_state": "changed" | "no_change",
  "writes": [
    { "path": "ralph/IMPLEMENT.md", "mode": "replace", "content": "..." }
  ],
  "strategy_report": {
    "summary": "...",
    "stage": "stage_0_bootstrap",
    "recommended_focus_tracks": ["..."]
  }
}
```

**Security:** `state/tasks/security/source-output.json`
```json
{
  "writes": [...],
  "security_report": {
    "current_cycle": ["runtime", "build_os"],
    "recommended_next_scopes": ["deployment"],
    "scope_statuses": [...]
  }
}
```

**Operations:** `state/tasks/operations/source-output.json`
```json
{
  "writes": [...],
  "operations_report": {
    "current_health_status": "...",
    "active_issues": [...]
  }
}
```

**Learning:** `state/tasks/learning/source-output.json`
```json
{
  "writes": [...],
  "learning_report": {
    "failure_clusters": [
      { "outcome_class": "...", "failure_reason": "...", "task_ids": [...] }
    ]
  }
}
```

### 4.2 Lane Configuration

**File:** `project.yaml`

```yaml
recurring:
  tasks:
    - id: strategy
      role: strategy
      cadence: 24h
      focus_source: doctrine        # doctrine | specs | operations
      freshness: 36h                # max age before re-run
      semantic_adapters:
        - strategy_repo_native_aliases_v1
      proof_commands:
        - true                       # often just sanity check
      engine:
        adapter: codex
        model: gpt-5.4
        reasoning: high
        command: codex exec --yolo

    - id: security
      role: security
      cadence: 24h
      focus_source: specs
      freshness: 36h
      audit_scopes:
        - id: runtime
          roots: [src, tests]
          attestation_dir: state/tasks/security/attestations/runtime
      semantic_adapters:
        - security_legacy_remediation_packets_v1
        - security_attestation_path_rewrites_v1

    - id: operations
      role: operations
      cadence: 15m
      focus_source: doctrine
      freshness: 1h
      no_change_yield_after: 2     # skip if unchanged N times
      no_change_yield_multiplier: 2 # multiply interval after yield
      pre_authorized_action_classes:
        - inspect
        - status_check
        - log_read

    - id: learning
      role: learning
      cadence: 6h
      focus_source: doctrine
      freshness: 12h
      no_change_yield_after: 2
      pre_authorized_action_classes:
        - inspect
        - status_check
```

### 4.3 Writable Targets & Validation

**File:** `WORKFLOW.md`

Each recurring lane declares what it can write:

```yaml
recurring_sources:
  strategy:
    artifact_path: state/tasks/strategy/source-output.json
    writable_targets:
      - kind: file
        path: ralph/IMPLEMENT.md
      - kind: file
        path: OS.md
      - kind: file
        path: INVARIANTS.md
      - kind: file
        path: project.yaml
      - kind: file
        path: WORKFLOW.md
      - kind: directory
        path: ops
      - kind: directory
        path: specs

  security:
    artifact_path: state/tasks/security/source-output.json
    writable_targets:
      - kind: file
        path: ralph/IMPLEMENT.md
      - kind: directory
        path: state/tasks/security/attestations
      - kind: directory
        path: specs
```

**Validation:**
- All writes in source artifact must fall within writable targets
- Out-of-scope writes are rejected → INV-004 violation
- Plan mutations (`ralph/IMPLEMENT.md`) validated to remain parseable

### 4.4 Semantic Adapters

**Files:** `src/profile.rs`, `project.yaml`

Recurring source artifacts can be **normalized** by semantic adapters:

```yaml
semantic_adapters:
  - id: repo_strategy_lane_aliases
    kind: strategy_operating_lane_aliases
    aliases:
      endurance_gate: decentralized_casino

  - id: repo_security_status_aliases
    kind: security_scope_status_aliases
    aliases:
      finding_reported: finding
```

Adapters:
- Map vocabulary from recurring reports to canonical terms
- Normalize mismatched field names across brain generations
- **Never** widen write scope, trust metadata, or proof expectations
- Record normalization in compiler metadata

### 4.5 Freshness & Scheduling

Recurring tasks are scheduled when:
- Cadence interval has passed, **OR**
- Freshness threshold exceeded (output is stale), **OR**
- Operator explicitly requests run

Stale-detection skips runs when:
- Output unchanged for `no_change_yield_after` consecutive runs
- Interval multiplied by `no_change_yield_multiplier` on each yield
- E.g., operations with defaults: skip 2 unchanged runs, double 15m → 30m → 60m

---

## 5. ENGINE ADAPTERS

### 5.1 Supported Engines

**File:** `src/engines/mod.rs`, `project.yaml`

Malinka supports five engine adapters. In this repo, delivery work defaults to
OpenCode on MiniMax, while standing lanes default to Codex on GPT-5.4.

#### OpenCode (provider-routed CLI; current delivery default)
```yaml
engine:
  adapter: opencode
  model: minimax-coding-plan/MiniMax-M2.7-highspeed
  reasoning: high
  command: opencode run
  timeout_secs: 3600
```

**Command line built:**
```
opencode run --model minimax-coding-plan/MiniMax-M2.7-highspeed --agent malinka-worker --variant high
```

#### Codex (OpenAI via CLI)
```yaml
engine:
  adapter: codex
  model: gpt-5.4
  reasoning: high
  command: codex exec --yolo
  timeout_secs: 3600
```

**Command line built:**
```
codex exec --yolo --model gpt-5.4 --config model_reasoning_effort="xhigh"
```

**Flags:**
- `reasoning` → `model_reasoning_effort` config (`low`, `medium`, `high`, `xhigh`)

#### Claude (Anthropic via CLI)
```yaml
engine:
  adapter: claude
  model: claude-opus-4-6
  reasoning: high
  command: claude
  timeout_secs: 3600
```

**Command line built:**
```
claude --model claude-opus-4-6 --thinking-budget 8000
```

#### Kimi (legacy direct CLI adapter; supported but not the repo default)
```yaml
engine:
  adapter: kimi
  model: moonshot-v1-8k
  reasoning: high
  command: kimi-cli --yolo
  timeout_secs: 3600
```

#### Gemini (Google via generic CLI)
```yaml
engine:
  adapter: gemini
  model: gemini-2.0-flash
  reasoning: medium
  command: gcloud ai predict
  timeout_secs: 3600
```

### 5.2 Engine Contract Resolution

**Files:** `src/engines/mod.rs`, `src/supervisor/mod.rs`

Before dispatch, engine config is validated:

```rust
pub struct ResolvedEngineContract {
    pub config: EngineConfig,
    pub turn_timeout: Duration,   // from WORKFLOW.md codex config
    pub stall_timeout: Duration,  // stall_timeout_ms
}
```

**Validation:**
- Command must be non-empty → error
- Profile timeout vs workflow turn timeout conflict → error
- Adapter is created: `create_adapter(AdapterKind::Codex)` → Box<dyn EngineAdapter>

### 5.3 Execution Context

**File:** `src/engines/mod.rs`

Workspace-aware environment applied uniformly:

```rust
pub fn apply_execution_context(command: &mut Command, context: &EngineExecutionContext) {
    command.current_dir(&context.workspace_path);
    command.env("MALINKA_WORKSPACE_PATH", &context.workspace_path);
    command.env("MALINKA_SHARED_CACHE_PATH", &context.shared_cache_path);
    command.env("MALINKA_SCRATCH_DIR", &context.scratch_dir);
    command.env("MALINKA_REPO_ROOT", &context.repo_root);
}
```

**Available in prompt context as:**
- `{{ workspace_path }}`
- `{{ shared_cache_path }}`
- `{{ scratch_dir }}`
- `{{ repo_root }}`

### 5.4 Provider Failover & Cooldowns

**File:** `src/supervisor/mod.rs`

Malinka treats provider failures distinctly from task failures:

**Provider Failures:**
- Auth/quota exhausted → `ProviderCooldownRecord` logged
- Provider outage detected → cooldown for `ENGINE_QUOTA_COOLDOWN_SECS` (1800s)
- Fallback provider tried if configured

**Cooldown Tracking (SessionState):**
```rust
pub struct ProviderCooldownRecord {
    pub adapter: AdapterKind,
    pub reason: String,
    pub until_unix_seconds: i64,
}
```

**Fallback Config:** `project.yaml`
```yaml
engine:
  adapter: codex
  ...

fallback_engine:
  adapter: claude
  model: claude-opus-4-6
  ...
```

---

## 6. WORKSPACE MODEL

### 6.1 Workspace Preparation & Materialization

**File:** `src/workspaces/mod.rs`

Each task gets an isolated workspace:

**Structure:**
```
workspace.root/
  <task_id>.json            # workspace record
  <task_id>.patch           # workspace delta patch
  <cache_key>/              # shared cache (may be repo-wide or per-affinity)
    .malinka-derived-cache.json
    <cached files>
  <task_id>/
    .                       # cloned/materialized repo
    .malinka-canonical-root # marker file
    .malinka-scratch/       # scratch directory
```

**Config (project.yaml):**
```yaml
workspace:
  root: ~/coding/malinka-workspaces
  max_drift_commits: 20
```

**Config (WORKFLOW.md):**
```yaml
workspace:
  root: ~/coding/malinka-workspaces
  cache_root: .malinka-cache     # relative path in repo
  cache_scope: repo              # repo | affinity
```

### 6.2 Workspace Lifecycle

#### Ready → Materialized
1. **Clone/reset repo** from origin (or reuse existing if clean)
2. **Run `after_create` hook** (e.g., install dependencies)
3. **Record start time** and status → Running

#### Running → Dispatch
1. **Run `before_run` hook** (e.g., setup test fixtures)
2. **Apply execution context env vars**
3. **Run engine command** with prompt on stdin
4. **Capture stdout/stderr**

#### Dispatch → Adjudication
1. **Run `after_run` hook** (e.g., cleanup)
2. **Run proof commands** in workspace
3. **Adjudicate outcome** (parse, proof, evidence)

#### Adjudication → Land/Preserve
1. **If verified:** stage delta, land to trunk
2. **If productive-incomplete:** preserve workspace for repair
3. **If failed:** preserve evidence, clean workspace (after export)
4. **If complete:** snapshot workspace, clean after grace period

### 6.3 Drift Detection & Limits

**Drift:** Workspace commits behind `origin/trunk`

**Policy:**
- Fetch latest origin/trunk
- Count commits behind: `origin/trunk..HEAD`
- If `drift_commits_behind > max_drift_commits` (20 default) → **DriftBlocked**
- Prevents dispatch of stale workspace

**Reconciliation on startup:**
- Scan `state/workspaces/`
- Reconcile running/stale workspaces with actual filesystem
- Clean stale workspaces unless productive evidence exists

### 6.4 Cache Scope

**repo:** Shared cache across all tasks (default)
```
.malinka-cache/
  <shared files>
```

**affinity:** Per-affinity-group cache
```
.malinka-cache/
  affinity-<key>/
    <cached files>
```

**Benefits:**
- Reuse build artifacts, dependencies, tool caches
- Reduce cold-start time
- Reduce disk usage

### 6.5 Hooks

**Configuration (WORKFLOW.md):**
```yaml
hooks:
  after_create: ""              # e.g., npm install
  before_run: ""                # e.g., setup test DB
  after_run: ""                 # e.g., cleanup
  timeout_ms: 60000
```

**Behavior:**
- Shell commands (bash, sh)
- Run in workspace root
- Timeout enforced per hook
- Failure stops dispatch
- Success recorded in workspace record

---

## 7. IMPLEMENT.MD FORMAT

### 7.1 Syntax

**File:** `src/plan.rs`

IMPLEMENT.md is markdown with structured sections and tasks:

```markdown
# Implementation Plan

## Section Title
Source spec: specs/031226-malinka-platform.md

- [ ] **TASK-01** — Task description here
  - Where: `src/file.rs`, `src/module/*.rs`
  - Tests: `cargo test module::`
  - Verify: Parses correctly
  - Depends on: `TASK-00`, `FOUNDATION-02`
  - Conflicts with: `TASK-02`
  - Tags: backend, performance
  - Affinity: group-a
  - Priority: 1
  - Origin: strategy:strategy:repo
  - Lineage: lineage-st-03
  - Authority: ceo_strategy
  - Quality gate: approved
  - Blocked by: delivery-slot
  - Executive review: exec-rec-1
  - Engineering review: approved
  - Arbitration: approved
  - Spec path: specs/feature.md
  - Deployment target: staging
  - Deploy command: ./ops/deploy_staging.sh
  - Rollback command: ./ops/rollback_staging.sh
  - Monitoring checks: `curl -sf http://localhost:8080/health`
  - Requires services: devnet, local-relay
  - Semantic checks: `/health returns 200`, `/metrics exposed`

- [x] **TASK-02** — Completed task
  - Where: `src/feature.rs`

- [!] **TASK-03** — Blocked task
  - Blocking reason goes here
```

### 7.2 Status Markers

- `[ ]` = Pending (not started)
- `[x]` = Complete (done)
- `[!]` = Blocked (cannot proceed)

### 7.3 Supported Fields

All fields optional except task ID and title.

**Path & Location:**
- `Where`: file/directory paths (backtick-delimited list or comma-separated)

**Proof & Testing:**
- `Tests` / `Verify`: inline test commands
- `Proof commands`: override repo-level proofs
- `Requires services`: service names from config
- `Monitoring checks`: post-deploy checks (comma-delimited)
- `Semantic checks`: semantic postcondition assertions

**Dependency & Gating:**
- `Depends on`: task ID list (delivery dependencies)
- `Conflicts with`: task ID list (parallel incompatibilities)
- `Priority`: numeric priority (lower = higher)
- `Affinity`: grouping key for workspace reuse
- `Blocked by`: lineage references (external blockers)

**Lineage & Authority (for generated work):**
- `Origin`: origin marker (`<brain>:<role>:<scope>`)
- `Lineage`: lineage ID for tracking
- `Authority`: approval source (e.g., ceo_strategy)
- `Quality gate`: gate status (approved, pending, etc.)
- `Executive review`: executive approval lineage
- `Engineering review`: engineering validation lineage
- `Arbitration`: arbitration verdict lineage

**Deployment (for delivery tasks):**
- `Spec path`: linked spec document
- `Deployment target`: target stage/environment
- `Deploy commands`: command list for deployment
- `Rollback commands`: command list for rollback
- `Deploy command` / `Rollback command`: singular forms

**Metadata:**
- `Tags`: tags for routing/filtering (comma-delimited)
- `State effect`: state mutation summary
- `Persistence effect`: storage/plan side effects
- `Observable signal`: operator-visible signal

### 7.4 Parsing Rules

- Section headers: `## Title`
- Source spec: `Source spec: path/to/spec.md`
- Task line: `- [STATUS] **TASK-ID** — Task title`
- Field line: `  - Field name: field value`
- Field values can use backticks for lists: `` `item1`, `item2` `` or comma-separated: `item1, item2`
- Parsing is **fail-closed**: malformed syntax returns error, not silent skip

---

## 8. WORKFLOW.MD FORMAT

### 8.1 Structure

**File:** `src/workflow.rs`

WORKFLOW.md is YAML front matter + prompt template:

```markdown
---
tracker:
  kind: file
  path: ralph/IMPLEMENT.md

polling:
  interval_ms: 30000

workspace:
  root: ~/coding/malinka-workspaces
  cache_root: .malinka-cache
  cache_scope: repo

hooks:
  after_create: ""
  before_run: ""
  after_run: ""
  timeout_ms: 60000

agent:
  max_concurrent_agents: 3
  max_retry_backoff_ms: 300000

codex:
  command: codex exec --yolo
  turn_timeout_ms: 3600000
  stall_timeout_ms: 300000

recurring_sources:
  strategy:
    artifact_path: state/tasks/strategy/source-output.json
    writable_targets:
      - kind: file
        path: ralph/IMPLEMENT.md
      - kind: directory
        path: specs
---

# Prompt Template

You are a coding agent working on {{ project_name }}.

{{ prompt_body }}

## Task

{{ issue.title }}

{{ issue.description }}

## Rules

1. Run proof commands before claiming done.
2. Update {{ plan_file }} with task status and evidence.
3. End with: `RESULT: <id> <proof> <commit> <next>`
```

### 8.2 Front Matter Fields

**Tracker:**
- `kind`: `file` (only supported kind)
- `path`: path to IMPLEMENT.md

**Polling:**
- `interval_ms`: supervisor poll interval for new work

**Workspace:**
- `root`: workspace directory
- `cache_root`: cache directory path (relative or absolute)
- `cache_scope`: `repo` or `affinity`

**Hooks:**
- `after_create`, `before_run`, `after_run`: shell commands
- `timeout_ms`: hook timeout in milliseconds

**Agent:**
- `max_concurrent_agents`: max parallel task workers
- `max_retry_backoff_ms`: max retry backoff duration

**Codex (engine-specific):**
- `command`: codex command string
- `turn_timeout_ms`: per-turn timeout (can override profile)
- `stall_timeout_ms`: stall detection timeout

**Recurring sources:**
- `<role>`:
  - `artifact_path`: path to source-output.json
  - `writable_targets`: list of allowed write locations
    - `kind`: `file` or `directory`
    - `path`: path relative to repo root

### 8.3 Prompt Template Variables

**Available variables:**
- `{{ project_name }}` → from project.yaml
- `{{ plan_file }}` → from project.yaml
- `{{ workspace_path }}` → from execution context
- `{{ repo_root }}` → from execution context
- `{{ issue.title }}` → task title (delivery tasks)
- `{{ issue.description }}` → task title + fields (delivery tasks)
- `{{ doc.<path> }}` → loaded document content

**Syntax:**
- Variable references: `{{ variable_name }}`
- Unclosed variables → error during compilation
- Unknown variables → error during compilation
- Non-scalar variables (objects) → error during compilation

---

## 9. PROJECT.YAML CONFIGURATION

### 9.1 Minimal Configuration

```yaml
name: demo
spec_dir: specs/
plan_file: ralph/IMPLEMENT.md

proof_commands:
  - cargo test
  - cargo clippy -- -D warnings

engine:
  adapter: codex
  model: gpt-5.4
  reasoning: high
  command: codex exec --yolo
  timeout_secs: 600

workspace:
  root: ~/coding/demo-workspaces
  max_drift_commits: 20
```

### 9.2 Full Configuration Reference

**Basic:**
```yaml
name: project-name
spec_dir: specs/                    # spec document directory
plan_file: ralph/IMPLEMENT.md       # canonical plan file
```

**Proof:**
```yaml
proof_commands:
  - cargo test
  - cargo test -p malinka execution_trust_plane::
  - cargo clippy -- -D warnings

services:
  devnet:
    command: npm run devnet
    ready_check: curl -sf http://localhost:8545
    ready_timeout_ms: 30000
    stop_signal: SIGTERM

references:
  design_doc:
    path: docs/DESIGN.md
    purpose: Architecture reference
```

**Engine:**
```yaml
engine:
  adapter: opencode           # codex | claude | kimi | gemini | opencode
  model: minimax-coding-plan/MiniMax-M2.7-highspeed
  reasoning: high             # low | medium | high (influences config)
  command: opencode run
  timeout_secs: 3600

fallback_engine:
  adapter: claude
  model: claude-opus-4-6
  reasoning: high
  command: claude
  timeout_secs: 3600
```

**Doctrine (company OS):**
```yaml
doctrine:
  mission_paths:
    - specs/031226-malinka-v2.md
    - ralph/SPEC.md
  invariant_paths:
    - INVARIANTS.md
  os_path: OS.md
  kpi_registry_paths:
    - ops/kpi_registry.yaml
  scorecard_paths:
    - ops/scorecard.md
  no_ship_paths:
    - ops/no-ship-ledger.md
  reference_paths:
    - ops/risk_register.md
    - ops/instrumentation_backlog.md
    - ops/decision_log.md
    - agents/generated/ceo-strategy-context.md
```

**Recurring Tasks:**
```yaml
recurring:
  tasks:
    - id: strategy
      role: strategy
      cadence: 24h
      focus_source: doctrine
      freshness: 36h
      semantic_adapters:
        - strategy_repo_native_aliases_v1
      proof_commands:
        - true
      engine:
        adapter: codex
        model: gpt-5.4
        reasoning: high
        command: codex exec --yolo

    - id: security
      role: security
      cadence: 24h
      focus_source: specs
      freshness: 36h
      audit_scopes:
        - id: runtime
          roots: [src, tests]
          attestation_dir: state/tasks/security/attestations/runtime
      semantic_adapters:
        - security_legacy_remediation_packets_v1
```

**Recurring Sources & Writable Targets:**
```yaml
recurring_sources:
  strategy:
    artifact_path: state/tasks/strategy/source-output.json
    writable_targets:
      - kind: file
        path: ralph/IMPLEMENT.md
      - kind: directory
        path: specs
    semantic_adapters:
      - id: repo_strategy_aliases
        kind: strategy_operating_lane_aliases
        aliases:
          endurance_gate: decentralized_casino
```

**Adaptive Ramp-Up:**
```yaml
adaptive_ramp_up:
  enabled: true
  broad_context: true
  permissive_semantic_compilation: true
  suppress_repeated_semantic_failures: true
  default_compile_generated_work: true
  research_watch_sources:
    - id: trendshift
      domains:
        - trendshift.io
      purpose: Monitor platform trends
```

**Autonomy (zero-human):**
```yaml
autonomy:
  enabled: true
  daemon:
    pid_file: state/daemon.pid
    health_bind: 127.0.0.1:7654
    watchdog_file: state/daemon.watchdog
    watchdog_interval_secs: 30
  action_policy:
    deploy_staging:
      require_green_proof: true
      max_per_hour: 5
      canary_percentages: [10, 25, 50]
  budgets:
    staging_deploys:
      limit: 10
      alert_at: 8
      scope: staging
  circuit_breakers:
    deployment_freeze_trigger_consecutive_failures: 3
    auto_clear_after_minutes: 60
  self_healing:
    check_interval_minutes: 15
    disk_space:
      threshold_gb: 10
      workspace_retention_days: 7
```

**Health:**
```yaml
health:
  heartbeat_stale_after: 90s
  result_stale_after: 30m
```

**Workspace:**
```yaml
workspace:
  root: ~/coding/malinka-workspaces
  max_drift_commits: 20
```

**Polling & Tracking:**
```yaml
polling:
  interval_ms: 30000

tracker:
  kind: file
  path: ralph/IMPLEMENT.md

runtime_workspace:
  cache_root: .malinka-cache
  cache_scope: repo
```

---

## 10. HEALTH & MONITORING

### 10.1 Health State Surface

**File:** `src/health/mod.rs`, `state/health.json`

Malinka maintains a comprehensive health snapshot:

```json
{
  "worker_target": 3,
  "live_worker_count": 2,
  "selected_task_ids": ["EA-01", "EA-02"],
  "running_task_ids": ["EA-01"],
  "live_task_ids": ["EA-01"],
  "active_config": {
    "generation": 45,
    "degraded": false,
    "reload_error": null
  },
  "tasks": [
    {
      "task_id": "EA-01",
      "title": "Implement engine adapter",
      "plan_status": "pending",
      "status": "running",
      "running": true,
      "live": true,
      "heartbeat_age_seconds": 2.5,
      "last_adjudication": "verified"
    }
  ],
  "meta_tasks": [
    {
      "task_id": "strategy",
      "role": "strategy",
      "status": "running",
      "fresh": true,
      "useful": true,
      "degraded": false,
      "age_seconds": 3600,
      "last_outcome_class": "changed",
      "summary": "Prioritized feature X",
      "generated_work_count": 3,
      "brain_state_version": 15,
      "self_authored_work_pending": 0,
      "arbitration_debt": 0,
      "provider_continuity_status": "active"
    }
  ],
  "company_os": {
    "no_ship": false,
    "freeze_non_urgent_changes": false,
    "error_budget_status": "healthy",
    "allowed_exception_tags": ["critical", "rollback-only"],
    "reasons": []
  },
  "control_evidence": {
    "lane": "platform",
    "status": "green",
    "no_ship": false
  },
  "execution_metrics": {
    "window_days": 7,
    "metrics": [
      {
        "id": "truthful_unattended_turn_rate",
        "status": "green",
        "value": 0.97
      }
    ]
  },
  "autonomy": {
    "enabled": true,
    "daemon_status": "running",
    "provider_continuity": "active",
    "circuit_breaker_status": "armed"
  }
}
```

### 10.2 Key Metrics

**Standard Metrics:**
- `worker_target`: configured max concurrent agents
- `live_worker_count`: currently active workers
- `selected_task_ids`: tasks selected in last dispatch wave
- `running_task_ids`: tasks currently executing
- `live_task_ids`: tasks with recent heartbeats (not stale)

**Task Health:**
- `status`: pending, running, completed, failed, blocked
- `heartbeat_age_seconds`: time since last runtime evidence
- `last_adjudication`: last adjudication outcome (verified, rejected, incomplete, etc.)

**Meta-Task Health (Recurring):**
- `fresh`: output is within freshness threshold
- `useful`: output is being acted upon
- `degraded`: experiencing semantic failures or stale context
- `last_outcome_class`: changed, no_change, failed, etc.
- `generated_work_count`: pending work generated by this brain
- `self_authored_work_pending`: proposals awaiting arbitration

**Company OS Gates:**
- `no_ship`: hard no-ship state (blocks all work)
- `freeze_non_urgent_changes`: freezes non-critical work
- `error_budget_status`: healthy, yellow, exhausted, bootstrap
- `allowed_exception_tags`: tags that can bypass gates

### 10.3 Stale Detection

**Heartbeat:** Task runtime evidence (latest_result.json, prompt.md)
- **Stale after:** 90 seconds (configurable `heartbeat_stale_after`)
- **Signal:** task is running but not updating

**Result:** Task adjudication result
- **Stale after:** 30 minutes (configurable `result_stale_after`)
- **Signal:** task outcome is old, may need re-adjudication

**Operator Surface:** Health state
- **Stale after:** 1 hour
- **Signal:** operator view is stale

---

## 11. SESSION STATE & RETRY LOGIC

### 11.1 Session State

**File:** `src/session.rs`, `state/session.json`

Malinka persists runtime session state for recovery:

```json
{
  "selected_task_ids": ["EA-01", "EA-02"],
  "selected_task_scores": [
    {
      "task_id": "EA-01",
      "priority": 1,
      "dependency_depth": 0,
      "potential_conflicts": 0,
      "last_failed_age_seconds": null
    }
  ],
  "failed_tasks": [
    {
      "task_id": "EA-03",
      "last_failed_at_unix_seconds": 1710000000,
      "retry_count": 2
    }
  ],
  "running_tasks": [
    {
      "task_id": "EA-01",
      "claimed_at_unix_seconds": 1710000100
    }
  ],
  "retrying_tasks": [
    {
      "task_id": "EA-03",
      "retry_count": 1,
      "next_retry_at_unix_seconds": 1710000300
    }
  ],
  "provider_cooldowns": [
    {
      "adapter": "codex",
      "reason": "quota_exhausted",
      "until_unix_seconds": 1710001800
    }
  ]
}
```

### 11.2 Failure Tracking

**Failed Task Record:**
- `task_id`: task ID
- `last_failed_at_unix_seconds`: timestamp of last failure
- `retry_count`: cumulative retry attempts

**Dispatch Backoff:**
- Failed tasks in `FAILURE_BACKOFF_WINDOW_SECONDS` (900s) receive dispatch penalty
- Penalty applied to scoring: `last_failed_age_seconds` included in TaskDispatchScore
- Recent failures deprioritized against fresh tasks

### 11.3 Retry Schedule

**Retrying Task Record:**
- `task_id`: task ID
- `retry_count`: current retry attempt
- `next_retry_at_unix_seconds`: when to try again

**Backoff Schedule:**
- Initial: configurable per engine/task
- Exponential: backoff doubles on each retry
- Max: `max_retry_backoff_ms` from WORKFLOW.md (300000ms = 5 minutes default)
- Capped at max, never exceeds

**Trigger conditions:**
- Task failed (failed_tasks record created)
- Task rejected after proof failure
- Task incomplete but productive (preserved for repair)

### 11.4 Provider Cooldowns

**Cooldown Record:**
- `adapter`: provider/engine kind (Codex, Claude, Kimi, Gemini)
- `reason`: auth_failed, quota_exhausted, upstream_outage, etc.
- `until_unix_seconds`: when cooldown expires

**Behavior:**
- Provider cooldown blocks dispatch to that adapter
- Fallback provider tried if available
- Cooldown expires automatically at threshold
- Does NOT poison task failure history

---

## 12. COMPANY OS & DOCTRINE CONTROL

### 12.1 Company OS Header

**File:** `OS.md` front matter

```yaml
---
os_kind: "autonomous_company_os"
os_version: "0.2"
last_updated: "2026-03-16"
company_name: "Malinka"
company_stage: "stage_0_bootstrap"
domain_overlay: "platform"
primary_mission_doctrine: "specs/031226-malinka-v2.md"
hard_invariants_doctrine:
  - "INVARIANTS.md"
kpi_registry_path: "ops/kpi_registry.yaml"
scorecard_path: "ops/scorecard.md"
instrumentation_backlog_path: "ops/instrumentation_backlog.md"
risk_register_path: "ops/risk_register.md"
incident_ledger_path: "ops/incidents/"
decision_log_path: "ops/decision_log.md"
evidence_root: "ops/evidence/"
---
```

**Company Stages:**
- `stage_0_bootstrap`: core platform, missing some operating truth
- `stage_1_pmf`: product-market fit phase
- `stage_2_early_scale`: scaling operations
- `stage_3_operational_scale`: mature operations
- `stage_4_regulated_high_risk`: regulated/high-stakes operations

### 12.2 Doctrine Hierarchy

Recurring brains consume doctrine in priority order:

1. **Mission doctrine** (specs/)
2. **Hard invariants** (INVARIANTS.md)
3. **Company OS** (OS.md)
4. **KPI registry & scorecard** (ops/)
5. **Risk register, decision log** (ops/)
6. **Runtime truth** (state/)

### 12.3 Brain State

**Files:** `src/company_os.rs`, `state/company_os/brains/<role>/latest.json`

Each recurring brain maintains durable state:

```json
{
  "role": "strategy",
  "version": 15,
  "updated_at_unix_seconds": 1710000000,
  "goals": [
    "Replace Autonomy kernel safely",
    "Achieve replacement-readiness"
  ],
  "hypotheses": [
    "Current closure contract is tight enough",
    "Provider failover reduces incidents"
  ],
  "constraints": [
    "Cannot mutate hard invariants",
    "Must preserve proof honesty"
  ],
  "recent_outcomes": [
    "Prioritized EA-01 delivery",
    "Proposed security audit scope"
  ]
}
```

### 12.4 Self-Authored Work & Arbitration

**Self-Authored Proposals:** Brains can propose work for approval

```json
{
  "proposal_id": "prop-st-001",
  "role": "strategy",
  "title": "Prioritize security audit",
  "rationale": "Risk register shows audit debt",
  "evidence": "risk_register.md indicates 3 open findings",
  "budget_class": "high_priority",
  "status": "pending",
  "allowed_paths": ["ralph/IMPLEMENT.md"],
  "proof_commands": [],
  "tags": ["critical"],
  "reflection_id": "refl-st-001"
}
```

**Status:** pending → draft → compiled → expired

**Arbitration:** Board arbitrates cross-brain tensions

```json
{
  "tension_id": "tension-001",
  "summary": "Deploy rate vs stability trade-off",
  "roles": ["strategy", "operations"],
  "status": "unresolved" | "deferred" | "ratified",
  "source_task_ids": ["EA-05", "EA-06"],
  "resolution_lineage": "board-decision-2026-03-16"
}
```

### 12.5 Control Evidence & Land Gates

**Control Evidence State:**
```json
{
  "lane": "platform",
  "status": "green" | "yellow" | "red",
  "no_ship": false,
  "freeze_non_urgent_changes": false,
  "reasons": [
    {
      "summary": "Error budget below threshold"
    }
  ]
}
```

**Gate State (GateState):**
- `no_ship`: hard block on shipping
- `freeze_non_urgent_changes`: only critical work
- `deployment_freeze`: no auto-deploy
- `auto_rollback_only`: rollback-only mode
- `no_new_specs`: stop new specification work
- `error_budget_status`: healthy | yellow | exhausted | bootstrap
- `allowed_exception_tags`: tags that can bypass (e.g., critical, rollback-only)

**Gate Blocking:**
- `no_ship` → blocks delivery
- `freeze_non_urgent_changes` → blocks non-tagged work
- `deployment_freeze` → blocks deploy commands
- `error_budget_status == exhausted` → blocks delivery
- `error_budget_status == bootstrap` → blocks all but initialization work

---

## 13. SEMANTIC ADAPTERS & NORMALIZATION

### 13.1 What They Do

Semantic adapters normalize recurring source artifact output to canonical vocabulary:

**Strategy Aliases:**
```yaml
- id: repo_strategy_lane_aliases
  kind: strategy_operating_lane_aliases
  aliases:
    endurance_gate: decentralized_casino
```

Maps `endurance_gate` → `decentralized_casino` in strategy reports

**Security Aliases:**
```yaml
- id: repo_security_status_aliases
  kind: security_scope_status_aliases
  aliases:
    finding_reported: finding
```

Maps `finding_reported` → `finding` in security scope statuses

### 13.2 Adaptation Rules

- Map repeated field names to canonical terms
- Normalize enum values (e.g., `critical_finding` → `finding`)
- Preserve source nesting and structure
- **Never** widen write scope or trust metadata
- Record normalization in compiler metadata

### 13.3 Validation

Adapted output must:
1. Remain within writable targets
2. Preserve field semantics
3. Pass subsequent schema validation
4. Remain bounded by proof expectations

---

## 14. EXECUTION POLICY & TRUST METADATA

### 14.1 Policy Decisions

**File:** `src/execution_policy.rs`

Every mutating action is adjudicated against policy:

**Decision Kinds:**
- `DeliveryPacket`: delivery task allowed-paths & proof
- `ControlWrite`: spec/plan/OS write requires trust metadata
- `Land`: landing to trunk requires gates & proof
- `BrainState`: recurring brain state mutations require authorization

**Verdicts:**
- `Allowed`: action approved
- `Rejected`: action violates policy, error code specified

**Violation Codes:**
- `DeliveryAllowedPathsMissing`: task has no allowed paths
- `DeliveryAllowedPathsBroad`: allowed paths too broad (should be narrower)
- `DeliveryProofCommandsMissing`: task has no proof commands
- `DeliveryProofCommandsInvalid`: proof command malformed
- `ControlWriteMissingLineage`: spec/plan write missing lineage ID
- `ControlWriteMissingAuthority`: spec/plan write missing authority
- `ControlWriteMissingQualityGate`: spec/plan write missing gate
- `BrainStateDeleteUnauthorized`: brain state deletion not allowed
- `BrainStateCompactionMissingReason`: brain compaction lacks reason
- `BrainStateSelfModificationBlocked`: brain cannot modify itself
- `LandNoShipOpen`: gate is open, cannot land
- `LandExecutionTrustRed`: execution trust surface is red

### 14.2 Trust Metadata

**Required for:**
- Spec/plan writes (origin, lineage, authority, quality gate)
- Control-plane artifact mutations
- Recurring source writes to specs/

**Fields:**
```rust
WriteTrustMetadata {
  lineage_id: Option<&'a str>,      // e.g., lineage-st-03
  authority: Option<&'a str>,       // e.g., ceo_strategy
  quality_gate: Option<&'a str>,    // e.g., approved
}
```

**Sources:**
- Delivery task IMPLEMENT.md fields
- Recurring brain reports (generated_work references)
- Arbitration decisions

### 14.3 Bounded Authorizations

**Brain State Protection:**
```rust
pub struct PolicyAuthorization {
  pub reason: String,               // why this action is allowed
  pub authority: String,            // who authorized it
  pub rollback_path: Option<PathBuf>, // rollback plan if needed
  pub task_id: Option<String>,      // task that triggered it
  pub lineage_id: Option<String>,   // lineage trace
}
```

---

## 15. AUTONOMY FEATURES

### 15.1 Daemon Mode

**Configuration (project.yaml):**
```yaml
autonomy:
  enabled: true
  daemon:
    pid_file: state/daemon.pid
    health_bind: 127.0.0.1:7654
    watchdog_file: state/daemon.watchdog
    watchdog_interval_secs: 30
```

**Operation:**
- Daemonizes after startup
- Writes PID to file
- Listens on health_bind for status queries
- Periodically writes watchdog signal

**Graceful Shutdown:**
- First SIGTERM: stops new dispatch, drains active workers
- Second SIGTERM (if first hangs): force-stops watch

### 15.2 Pre-Authorized Actions

**Configuration (project.yaml):**
```yaml
recurring:
  tasks:
    - id: operations
      pre_authorized_action_classes:
        - inspect
        - status_check
        - log_read
        - deploy_staging  # if approved by ops policy
```

**Meaning:**
- Operations brain can issue these action classes without explicit approval
- Actions still bounded by budget limits and circuit breakers
- Deploy actions require green proof before execution

### 15.3 Budget Enforcement

**Configuration (project.yaml):**
```yaml
autonomy:
  budgets:
    staging_deploys:
      limit: 10
      alert_at: 8
      scope: staging
    production_deploys:
      limit: 2
      alert_at: 1
      scope: production
```

**Tracking:**
- Operations brain projects deployed actions into future task selection
- Budget exhaustion → circuit breaker trips
- Alert at threshold (e.g., 8/10 used)
- Freeze new deployments at exhaustion

**Health Projection:**
- Budget status visible in health.json
- Operators can see remaining capacity
- Automatic action freeze if breached

### 15.4 Circuit Breakers

**Configuration (project.yaml):**
```yaml
autonomy:
  circuit_breakers:
    deployment_freeze_trigger_consecutive_failures: 3
    auto_clear_after_minutes: 60
```

**Behavior:**
- 3 consecutive deployment failures → deployment freeze
- Freeze blocks auto-deploy for 60 minutes
- Operator can manually clear or wait for auto-clear
- Circuit breaker recorded in gate state

### 15.5 Self-Healing

**Configuration (project.yaml):**
```yaml
autonomy:
  self_healing:
    check_interval_minutes: 15
    disk_space:
      threshold_gb: 10
      workspace_retention_days: 7
```

**Checks:**
- Every 15 minutes: check disk space
- If below 10 GB: clean workspaces older than 7 days
- Preserve productive-incomplete workspaces
- Log healing actions in audit

---

## 16. EXECUTION AUDIT & OBSERVABILITY

### 16.1 Audit Trail

**Files:** `state/audit/latest.json`, `state/audit/events.jsonl`

Every significant action is audited:

**Audit Entry:**
```json
{
  "event_id": "evt-20260316-001",
  "timestamp": "2026-03-16T10:30:00Z",
  "event_type": "delivery_completion",
  "task_id": "EA-01",
  "actor": "supervisor",
  "outcome": "verified",
  "evidence": {
    "proof_status": "passed",
    "git_sha": "abc123def456",
    "gate_bypass": false
  }
}
```

**Event Types:**
- `dispatch_selected`: task selected for dispatch
- `dispatch_started`: engine invoked
- `turn_outcome_parsed`: outcome line parsed
- `proof_gate_passed`: proof commands passed
- `proof_gate_failed`: proof commands failed
- `adjudication_completed`: adjudication finished
- `delivery_landed`: task landed to trunk
- `delivery_blocked`: land gate blocked
- `recurring_completed`: recurring task finished
- `recovery_attempted`: recovery/repair run
- `provider_cooldown`: provider failure detected
- `execution_policy_violation`: policy check failed

### 16.2 Execution Metrics

**Files:** `state/health.json`, `ops/scorecard.md`

Key metrics for operator visibility:

**Truthful unattended turn rate (7d):**
```
= turns_with_structured_terminal_outcome_and_coherent_runtime_truth
/ total_turns
```
- **Green:** >= 0.95
- **Yellow:** 0.80-0.949
- **Red:** < 0.80

**Autonomous recovery salvage rate (7d):**
```
= productive_failed_or_rejected_turns_recovered_without_human_edits
/ total_productive_failed_or_rejected_turns
```
- **Green:** >= 0.80
- **Yellow:** 0.50-0.79
- **Red:** < 0.50

**False green proof count:**
```
= count of completion claims whose named proof did not execute
```
- **Green:** 0
- **Yellow:** 1
- **Red:** > 1

---

## 17. OPERATOR COMMANDS & SURFACES

### 17.1 CLI Commands

**Status & Monitoring:**
```bash
malinka status              # Overall platform status
malinka health              # Detailed health state
malinka queue               # Current task queue
malinka list                # List all tasks
malinka results <task-id>   # Task results & adjudication
malinka monitor <task-id> --lines 50  # Task output monitoring
```

**Operators:**
```bash
malinka catalog             # Installed agent catalog
malinka import <path>       # Import external work
```

**Execution:**
```bash
malinka run --native                      # Run supervisor once
malinka run --native --task strategy      # Run single recurring task
malinka run --daemon --watch              # Daemonize with watch
```

### 17.2 State Surfaces

**Queue:** `state/queue.json`
- selected_task_ids
- packets (all admitted work)

**Session:** `state/session.json`
- failed_tasks (with retry count)
- running_tasks
- retrying_tasks
- provider_cooldowns

**Health:** `state/health.json`
- worker status
- task status
- meta-task status
- company OS gates
- execution metrics

**Audit:** `state/audit/latest.json`, `state/audit/events.jsonl`
- Event log
- Provenance

**Task Runtime:** `state/tasks/<task_id>/`
- `latest_result.json`: adjudication result
- `prompt.md`: compiled prompt
- `report.json`: recurring report
- `source-output.json`: recurring source artifact
- `events.jsonl`: task event log

---

## 18. GOTCHAS & LIMITATIONS

### 18.1 Hard Requirements

**Git & Branches:**
- Target repo must be git initialized
- Branch must be named `trunk` (not `main`)
- Remote `origin` must exist pointing to canonical upstream
- Workspaces use `origin/trunk` for drift checks

**Workspace Root:**
- Must be **outside** the repo (recommended)
- Must be writable, with sufficient disk space
- Shared across all tasks (single shared root)

**Proof Honesty:**
- Proof commands must actually execute (not `true` or `echo pass`)
- False-green proof blocks readiness (INV-002)
- Task-level proof overrides must exist as real commands

**Closure Contract:**
- Every turn must end in `RESULT:` or `BLOCKED:` (INV-001)
- Incomplete turns are treated as failures (no optimization for partial progress)
- Only `ProductiveIncomplete` state preserves workspace for repair

### 18.2 Gotchas & Edge Cases

**Dependency Cycles:**
- Detected on task graph build → error
- Plan mutations that introduce cycles cause parsing to fail
- Requires manual cycle breaking in IMPLEMENT.md

**Affinity & Cache Reuse:**
- Affinity key groups cache only if tasks run (affinity penalty applies)
- Cache scope must be consistent (all repo, or all affinity)
- Cache misses are silent (no error, just cache miss)

**Provider Failover:**
- Fallback engine is optional
- If provider cools down and no fallback, work blocks until cooldown expires
- Cooldown does not poison task retry history (task can retry with different provider)

**Recurring Freshness:**
- Freshness threshold is `max(last_artifact_age, last_run_time)`, not just artifact age
- If brain produces unchanged output N times, it yields (skips) until multiplied interval passes
- No explicit "stop stale recurring" – only graceful yield

**Workspace Drift:**
- Drift limit is commits behind `origin/trunk`
- Clean workspace with local-only commits exceeding drift limit → DriftBlocked
- Rebase or merge before retry

**Proof Service Timeouts:**
- Services started for proof have a `ready_timeout_ms` to become ready
- If service fails to become ready, proof errors
- Service remains running until proof completes (or stall timeout)

**Land Gate Bypass:**
- Exception tags allow bypass only if gate declares them in `allowed_exception_tags`
- Bypass is recorded but still goes through execution policy
- Bypass does not skip proof requirements

### 18.3 Not Implemented

- **Interactive prompt override:** Cannot override task fields from CLI
- **Partial plan reload:** Plan is reloaded fresh on each supervisor cycle
- **Task priority dynamics:** Priority is static from IMPLEMENT.md (not learned)
- **Provider auto-selection:** Adapters are explicit in config (not auto-switched on quality)
- **Operator approval gates:** Approval is manual (via IMPLEMENT.md Quality gate field, not interactive)
- **Custom proof report formats:** Proof must succeed/fail, no structured proof metadata
- **Scheduled work:** Cadence is recurring only (no one-shot scheduled work)
- **Affinity cache invalidation:** Cache is never invalidated (persist until cleanup)

---

## 19. NOTABLE ARCHITECTURAL PATTERNS

### 19.1 Shared State Token

**Pattern:** `SharedStateToken` prevents unauthorized writes to queue/session/health

```rust
pub(crate) struct SharedStateToken {
    _private: (),
}
```

Only supervisor can create token (via `shared_state_token()` or `testing_shared_state_token()`). All writes to shared state require token reference. This prevents worker threads or recurring tasks from directly mutating operator surfaces.

### 19.2 Fail-Closed on Evidence Absence

**Pattern:** Missing closure evidence = incomplete, not assumed success

- No structured outcome → `Incomplete` (not assumed working)
- No proof output → proof failed (not assumed passing)
- No allowed paths → delivery blocked (not assumed safe)
- No tree metadata for control writes → rejected (not assumed authorized)

This ensures Malinka's core promise: **truthful unattended execution**.

### 19.3 Bounded Recurring Mutations

**Pattern:** Recurring brains write only to source artifacts; those artifacts feed delivery queue

Benefits:
- Brains cannot directly mutate code/spec (only propose)
- Semantic adapters can normalize without widening scope
- Delivery work inherits trust metadata from brain
- Single point of review: source artifact validation

### 19.4 Atomic Workspace Snapshots

**Pattern:** Workspace delta exported before adjudication; snapshots preserve productive work

If engine produces changes but misses closure contract, snapshot captures those changes. Recovery retry or operator can reuse the snapshot instead of re-running engine.

### 19.5 Lineage Chains

**Pattern:** `Origin`, `Lineage`, `Authority`, `Quality gate` form immutable proof chain

```
Origin: strategy:strategy:repo      # which brain produced this
Lineage: lineage-st-03              # unique ID for tracking
Authority: ceo_strategy             # approval source
Quality gate: approved              # gate status
```

Allows auditing work back to its origin brain and approval chain.

---

## 20. DEPLOYMENT CHECKLIST

To deploy Malinka into another repo:

1. **Git setup:**
   - Rename branch to `trunk`
   - Ensure `origin` points to canonical upstream
   - Add to .gitignore: `state/`, `.malinka-*`, `results/`

2. **Create minimal `project.yaml`:**
   - `name`, `spec_dir`, `plan_file`
   - `proof_commands`
   - `engine` config
   - `workspace.root`

3. **Create minimal `WORKFLOW.md`:**
   - Front matter with tracker, polling, workspace, hooks, agent, codex config
   - Prompt template body

4. **Create minimal `ralph/IMPLEMENT.md`:**
   - One section with starter tasks

5. **Create `INVARIANTS.md`:**
   - At least INV-001 through INV-005 adapted to your repo

6. **Create `OS.md`:**
   - Company OS header with paths to OS artifacts
   - Operating rules and doctrine hierarchy

7. **Build & install:**
   ```bash
   cargo build --release
   ./target/release/malinka run --native
   ```

8. **Iterate:**
   - Add real spec documents in `specs/`
   - Expand IMPLEMENT.md with real work
   - Refine recurring brains (strategy, security)
   - Add company OS control surfaces (scorecard, KPI registry, etc.)

---

## Summary Table

| Component | File | Primary Config | Key Capability |
|---|---|---|---|
| **Task Dispatch** | src/queue.rs, src/task_graph.rs | IMPLEMENT.md | Dependency resolution, conflict detection, ready-set selection |
| **Proof System** | src/adjudicator.rs | project.yaml, IMPLEMENT.md | Fail-closed verification, proof gating, adjudication |
| **Landing** | src/trunk_integrator.rs | IMPLEMENT.md, project.yaml | Atomic git commits, execution policy, gate enforcement |
| **Recurring Strategy** | src/recurring/strategy.rs | project.yaml, OS.md | Doctrine-driven work generation |
| **Recurring Security** | src/recurring/security.rs | project.yaml | Audit scopes, remediation packets |
| **Recurring Operations** | src/recurring/operations.rs | project.yaml | Status checks, health reports |
| **Recurring Learning** | src/recurring/learning.rs | project.yaml | Failure cluster detection, repair proposals |
| **Engine Adapters** | src/engines/ | project.yaml | Codex, Claude, Kimi, Gemini support |
| **Workspaces** | src/workspaces/mod.rs | project.yaml, WORKFLOW.md | Isolation, caching, drift detection |
| **Health** | src/health/mod.rs | project.yaml | Operator visibility, metrics, stale detection |
| **Session** | src/session.rs | — | Failure tracking, retry scheduling, provider cooldowns |
| **Company OS** | src/company_os.rs | OS.md | Doctrine hierarchy, gates, brain state |
| **Execution Policy** | src/execution_policy.rs | — | Trust metadata enforcement, allowed-path validation |
| **Autonomy** | src/supervisor/mod.rs | project.yaml | Daemon mode, circuit breakers, self-healing |

This is Malinka: **truthful, bounded, autonomous development at scale.**
