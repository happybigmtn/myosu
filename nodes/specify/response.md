Both artifacts created. Let me provide the final summary.

---

## `agent-integration` Lane — Execution Summary

### What was produced

**`outputs/agent-integration/agent-adapter.md`** — The integration contract for `agent:experience`. Documents:
- The upstream dependency graph (`tui:shell`, `games:traits`, `play:tui`)
- The five downstream surfaces agents consume (pipe protocol, JSON schema, context file, journal, spectator relay)
- Phase 0 vs Phase 1 capability split (Phase 0 works offline; Phase 1 requires chain)
- Critical path to `agent:experience` Slice 1 (depends only on trusted upstream; can start today)

**`outputs/agent-integration/review.md`** — The integration decision. Judges:
- **`PROCEED`** — `play:tui` Slice 1 is the next honest implementation step
- `agent:experience` Slices 1–2 can start immediately (no binary required)
- `agent:experience` Slices 3–9 + `play:tui` Slices 2–6 all depend on the `myosu-play` binary, which requires `play:tui` Slice 1 first
- `chain:runtime` restart is the parallel long-lead track; it does **not** block the product lanes

### The Decision Logic

```
tui:shell ──trusted──► play:tui Slice 1 ──unblocks──► agent:experience Slices 3-9
games:traits ─trusted─► myosu-play binary              agent:experience Slices 1-2
                       (NO chain:runtime needed)         can start NOW (parallel)
                              │
                              └──► chain:runtime restart (PARALLEL, long lead)
```

The `myosu-play` binary is a local CLI that works in training mode. It does
not query the chain. Its only upstream dependencies are `tui:shell` (82 tests)
and `games:traits` (14 tests, git-pinned) — both already trusted.

**Next honest step**: `play:tui` Slice 1 — bare binary skeleton with `--train`
dispatch, hardcoded `NlheRenderer`, wires into `Shell`. This simultaneously
unblocks `agent:experience` and delivers the first consumer-facing product surface.

`★ Insight ─────────────────────────────────────`
The integration question wasn't "implementation family vs upstream unblock" — it
was "which upstream unblock is on the critical path and doesn't itself depend
on the longest-lead-time item (chain:runtime)?" The answer: the `myosu-play`
binary. It depends only on already-trusted lanes, works in training mode
(offline), and is the exact thing blocking 9 of 9 `agent:experience` slices.
`─────────────────────────────────────────────────`