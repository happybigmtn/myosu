Both artifacts produced. Here's the summary:

---

## `agent-integration` Lane — Bootstrap Complete

**Artifacts produced:**
- `outputs/agent-integration/agent-adapter.md` — 17KB spec defining the integration layer
- `outputs/agent-integration/review.md` — 11KB honest review with judgment and blockers

---

### Decision: **Implementation-family workflow next**

`agent:experience` (the last remaining ready product lane) has already been reviewed and judged KEEP. Its upstream dependencies (`tui:shell`, `games:traits`) are trusted. The remaining blockers are owned by other lanes and do not prevent implementation work from starting.

### What `agent-integration` owns

The gap between `agent:experience` (presentation surfaces — pipe output, narration, journal, reflection, spectator relay) and the actual miner/validator infrastructure:

| Component | Role |
|-----------|------|
| `AgentAdapter` trait | Unified interface over 4 transports: pipe subprocess, HTTP, WebSocket, in-process Rust |
| `ChainDiscovery` trait | Subnet → miner axon URL resolution (devnet stub in Phase 1, real Substrate RPC in Phase 3) |
| `SessionManager` | Multi-hand session with context save/restore and reconnection logic |
| `myosu-agent` CLI | Binary wiring all transports with `--pipe`, `--http`, `--ws`, `--subnet`, `--context` flags |

### Key design choices in the adapter

- **`Send + Sync` on the trait** — enforces that all transports are thread-safe (required for multi-threaded session handling)
- **Adapter errors are typed, never stringly-typed** — `AdapterError` enum covers timeout, invalid response, unreachable, encoding, disconnected, chain error
- **Miner responses are untrusted** — `StrategyResponse` is serde-validated before constructing `ActionDistribution`
- **Recovery is local-only** — session context is saved to a `0o600` JSON file; miner is not re-notified of missed hands (correctness over completeness)
- **10 sequential slices with honest gates** — Phase 1 (trait + 2 adapters) has no external dependencies; Phase 3 defers to `chain:runtime`; Phase 4 defers to `agent:experience` Slice 8 (SpectatorRelay)

### Remaining blockers

| Blocker | Severity | Owner |
|---------|----------|-------|
| `myosu-play` binary missing | HIGH | `play:tui` lane |
| `SpectatorRelay` (AC-SP-01) missing | MEDIUM | `agent:experience` Slice 8 |
| `robopoker` git migration | LOW | `games:traits` lane (indirect) |

### Phase ordering

```
Phase 1 (local, no deps): Slice 1-3  → trait + PipeAdapter + HttpAdapter
Phase 2 (local):           Slice 4-6  → WsAdapter + ProcessAdapter + ChainDiscovery stub
Phase 3 (chain):           Slice 7-9  → real chain discovery + reconnection
Phase 4 (spectator):       Slice 10   → SpectatorRelay integration (blocked on agent:experience Slice 8)
```