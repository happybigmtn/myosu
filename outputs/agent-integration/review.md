# `agent-integration` Lane Review

## Judgment Summary

**Judgment: PROCEED with Phase 0 implementation — Slices 1, 2, 5 only. Slices 3–9 wait on `play:tui` Slice 1.**

The `agent:experience` review says "KEEP — proceed to implementation-family workflow." This lane confirms that is the correct next step **for three slices only**: Slice 1 (`agent_context.rs`), Slice 2 (`journal.rs`), and Slice 5 (`narration.rs`). These three slices depend only on already-trusted upstreams (`tui:shell`, `games:traits`) and can begin immediately.

The remaining six slices (3–9) are blocked on the `myosu-play` binary skeleton, which is owned by the `play:tui` lane and not yet created.

---

## Decision: Implementation Family, But Narrow

**Verdict: Implementation family for Phase 0 slices only.**

The question was: does product need an implementation family next, or another upstream unblock?

The honest answer is: **both are true, but on different timelines**.

1. **Implementation family is ready now** for Slices 1, 2, 5. These have no external blockers and can be executed by an implementation agent immediately using the proof gates defined in `outputs/agent/experience/spec.md`.

2. **Upstream unblock is still needed** for Slices 3–9. The `play:tui` lane owns the binary skeleton (Slice 1 of that lane). Until it exists, CLI flag wiring, lobby mode, and spectator relay cannot be wired to the binary.

3. **Full integration testing requires** `games:traits` Slice 1 (robopoker git migration). This is tracked in `outputs/games/traits/review.md` Risk 1.

---

## Phase 0 Slice Readiness

| Slice | Module | Upstream Status | Can Execute Now? |
|-------|--------|-----------------|-----------------|
| 1 | `agent_context.rs` | `tui:shell` TRUSTED | **YES** |
| 2 | `journal.rs` | `tui:shell` TRUSTED | **YES** |
| 3 | `--context` flag wiring | `myosu-play` MISSING | NO |
| 4 | `reflect>` prompt | `pipe.rs` extend | NO (blocked by Slice 3) |
| 5 | `narration.rs` | `games:traits` TRUSTED | **YES** |
| 6 | `--narrate` flag wiring | `myosu-play` MISSING | NO |
| 7 | Lobby + game selection | `myosu-play` MISSING | NO |
| 8 | `SpectatorRelay` | `myosu-play` MISSING | NO |
| 9 | `SpectateScreen` | Slice 8 | NO |

**Phase 0 = Slices 1, 2, 5.** Three slices. All upstreams trusted.

---

## Concrete Evidence

### Why Slices 1, 2, 5 can proceed

- `agent_context.rs` uses only `serde` (std) + `tui:shell`'s `Shell` types. No robopoker calls.
- `journal.rs` is a pure file-writer. No game state, no chain, no binary.
- `narration.rs` takes `GameState` (from `games:traits`) and emits `String`. No chain, no binary.

### Why Slices 3–9 are blocked

All require modifications to `myosu-play`'s `main.rs` CLI dispatch, or depend on the binary existing as a cargo package target. The `play:tui` lane review (`outputs/play/tui/review.md`) explicitly marks the binary skeleton as "Slice 1: must be created first."

### Why robopoker migration matters for later

Even Slices 1, 2, 5 call through `games:traits` → `tui:shell` → eventually robopoker. On a clean checkout (no local `/home/r/coding/robopoker/`), `cargo build` will fail. However, the **implementation work itself** can proceed on a machine with the local path. Full CI requires `games:traits` Slice 1 first.

---

## Proof Gate for Phase 0

```bash
# All three runnable without myosu-play binary or robopoker git migration

# Slice 1: AgentContext load/save
cargo test -p myosu-tui agent_context::tests

# Slice 2: Journal append-only
cargo test -p myosu-tui journal::tests

# Slice 5: Narration board texture
cargo test -p myosu-tui narration::tests
```

If any of these fail, the implementation agent should fix the failing module before proceeding.

---

## Remaining Blockers for Full Lane Completion

| Blocker | Severity | Owner | Status |
|---------|----------|-------|--------|
| `myosu-play` binary skeleton | **HIGH** | `play:tui` Slice 1 | Not started |
| robopoker git migration | **HIGH** (CI) | `games:traits` Slice 1 | Not started |
| Spectator socket path alignment | LOW | `play:tui` + `agent:experience` | Not confirmed |
| Chain discovery for lobby | MEDIUM | Can stub for Phase 0 | Stub acceptable |

---

## Lane Readiness for Phase 0

| Dimension | Status |
|-----------|--------|
| Specification | **READY** — `agent:experience/spec.md` defines 9 slices |
| Upstream (`tui:shell`) | **TRUSTED** — 82 tests |
| Upstream (`games:traits`) | **TRUSTED** — 14 tests |
| Binary target (`play:tui`) | **MISSING** — blocks Slices 3–9 |
| Phase 0 slices (1, 2, 5) | **READY TO EXECUTE** |
| Integration risks | **IDENTIFIED** — documented in `agent-adapter.md` |

---

## Recommendation

**Proceed to implementation-family workflow for Phase 0 slices (1, 2, 5) immediately.**

The implementation agent should:
1. Create `crates/myosu-tui/src/agent_context.rs` (Slice 1)
2. Create `crates/myosu-tui/src/journal.rs` (Slice 2)
3. Create `crates/myosu-tui/src/narration.rs` (Slice 5)
4. Run the proof gates above; all must pass

After Phase 0:
- Update `agent:experience/spec.md` to mark Slices 1, 2, 5 as complete
- Await `play:tui` Slice 1 completion before proceeding to Slices 3–9
- Track `games:traits` Slice 1 (robopoker git migration) as a parallel dependency for full integration testing

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent-integration/agent-adapter.md` | Integration contract and slice dependency map |
| `outputs/agent/experience/spec.md` | Full `agent:experience` spec (9 slices) |
| `outputs/agent/experience/review.md` | `agent:experience` judgment: KEEP → implementation family |
| `outputs/games/traits/review.md` | `games:traits` judgment; robopoker Risk 1 tracked |
| `outputs/play/tui/review.md` | `play:tui` judgment: KEEP → implementation family; binary missing |
| `fabro/programs/myosu-bootstrap.yaml` | Raspberry program manifest |
