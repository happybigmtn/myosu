`★ Insight ─────────────────────────────────────`
The `agent-integration` lane synthesizes the `agent:experience` review and produces an **integration adapter** that maps product surfaces to concrete dependencies and sequencing. The key insight is that the integration decision is structural, not product: the nine slices have a strict dependency ordering where Phase 1 (Slices 1–4) has zero external dependencies beyond the already-trusted `tui:shell`, making immediate implementation feasible despite the robopoker blocker affecting later phases.
`─────────────────────────────────────────────────`

Both required artifacts are now in place:

```
outputs/agent-integration/
├── agent-adapter.md   (16,993 bytes) — integration map + data flow + slice sequencing
└── review.md          (7,781 bytes)  — KEEP verdict + readiness assessment
```

**Decision: proceed to implementation-family workflow.**

The reviewed artifacts from `agent:experience` lead to this conclusion because:

| Gate | Status | Implication |
|------|--------|-------------|
| `tui:shell` (upstream) | **TRUSTED** (82 tests) | No risk here |
| `games:traits` (upstream) | **TRUSTED** (14 tests) | No risk here |
| `play:tui` binary | **MISSING** | Blocks Slice 3+ but can proceed concurrently |
| `robopoker` git migration | **BLOCKER** | Blocks Phase 2+ integration testing; owned by `games:traits` lane |
| `chain:runtime` | **MISSING** | Stub lobby and deferred WebSocket for Phase 4 |

**Phase 1 (Slices 1–4):** Can begin immediately. `agent_context.rs`, `journal.rs`, `--context` wiring, and `reflect>` prompt all depend only on `tui:shell` which is already trusted.

**Phase 2+:** Gated on robopoker git migration resolution (owned by `games:traits`) and `play:tui` binary skeleton (owned by `play:tui` lane).

The upstream unblock is real but **owned elsewhere** — it does not block this lane from starting.