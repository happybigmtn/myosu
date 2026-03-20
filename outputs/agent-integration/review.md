# `agent-integration` Lane Review

## Lane: `agent-integration`
## Derived from: `agent:experience` (reviewed: KEEP, 2026-03-20)

---

## Judgment: **DEFER** — upstream unblock required before implementation family

The `agent:experience` lane is ready to implement. The `agent:experience` review correctly identifies the lane as well-specified with trusted upstream dependencies. However, the **critical path blocker is not within the product frontier** — it is the `games:traits` lane's incomplete robopoker git migration.

Product should not open an implementation family for `agent:experience` until the robopoker dependency graph is resolved. The robopoker absolute paths block `cargo build` on any clean checkout or CI environment, making implementation slices impossible to verify.

---

## Rationale for DEFER

### 1. Robopoker Git Migration is a Blocker, Not an Implementation Decision

The `agent:experience` review (page 2) states:

> Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This is documented as the highest-priority Slice 1 fix.

The downstream lanes (including `agent:experience` slices 5-9) require full integration testing. Integration testing requires `cargo build` to succeed on a clean environment. The absolute path dependencies make this impossible.

**The robopoker git migration is owned by `games:traits` lane, not by `agent:experience` or `agent-integration`.** Until that lane completes Slice 1 (RF-01: replace absolute paths with git rev), no downstream implementation slice can be verified.

### 2. `myosu-play` Binary Skeleton is Missing

The `myosu-play` binary (defined in `play:tui` lane) is the vehicle for all agent-facing flags (`--pipe`, `--context`, `--narrate`, `--spectate`). The binary does not exist yet.

Impact on `agent:experience` slices:
- **Slice 3** (`--context` flag wiring) — blocked by binary skeleton
- **Slice 4** (`reflect>` prompt) — blocked by binary skeleton
- **Slice 6** (`--narrate` wiring) — blocked by binary skeleton
- **Slice 7** (lobby) — blocked by binary skeleton
- **Slices 8-9** (spectator) — blocked by binary skeleton

Only **Slices 1-2** (`agent_context.rs`, `journal.rs`) can proceed without the binary. These depend only on `tui:shell` which is trusted (82 tests pass).

**Recommendation**: `play:tui` lane should complete its binary skeleton (Slice 1) before or concurrently with `agent:experience` Slices 1-2.

### 3. Chain Discovery is Stubbed — Acceptable for Phase 0

The lobby (Slice 7) requires querying the chain for active subnet information. The `agent:experience` spec correctly identifies this as Phase 4 (depends on `chain:runtime`). Stubbing with hardcoded data for Phase 0 is the right call.

This is **not a blocker** — it's an acceptable Phase 0 limitation that does not prevent implementation or testing of the other slices.

---

## Integration Decision

### What `agent-integration` Produces

This lane produces `agent-adapter.md` — a structural document describing how the agent-facing interfaces (pipe mode, JSON schema, agent context, reflection, narration, spectator relay) connect to the rest of Myosu.

The adapter document is **not an implementation plan**. It is an integration map that:
1. Documents the interface contracts that implementation slices will implement against
2. Identifies the integration points with other lanes (`play:tui`, `games:traits`, `chain:runtime`)
3. Captures key design decisions (object-safe GameRenderer, exhaustive legal_actions, fog-of-war at relay)

### What the Product Needs Next

**Priority 1 — Unblock the critical path:**
The `games:traits` lane must complete Slice 1 (robopoker git migration: replace `/home/r/coding/robopoker/...` absolute paths with `git = "https://github.com/happybigmtn/robopoker", rev = "..."`).

This unblocks:
- All downstream lanes that call into `games:traits` or `tui:shell`
- `agent:experience` Slices 5-9 which require full integration testing
- Any future implementation family for the agent-facing surfaces

**Priority 2 — Unblock agent:experience binary dependency:**
The `play:tui` lane must complete Slice 1 (binary skeleton for `myosu-play`) so that `agent:experience` Slices 3+ can wire their flags to a real binary.

**Priority 3 — Begin agent:experience Slices 1-2 (no binary required):**
Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) depend only on `tui:shell` which is trusted. These can begin immediately and do not require the binary skeleton or robopoker migration.

### Why Not an Implementation Family Now

The `agent:experience` review recommends "proceed to implementation-family workflow." This recommendation is correct **once the blockers are resolved**. Opening an implementation family (multiple parallel slices with separate ownership) before the robopoker git migration and binary skeleton are in place would create verification impossibility:

- Slices cannot be `cargo build`-verified in CI or clean checkout
- Slices 3+ cannot be wired to a binary that doesn't exist
- Integration testing between slices requires a working dependency graph

The honest slice boundary is: **Slices 1-2 can start now; Slices 3-9 must wait for upstream resolution.**

---

## Blockers Summary

| Blocker | Owner | Severity | Blocks |
|---------|-------|----------|--------|
| Robopoker git migration | `games:traits` | HIGH | All downstream lanes, full integration testing |
| `myosu-play` binary skeleton | `play:tui` | HIGH | `agent:experience` Slices 3-9 |
| Chain discovery for lobby | `chain:runtime` | MEDIUM | Slice 7 (Phase 4 — acceptable stub for Phase 0) |
| Spectator socket path convention | `play:tui` | LOW | Slice 8 (confirm alignment with data-dir convention) |

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Integration map (`agent-adapter.md`) | **DONE** | This document |
| Interface contracts documented | **DONE** | 7 interfaces specified in adapter |
| Upstream: `games:traits` (robopoker) | **BLOCKED** | Slice 1 pending; absolute path deps |
| Upstream: `play:tui` (binary skeleton) | **BLOCKED** | Slice 1 pending; binary missing |
| Upstream: `tui:shell` (trusted) | **READY** | 82 tests pass |
| Upstream: `games:traits` (traits) | **READY** | 14 tests pass |
| Implementation slices defined | **READY** | 9 slices, sequential, clean dependency chain |
| Decision made | **DONE** | DEFER — upstream unblock required first |

---

## Recommendation

**Do not open an implementation family for `agent:experience` yet.**

1. **`games:traits` lane** — complete Slice 1 (robopoker git migration) as the critical path unblock
2. **`play:tui` lane** — complete Slice 1 (binary skeleton) to unblock `agent:experience` Slice 3+
3. **`agent:experience` lane** — begin Slices 1-2 (`agent_context.rs`, `journal.rs`) immediately; these have no upstream blockers beyond `tui:shell` which is trusted
4. **After upstream unblocks** — open implementation family for remaining slices; the adapter map in `agent-adapter.md` provides the integration boundaries

The `agent:experience` lane is well-specified. The issue is not readiness of the lane — it is readiness of the critical path dependencies. This is a ** sequencing decision**, not a quality or design concern.

---

## Next Steps

1. Update `fabro/programs/myosu-product.yaml` to reflect that `agent` unit depends on `play` unit completing `reviewed` milestone
2. Track `games:traits` Slice 1 as the critical path item in `ops/decision_log.md`
3. Once robopoker git migration completes, re-review `agent:experience` for implementation family opening
4. `agent:experience` review.md should be updated after each slice completes to track proof availability and remaining blockers
