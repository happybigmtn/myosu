# `agent-integration` Lane Review

## Judgment: **PROCEED — `play:tui` Slice 1 is the next honest implementation step**

The product has two ready upstream lanes (`tui:shell`, `games:traits`), one
reviewed product lane (`agent:experience`), and one absent binary (`myosu-play`).
The binary does not depend on `chain:runtime`. It depends on the two trusted
lanes. The next honest step is to implement `play:tui` Slice 1 immediately,
which unblocks `agent:experience` Slices 3–9 in parallel.

---

## State of the Product Surface

### Trusted Lanes (bootstrap complete)

| Lane | Status | Evidence |
|------|--------|---------|
| `games:traits` | **IMPLEMENTED + VERIFIED** | Git rev pinned; 10 unit + 4 doctest pass; `outputs/games/traits/verification.md` complete |
| `tui:shell` | **TRUSTED** | 82 tests pass; `GameRenderer`, `PipeMode`, `schema.rs` all trusted |

### Reviewed Lanes (bootstrap complete; awaiting implementation)

| Lane | Status | Blocker |
|------|--------|---------|
| `agent:experience` | **READY (KEEP)** | `play:tui` Slice 1 (`myosu-play` binary) — required for Slices 3–9 |
| `play:tui` | **READY (reviewed)** | Same blocker: `myosu-play` binary is completely absent |

### Restart Lanes (requires rebuild from scratch)

| Lane | Status | Phase |
|------|--------|-------|
| `chain:runtime` | **RESTART** | Phase 0 — workspace wiring; no compile path yet |
| `chain:pallet` | **RESTART** | Phase 1 — strip non-Myosu modules, fix deps |

---

## Decision: Implementation Family vs. Upstream Unblock

### The False Choice

There are two framing questions:
1. Should the product begin an **implementation family** (execute a lane's slices)?
2. Should it first **unblock an upstream** (fix a blocking dependency)?

For `agent:experience`, the answer is: **both, in the right order**.

### The Two-Knobs Picture

**Knob 1 — `agent:experience` Slices 1–2**: Can begin immediately.
These are `agent_context.rs` and `journal.rs`. They depend only on
`tui:shell` and `games:traits` — both already trusted. No `myosu-play`
binary required.

**Knob 2 — `agent:experience` Slices 3–9 + `play:tui` Slice 1**: Both blocked
on the `myosu-play` binary. The binary does not require `chain:runtime`.
It requires `tui:shell` (trusted) and `games:traits` (trusted). The binary
is a local CLI scaffold that works in training mode.

### What `chain:runtime` Does and Doesn't Block

`chain:runtime` restart must proceed — it is the foundation for miner
discovery, subnet queries, and WebSocket spectator upgrades. But it does
**not** block:

- `play:tui` Slice 1 (the binary skeleton)
- `agent:experience` Slices 1–2 (agent_context + journal)
- Any training-mode functionality

`chain:runtime` **does** block:

- `play:tui` Slice 7 (chain discovery in lobby)
- `agent:experience` Slice 7 (lobby with live data)
- `agent:experience` Phase 1 (miner-connected pipe mode)

---

## Recommendation

### Immediate (can start today without `chain:runtime`):

**1. `play:tui` Slice 1 — `myosu-play` binary skeleton**

The `myosu-play` binary is a ` clap` CLI with `--train`/`--chain`/`--pipe`
dispatch. Its Slice 1 is a bare-bones scaffold: creates `NlheRenderer`
(hardcoded states), wires into `Shell` from `myosu-tui`, proves the render
loop compiles. This is the exact dependency that `agent:experience` Slices
3–9 require.

```
$ cargo build -p myosu-play
$ myosu-play --train
# renders 5-panel layout without panic
```

**2. `agent:experience` Slices 1–2 — `agent_context.rs` + `journal.rs`**

Can proceed in parallel with `play:tui` Slice 1. Both slices depend only on
trusted upstream. They produce the persistence layer (`AgentContext`,
`Journal`) that `--context` and `reflect>` require.

### Parallel (restart lane, does not block product lanes):

**`chain:runtime` Phase 0 — workspace wiring**

Establish the compile path for a minimal Substrate runtime. This is the
longest-lead-time item and should proceed without blocking the product lanes.

### What to Sequence After

Once `play:tui` Slice 1 is complete and `agent:experience` Slices 1–2 are
done:

1. `agent:experience` Slices 3–6 (--context wiring, reflect>, narration)
   become unblocked — all depend on the `myosu-play` binary
2. `play:tui` Slices 2–6 proceed in their natural order
3. `chain:runtime` Phase 1+ continues (runtime + node + custom pallets)
4. `chain:pallet` proceeds after `chain:runtime` Phase 1

---

## Rationale Summary

| Factor | Assessment |
|--------|------------|
| `tui:shell` | Trusted — 82 tests pass |
| `games:traits` | Verified — git rev pinned, 14 tests pass |
| `agent:experience` spec | KEEP — well-reasoned, 9 sequential slices |
| `agent:experience` review | READY — no blockers beyond binary |
| `play:tui` spec | Reviewed — binary is the only missing surface |
| `myosu-play` binary | Can begin without chain:runtime |
| `chain:runtime` | Restart required; long lead time; parallel track |

The false choice between "implementation family" and "upstream unblock" resolves
by noting that `play:tui` Slice 1 is simultaneously:
- An **upstream unblock** for `agent:experience`
- An **implementation family step** for `play:tui`
- **Independent of `chain:runtime`** (training-mode CLI)

Therefore: proceed with `play:tui` Slice 1 immediately. Let
`chain:runtime` restart proceed in parallel. Revisit the chain-connected
slices (play:tui Slice 7, agent:experience Slice 7+) once
`chain:runtime` Phase 1 is proven.

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `myosu-play` binary scope creep | Medium | Delays agent:experience | Slice 1 is bare scaffold only; no game logic |
| `chain:runtime` restart takes longer than expected | High | Lobby slices deferred | Phase 0 lobby stubs already specced in agent:experience Slice 7 |
| `agent:experience` Slices 1–2 done before binary ready | Low | Wasted work | Slices 1–2 are additive; can merge anytime |
| `robopoker` API breaks on git update | Low | Breaks `games:traits` | rev is pinned; update is intentional, not automatic |
