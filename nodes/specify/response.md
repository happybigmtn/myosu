`★ Insight ─────────────────────────────────────`
The `agent-integration` lane is a **review-and-adapter layer**, not an implementation lane. It consumes the `agent:experience` spec and review (already KEEP'd) and produces two things: (1) an `agent-adapter.md` capability spec documenting the wire protocol and adapter types, and (2) a `review.md` delivering the **PROCEED** judgment — the product is ready for implementation-family workflow without any additional upstream unblocks.
`─────────────────────────────────────────────────`

---

## Deliverables Complete

**`outputs/agent-integration/agent-adapter.md`** — Capability spec defining:
- Wire protocol: pipe mode (`--pipe`) with plain-text `STATE` lines and `HAND COMPLETE` + `reflect>` turn structure
- JSON schema mode: `GameState` + `LegalAction` via `schema.rs` (trusted, 16 tests)
- Context file schema: `AgentContext` with identity, memory, journal
- Journal format: append-only markdown with session/hand/reflection structure
- Spectator relay: Unix socket at `~/.myosu/spectate/<session>.sock` with fog-of-war at relay
- Trust boundaries: serde validation on context JSON, fog-of-war enforced at relay (not renderer)
- Trusted vs missing types table mapping existing code (schema.rs, pipe.rs) to pending slices

**`outputs/agent-integration/review.md`** — Review judgment:
- **Judgment: PROCEED — implementation-family workflow**
- `agent:experience` is the last remaining ready product lane (KEEP)
- Slices 1–2 (`agent_context.rs`, `journal.rs`) are fully unblocked — depend only on trusted `tui:shell`
- `robopoker` git migration is a Phase 2 gate, owned by `games:traits`, not a start gate
- `play:tui` binary is a Slice 3+ gate, not a Slice 1 gate
- Concrete next step: begin `agent:experience` Slice 1 immediately

---

**Lane decision**: The product is ready for implementation-family workflow. No additional upstream unblock required before beginning `agent:experience` Slice 1 (`agent_context.rs`).