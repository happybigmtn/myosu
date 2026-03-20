# `agent:integration` Lane Review

## Judgment Summary

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **COMPLETE** | AX-01..05 + SP-01..03 defined; 9 slices with clear boundaries |
| `agent:experience` review | **COMPLETE** | Judgment: KEEP; 9 sequential slices |
| Adapter surfaces in `myosu-tui` | **PARTIAL** | `GameRenderer`, `PipeMode`, `schema.rs` trusted; new modules absent |
| CLI binary (`myosu-play`) | **ABSENT** | `crates/myosu-play/` does not exist; blocks Slice 3+ |
| robopoker git migration | **UNRESOLVED** | Blocks full integration testing; does not block Slices 1-2 |

**Recommendation**: Proceed with **two parallel tracks**:
1. Start `agent:experience` Slices 1–2 (library modules only; no binary required)
2. Start `play:tui` Slice 1 (binary skeleton) concurrently to unblock `agent:experience` Slice 3

---

## `agent:experience` Lane Assessment

### Spec Quality: SOUND

The `agent:experience` lane spec (`outputs/agent/experience/spec.md`) defines a coherent set of agent-facing surfaces:

| Surface | Spec Clarity | Implementation Status |
|---------|-------------|---------------------|
| `--pipe` mode | Clear | `PipeMode` exists, 6 tests |
| `--context` flag | Clear | Absent |
| `--narrate` flag | Clear | Absent |
| `reflect>` prompt | Clear | Absent |
| Lobby game selection | Clear | Absent |
| `SpectatorRelay` (Phase 0) | Clear | Absent |
| `SpectateScreen` | Clear | Absent |
| Agent context file | Clear | Absent |
| Agent journal | Clear | Absent |

The 9-slice decomposition is honest: each slice has a clear boundary and a single proof gate. The dependency chain is correct (Slices 1→2→3→4 are sequential; Slices 5→6 are sequential; Slice 7 depends on 3; Slice 8 depends on binary; Slice 9 depends on 8).

### Review Quality: SOUND

The `agent:experience` lane review (`outputs/agent/experience/review.md`) correctly identifies:
- **HIGH blockers**: robopoker git migration, `myosu-play` binary absence
- **MEDIUM blocker**: chain discovery stubbed for lobby (acceptable for Phase 0)
- **LOW blocker**: spectator socket path convention (needs confirmation against play:tui data dir)

The proof expectations are specific and verifiable. The recommendation to "proceed to implementation-family workflow" is well-reasoned.

---

## Adapter Layer Assessment

### What Already Exists (Trusted)

These surfaces are **already in `myosu-tui`** and should not be rewritten:

| Surface | Path | Tests |
|---------|------|-------|
| `GameRenderer` trait | `crates/myosu-tui/src/renderer.rs` | 8 tests |
| `PipeMode` | `crates/myosu-tui/src/pipe.rs` | 6 tests |
| `schema.rs` (`GameState`, `LegalAction`) | `crates/myosu-tui/src/schema.rs` | 16 tests |
| `docs/api/game-state.json` | `docs/api/game-state.json` | — |

### What Needs to Be Built (New Modules)

| Module | Path | Slice | robopoker dep? |
|--------|------|-------|---------------|
| `agent_context.rs` | `crates/myosu-tui/src/` | 1 | **No** |
| `journal.rs` | `crates/myosu-tui/src/` | 2 | **No** |
| `narration.rs` | `crates/myosu-tui/src/` | 5 | **No** (uses `GameState` from schema) |
| `pipe.rs` extensions (`--context`, `--narrate`, `reflect>`, lobby) | `crates/myosu-tui/src/pipe.rs` | 3, 4, 6, 7 | **No** |
| `spectate.rs` | `crates/myosu-play/src/spectate.rs` | 8 | **No** |

**Key finding**: Slices 1–2 (`agent_context.rs`, `journal.rs`) have **zero external dependencies** beyond what `myosu-tui` already exports. These can start immediately.

---

## Blocker Analysis

### Blocker 1: `myosu-play` Binary Does Not Exist

**Severity**: HIGH for Slices 3+; **NONE** for Slices 1–2

The `myosu-play` binary at `crates/myosu-play/src/main.rs` does not exist. This blocks:
- Slice 3 (`--context` flag wiring to CLI)
- Slice 6 (`--narrate` flag wiring to CLI)
- Slice 7 (lobby rendering)
- Slice 8 (`SpectatorRelay`)
- Slice 9 (`SpectateScreen`)

**Resolution**: Start `play:tui` Slice 1 (binary skeleton) concurrently with `agent:experience` Slices 1–2. These are independent work tracks that converge at Slice 3.

**Evidence**: `outputs/play/tui/spec.md` explicitly defines this as its Slice 1. The `play:tui` lane is in the `product` program under the `play` unit with `agent` as a dependent.

### Blocker 2: `robopoker` Git Migration Unresolved

**Severity**: MEDIUM for Phase 1 integration testing; **NONE** for Slices 1–2

The `robopoker` dependency uses **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This blocks:
- Full integration testing of Slices 5–9 (requires actual game state from robopoker)
- Clean checkout / CI environments

**Resolution for Slices 1–2**: Not required. These modules use only `schema.rs` types and serde JSON — no robopoker calls.

**Ownership**: `games:traits` lane owns robopoker resolution. This is documented in `outputs/games/traits/review.md`.

### Blocker 3: `play:tui` Binary Skeleton Needed Concurrently

**Severity**: HIGH for Slice 3+ convergence

`agent:experience` Slice 3 (`--context` wiring) requires `myosu-play`'s CLI dispatch to add the `--context` flag. Without the binary skeleton, the flag cannot be wired.

**Resolution**: Both tracks (play:tui Slice 1 and agent:experience Slices 1–2) can run in parallel. The first convergence point is when agent:experience Slice 3 needs the CLI.

---

## Decision: Implementation Family vs. Upstream Unblock

**Answer: BOTH — run in parallel**

The `agent:experience` lane is ready to produce code. The upstream `play:tui` binary is a **concurrent dependency**, not a **blocking dependency** for all of agent:experience.

```
Parallel Track A: agent:experience
  Slice 1 (agent_context.rs)   → can start NOW
  Slice 2 (journal.rs)        → depends on Slice 1
  Slice 3 (--context wiring)  → BLOCKED by play:tui Slice 1

Parallel Track B: play:tui
  Slice 1 (binary skeleton)   → can start NOW
  Slice 2 (NlheRenderer)      → depends on Slice 1
  Slice 3 (TrainingTable)     → depends on Slice 2
```

**Why not an "upstream unblock" first?**
- `play:tui` Slice 1 (binary skeleton) is a trivial scaffolding task — not a complex unblock
- Starting `agent:experience` Slices 1–2 immediately is honest work that can be reviewed independently
- The `robopoker` git migration is owned by `games:traits`, not `play:tui` or `agent:experience`

**Why not a full "implementation family" next?**
- Creating a new `agent:experience-implementation` frontier program is premature — the lane has only 9 slices, all within `myosu-tui` and `myosu-play` crates
- The current `product` program's `agent` unit is the right granularity

---

## Proof Expectations for This Lane

This lane is **proven** when:

| Evidence | How to Verify |
|----------|--------------|
| `agent:experience` spec + review complete | Files exist at `outputs/agent/experience/` |
| Adapter spec written | File exists at `outputs/agent-integration/agent-adapter.md` |
| Two parallel tracks identified | This review documents both |
| Slice 1 of `agent:experience` can start immediately | `agent_context.rs` has zero external deps beyond `myosu-tui` |

---

## What This Lane Owns

The `agent-integration` lane is a **review and adapter specification lane**. It:

1. **Reviews** the `agent:experience` lane outputs (spec + review) — ✅ complete
2. **Specifies** the adapter layer mapping spec → code surfaces — ✅ complete (this document's companion `agent-adapter.md`)
3. **Decides** whether to proceed to implementation or unblock upstream — ✅ complete (proceed in parallel)
4. **Does NOT own** implementation of the 9 slices — that belongs to the `agent:experience` lane

---

## Recommendation

**Proceed with two parallel tracks**:

1. **`agent:experience` Lane**: Start Slices 1–2 (`agent_context.rs`, `journal.rs`) immediately. These are pure library modules with no binary dependency and no robopoker dependency. Proof gate: `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` and `cargo test -p myosu-tui journal::tests::append_hand_entry`.

2. **`play:tui` Lane**: Start Slice 1 (binary skeleton) concurrently. Proof gate: `cargo build -p myosu-play` exits 0.

**Convergence**: When both tracks complete their first slices, `agent:experience` Slice 3 (`--context` wiring) can proceed with the binary in place.

**No new frontier program needed**. The `product` program's `agent` unit is the right home for `agent:experience` implementation.
