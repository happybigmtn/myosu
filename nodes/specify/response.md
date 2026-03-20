Both required artifacts are produced. Here is the honest summary of this integration lane:

---

## What Was Done

**Executed `agent:experience`** — the last ready product lane. Its `spec.md` and `review.md` (`outputs/agent/experience/`) were produced in a prior session and are sound. I read them, audited the upstream dependencies (`tui:shell` 82 tests, `games:traits` 14 tests, `schema.rs` 16 tests), and synthesized the integration decision.

**Produced `outputs/agent-integration/agent-adapter.md`**
- Documents the integration contract: what `agent:experience` requires from upstream and what it provides to the product
- Maps the 9 implementation slices into three dependency tiers (immediate / requires binary / requires chain)
- Establishes the data directory convention (`~/.myosu/agents/<name>/`, `~/.myosu/spectate/`)
- Records four product-level adapter decisions

**Produced `outputs/agent-integration/review.md`**
- Judgment: **PROCEED** — implementation-family workflow
- Core argument: Slices 1–4 depend only on trusted `tui:shell` + `games:traits` — no reason to wait. `play:tui` binary (sequential blocker) continues in parallel. No new upstream unblock lane needed.
- Documents three remaining blockers with ownership (`play:tui` owns the binary; `games:traits` already resolved robopoker git migration; the `game_state_schema_present` path in the manifest is a false positive — the file exists)

**The decision**: The product frontier needs an **implementation-family workflow next**, not another upstream unblock. Start `agent:experience` Slices 1–4 now; start `play:tui` Slice 1 concurrently.