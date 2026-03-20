All artifacts are in place. Here's what was created in this session:

**Created files:**

| File | Purpose |
|------|---------|
| `fabro/workflows/bootstrap/foundations.fabro` | Workflow graph following the specify→review→polish→verify pattern |
| `fabro/run-configs/bootstrap/foundations.toml` | Run config naming inputs, scope, and the two required artifact paths |
| `outputs/foundations/foundation-plan.md` | 6-lane inventory with trust levels, blocking relationships, and priority order |
| `outputs/foundations/review.md` | Trust judgments per lane, false-submit defect register, and next-execution recommendations |

**Key findings documented:**

- **`games:traits`**: KEEP, implementation-ready now
- **`tui:shell`**: KEEP (partial) — 3 modules have proof gaps but aren't blocking other lanes  
- **`chain:runtime` / `chain:pallet`**: RESET needed — non-building transplants confirmed
- **`games:poker-engine`**: GREENFIELD — spec is sound, crate doesn't exist yet
- **`games:multi-game`**: REOPEN — false-submit history must be resolved before re-running
- **`execute/status/watch`**: **NOT TRUSTWORTHY** — `games:multi-game` is the first confirmed instance of claim-without-delivery

**The two frontier tasks addressed honestly:**
1. The false-submit is the first real defect surfaced by execution — the rule "fix only when discovered" is the correct posture
2. `games:multi-game` needs a clean re-run through the proper Fabro path after diagnosing whether `goal_gate=true` on `verify` is passing when artifacts pre-exist