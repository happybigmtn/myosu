`★ Insight ─────────────────────────────────────`
**Why an Adapter Pattern Here?**

The `AgentAdapter` isn't a new abstraction — it's a **porcelain layer** that wires existing components without coupling them. The key insight is:

1. **Decoupling**: `PipeMode` shouldn't need to know about `CfrGame`, `Profile`, or the chain client directly. It just needs to translate actions, load/save context, and emit events.

2. **Testability**: `StubAgentAdapter` lets us test the entire `PipeMode` refactor without a live chain, miner, or `robopoker` dependency. This is critical because `robopoker` still uses absolute filesystem paths.

3. **Evolution**: When the real chain integration arrives (Phase 4), only `MyosuAgentAdapter::query_subnets()` changes — `PipeMode` and the trait boundary stay the same.

The 4 slices are intentionally narrow: trait+types → stub impl → pipe refactor → context refactor. Each slice modifies exactly one file (plus tests) and produces runnable evidence before proceeding.
`─────────────────────────────────────────────────`

## Lane Bootstrap Complete

**`agent:integration`** lane bootstrapped. Decision: **implementation family next**.

### Decision Rationale

The `agent:experience` review concluded "proceed to implementation-family workflow." `agent:integration` IS that workflow — the adapter layer that wires agent-facing surfaces (`PipeMode`, `AgentContext`, `Journal`, `SpectatorRelay`) to Myosu's core systems (`games:traits`, `play:tui`, chain client) without creating direct coupling.

### Key Findings

| Item | Status |
|------|--------|
| `agent:experience` | **COMPLETE** — KEEP/proceed |
| `agent:integration` spec | **BOOTSTRAPPED** — 4 slices defined |
| Primary blocker | `robopoker` git migration (owned by `games:traits`) |
| Secondary blocker | `play:tui` binary skeleton (needed for Slice 3) |
| Parallel opportunity | `agent:integration` Slices 1-2 can run in parallel with `agent:experience` Slice 1 |

### Required Artifacts Produced

- `outputs/agent-integration/agent-adapter.md` — `AgentAdapter` trait + types + 4 slices
- `outputs/agent-integration/review.md` — KEEP judgment + proof expectations + blockers

### Slice Map

```
Phase 1 (Adapter Core):
  Slice 1: AgentAdapter trait + StubAgentAdapter
  Slice 2: MyosuAgentAdapter skeleton (stubbed chain queries)
  Slice 3: Refactor PipeMode to use AgentAdapter
  Slice 4: Refactor AgentContext to use AgentAdapter

Phase 2: Spectator relay integration
Phase 3: Chain-connected queries (Phase 4 of agent:experience)
```

The lane is ready for execution. The primary upstream unblock (`robopoker` git migration) is tracked elsewhere — it doesn't prevent Slices 1–2 from completing and producing validated test evidence.