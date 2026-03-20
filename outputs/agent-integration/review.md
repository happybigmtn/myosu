# `agent:integration` Lane Review

## Judgment: **READY — proceed to implementation family**

`agent:experience` is the last remaining ready product lane. The upstream dependencies (`tui:shell`, `games:traits`) are trusted. The `agent:experience` lane itself is reviewed with a **KEEP** judgment. The product should proceed to the `agent:experience` implementation family now, beginning with Slices 1-2.

The robopoker git migration is the primary upstream blocker, but it is owned by the `games:traits` lane and should proceed in parallel — not as a gate that halts `agent:experience` Slices 1-2.

---

## Rationale

### 1. `agent:experience` review is clean

The `agent:experience` review (`outputs/agent/experience/review.md`) assigns **KEEP — proceed to implementation-family workflow**. The spec quality is high (AX-01..05 + SP-01..03 are sound), upstream is trusted (82 + 14 tests pass), the schema is the strongest surface (939 lines, 16 tests, covers 10 game types), and the slice dependency chain is clean.

### 2. Upstream lanes are trusted

| Lane | Tests | Status |
|------|-------|--------|
| `tui:shell` | 82 pass | **TRUSTED** |
| `games:traits` | 14 pass | **TRUSTED** |
| `play:tui` | binary missing | Blocks Slice 3+ |

Slices 1-2 (`agent_context.rs` + `journal.rs`) require only `tui:shell`, which is trusted. They can begin immediately without waiting for `play:tui` or `robopoker` resolution.

### 3. robopoker git migration is parallel work, not a gate

The robopoker absolute-path dependency is the primary blocker for full integration testing, but:
- It is owned by the `games:traits` lane
- It does not block Slice 1-2 implementation (those slices don't exercise the robopoker dep chain)
- Slices 1-2 can be implemented and test-driven against mock `GameRenderer` inputs
- The `games:traits` lane should proceed in parallel to unblock integration testing

### 4. `play:tui` binary skeleton is the true gate for Slice 3+

The `--context`, `--narrate`, and `--spectate` flags all require modifications to `myosu-play`'s `main.rs` CLI dispatch. The binary skeleton does not exist yet. This is the constraining resource for Slices 3-9.

However, this does not block Slices 1-2 or the design work for Slices 3-9.

---

## Decision: What the Product Needs Next

### Primary path: `agent:experience` implementation family

Begin Slices 1-2 immediately:
- **Slice 1**: `agent_context.rs` — `AgentContext` with load/save/default; identity/memory/journal fields; roundtrip test
- **Slice 2**: `journal.rs` — append-only markdown writer; hand entry formatter; never-truncates invariant

These can be implemented against mock `GameRenderer` inputs without requiring `myosu-play` binary or robopoker.

### Parallel path: unblock `play:tui` binary skeleton

`play:tui` lane Slice 1 (binary skeleton) must complete before Slices 3-9 can be wired to the CLI. Track this as a dependent milestone.

### Parallel path: robopoker git migration

`games:traits` lane owns this. It must complete before Phase 1 integration testing can run, but not before Slice 1-2 implementation begins.

---

## Lane Readiness for `agent:integration`

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03; 9 slices defined |
| `agent:experience` review | **KEEP** | Proceed to implementation family |
| `tui:shell` upstream | **TRUSTED** | 82 tests pass |
| `games:traits` upstream | **TRUSTED** | 14 tests pass |
| `play:tui` binary | **MISSING** | Blocks Slice 3+; owned by `play:tui` lane |
| robopoker git dep | **BLOCKED** | Owned by `games:traits` lane |
| Schema (`schema.rs`) | **TRUSTED** | 16 tests; covers 10 game types |
| Slices 1-2 implementable | **YES** | No external deps beyond trusted `tui:shell` |

---

## Recommendation

**Proceed to `agent:experience` implementation family now.** Do not wait for robopoker or `play:tui` to unblock Slices 1-2.

The `agent:experience` lane is the last remaining ready product lane. Starting it signals that the product bootstrap is complete and the team is ready to build implementation-facing surfaces.

Track the following as parallel work:
1. `games:traits` lane: robopoker git migration (unblocks integration testing)
2. `play:tui` lane: binary skeleton Slice 1 (unblocks Slices 3-9 CLI wiring)

Do not create an implementation-family lane for `agent:experience` yet — the lane itself should produce its own implementation artifacts first (Slices 1-2 code), then the decision about whether to create a dedicated implementation-family program can be revisited.
