# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow for product frontier

The `agent:experience` lane has produced complete, reviewed artifacts (`spec.md` and `review.md`) with a **KEEP** judgment. All upstream dependencies are trusted or ready. The product frontier can move to an implementation-family workflow.

---

## Rationale for KEEP

### 1. `agent:experience` Is Fully Specified and Reviewed

The `agent:experience` lane spec (`outputs/agent/experience/spec.md`) defines 9 implementation slices (Slices 1–9), all surfaces, all integration contracts, and the complete phase ordering. The companion review (`outputs/agent/experience/review.md`) confirms:

- **KEEP** judgment with implementation-family workflow recommended
- Schema (`schema.rs`) is **TRUSTED** — 16 tests passing, 939 lines, 10 game types
- `tui:shell` (82 tests) and `games:traits` (14 tests) are **TRUSTED** upstream
- 9 slices are sequential and minimally coupled

### 2. All Upstream Dependencies Are Trusted or Unblocked

| Upstream Lane | Status | Blocking This Lane? |
|--------------|--------|-------------------|
| `tui:shell` | TRUSTED (82 tests) | No |
| `games:traits` | TRUSTED (14 tests) | No |
| `play:tui` | KEEP, ready for implementation | No ( Slice 1 creates `myosu-play` binary) |
| `spectator-protocol` | Spec only (SP-01..03) | No (not needed for Phase 1–3) |
| `chain:runtime` | Not started | No (Phase 4 only) |

### 3. The Two HIGH Blockers Are Owned Elsewhere

The `agent:experience/review.md` identifies two HIGH blockers:

| Blocker | Owner | Status |
|---------|-------|--------|
| robopoker git migration | `games:traits` lane | KEEP, implementation unblocked |
| `myosu-play` binary missing | `play:tui` lane | KEEP, ready for implementation |

Both blocking items are tracked in lanes that are themselves **KEEP** with implementation unblocked. Neither represents an unaddressed risk to the product frontier.

### 4. Product Frontier Is the Last Ready Lane

The product frontier consists of:
- `play:tui` — **KEEP**, ready for implementation-family workflow
- `agent:experience` — **KEEP**, proceed to implementation-family workflow

Both lanes have completed bootstrap and produced reviewed artifacts. The product frontier is ready for the next phase.

---

## Decision: Implementation Family Next

The question is: **should product move to an implementation-family workflow next, or does it need another upstream unblock?**

**Answer: Implementation family next.**

**Reasoning:**

1. **No upstream unblocks needed for Phase 1–3**. Slices 1–4 of `agent:experience` (agent context, journal, --context flag, reflect prompt) depend only on `tui:shell` which is already trusted.

2. **The blocking items are owned and tracked**. The robopoker git migration and `myosu-play` binary are being addressed in their respective lanes (`games:traits` and `play:tui`), both of which have KEEP judgments.

3. **Phase 4 chain integration can proceed in parallel**. The Phase 4 integration points (lobby chain query, spectator WebSocket upgrade) depend on `chain:runtime`, but the Phase 1–3 surfaces (agent context, journal, narration, pipe mode) do not.

4. **The spec is complete and unambiguous**. 9 slices defined with clear boundaries, no design decisions pending, all integration contracts documented in `agent-adapter.md`.

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available after implementation:

| Proof | How to Verify |
|-------|--------------|
| Agent context roundtrips | `AgentContext::load()` → play 10 hands → `AgentContext::save()` → restart → context preserved |
| Journal is append-only | Write 1000 entries; verify file never truncates |
| Reflection prompt works | Pipe mode emits `HAND COMPLETE` + `reflect>`; empty skips; non-empty saved |
| Narration preserves game state | Same `GameState` produces identical terse + narrated output |
| Lobby presented without subnet | `myosu-play --pipe` → lobby → `info 1` → subnet detail |
| Spectator relay enforces fog-of-war | Hole cards never appear during active play |

---

## Remaining Blockers

### 1. robopoker Git Migration (HIGH — tracked in `games:traits`)

Both `tui:shell` and `games:traits` depend on robopoker via absolute filesystem paths. This must be resolved before Phase 1 integration testing can proceed in CI.

**Owner**: `games:traits` lane

**Resolution path**: `games:traits` Slice 1 — replace `path = "/home/r/coding/robopoker/..."` with `git = "https://github.com/happybigmtn/robopoker", rev = "..."`

### 2. `myosu-play` Binary Missing (HIGH — tracked in `play:tui`)

The `myosu-play` binary is the delivery vehicle for all `agent:experience` surfaces. It does not exist yet.

**Owner**: `play:tui` lane, Slice 1

**Resolution path**: Create `crates/myosu-play/src/main.rs` with CLI dispatch for `--pipe`, `--context`, `--narrate`, `--spectate`

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **COMPLETE** | 9 slices defined, all surfaces documented |
| `agent:experience` review | **KEEP** | Implementation-family workflow recommended |
| Upstream (`tui:shell`) | **TRUSTED** | 82 tests pass |
| Upstream (`games:traits`) | **TRUSTED** | 14 tests pass |
| Integration adapter | **DEFINED** | `agent-adapter.md` documents all integration contracts |
| `myosu-play` binary | **BLOCKED** | `play:tui` lane owns; KEEP with implementation ready |
| robopoker git migration | **BLOCKED** | `games:traits` lane owns; KEEP with implementation unblocked |
| Phase 4 chain integration | **FUTURE** | Blocked on `chain:runtime`; not needed for Phase 1–3 |

---

## Recommendation

**Proceed to implementation-family workflow next** for the product frontier.

The `agent:experience` lane is the last remaining ready product lane. Its artifacts are complete and reviewed. The implementation slices are defined and ordered. The blocking items (robopoker migration, `myosu-play` binary) are owned by lanes that are themselves ready for implementation.

**Immediate next steps**:
1. Begin `agent:experience` Slices 1–4 (agent context, journal, --context flag, reflect prompt) — no external dependencies beyond trusted `tui:shell`
2. `play:tui` lane begins Slice 1 (binary skeleton) — unblocks `agent:experience` Slice 3
3. `games:traits` lane proceeds with Slice 1 (robopoker git migration) — unblocks CI for all downstream lanes

The `agent-integration/agent-adapter.md` artifact documents the integration contracts for reference during implementation.
